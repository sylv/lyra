pub mod local;
pub mod node_generation;

use crate::config::get_config;
use crate::entities::{files, libraries, node_metadata, nodes};
use crate::scanner::local::upsert_local_metadata_for_node;
use crate::scanner::node_generation::get_recommended_nodes_for_file;
use lyra_parser::{ParsedFile, parse_files};
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QuerySelect,
    RelationTrait, Set, TransactionTrait,
};
use std::collections::HashMap;
use std::path::Path as StdPath;
use std::path::PathBuf;
use tokio::time::{Duration, sleep};
use tracing::info;

const MIN_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50MB
const PARSE_BATCH_SIZE: usize = 100;
const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "3gp", "ts", "m2ts",
];

pub async fn start_scanner(pool: DatabaseConnection) -> anyhow::Result<()> {
    loop {
        let config = get_config();
        let scan_ago_filter = chrono::Utc::now().timestamp() - config.library_scan_interval;
        let to_scan = libraries::Entity::find()
            .filter(
                libraries::Column::LastScannedAt
                    .lt(scan_ago_filter)
                    .or(libraries::Column::LastScannedAt.is_null()),
            )
            .one(&pool)
            .await?;

        if let Some(library) = to_scan {
            scan_library(&pool, &library).await?;
        } else {
            sleep(Duration::from_secs(5)).await;
        }
    }
}

async fn scan_library(pool: &DatabaseConnection, library: &libraries::Model) -> anyhow::Result<()> {
    let scan_start_time = chrono::Utc::now().timestamp();
    let library_path = PathBuf::from(&library.path);

    info!(
        "Scanning directory: {} for library: {}",
        library_path.display(),
        library.name
    );

    scan_directory(pool, library, &library_path, &library_path, scan_start_time).await?;

    files::Entity::update_many()
        .set(files::ActiveModel {
            unavailable_at: Set(Some(scan_start_time)),
            ..Default::default()
        })
        .filter(files::Column::LibraryId.eq(library.id))
        .filter(files::Column::ScannedAt.lt(scan_start_time))
        .filter(files::Column::UnavailableAt.is_null())
        .exec(pool)
        .await?;

    attach_nodes_for_pending_files(pool, library, &library_path, scan_start_time).await?;

    libraries::Entity::update(libraries::ActiveModel {
        id: Set(library.id),
        last_scanned_at: Set(Some(chrono::Utc::now().timestamp())),
        ..Default::default()
    })
    .exec(pool)
    .await?;

    tracing::info!("Scan completed for library '{}'", library.name);

    Ok(())
}

#[async_recursion::async_recursion]
async fn scan_directory(
    pool: &DatabaseConnection,
    library: &libraries::Model,
    root_dir: &StdPath,
    current_dir: &StdPath,
    scan_start_time: i64,
) -> anyhow::Result<()> {
    let mut entries = tokio::fs::read_dir(current_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            scan_directory(pool, library, root_dir, &path, scan_start_time).await?;
        } else if path.is_file() {
            scan_file(pool, library, &path, root_dir, scan_start_time).await?;
        }
    }

    Ok(())
}

async fn scan_file(
    pool: &DatabaseConnection,
    library: &libraries::Model,
    path: &PathBuf,
    root_dir: &StdPath,
    scan_start_time: i64,
) -> anyhow::Result<()> {
    let Some(extension) = path.extension().and_then(|e| e.to_str()) else {
        return Ok(());
    };

    if !VIDEO_EXTENSIONS.contains(&extension.to_lowercase().as_str()) {
        return Ok(());
    }

    let metadata = match tokio::fs::metadata(path).await {
        Ok(metadata) => metadata,
        Err(_) => {
            tracing::error!(
                "error getting metadata for file {}, ignoring",
                path.display()
            );
            return Ok(());
        }
    };

    if metadata.len() < MIN_FILE_SIZE {
        return Ok(());
    }

    let relative_path = path
        .strip_prefix(root_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();

    files::Entity::insert(files::ActiveModel {
        library_id: Set(library.id),
        relative_path: Set(relative_path.clone()),
        size_bytes: Set(metadata.len() as i64),
        scanned_at: Set(Some(scan_start_time)),
        unavailable_at: Set(None),
        ..Default::default()
    })
    .on_conflict(
        OnConflict::columns([files::Column::LibraryId, files::Column::RelativePath])
            .update_columns([
                files::Column::SizeBytes,
                files::Column::ScannedAt,
                files::Column::UnavailableAt,
            ])
            .to_owned(),
    )
    .exec(pool)
    .await?;

    Ok(())
}

struct PendingFile {
    file: files::Model,
    file_path: PathBuf,
}

async fn attach_nodes_for_pending_files(
    pool: &DatabaseConnection,
    library: &libraries::Model,
    library_root: &StdPath,
    scan_start_time: i64,
) -> anyhow::Result<()> {
    let mut pending_by_id: HashMap<i64, PendingFile> = HashMap::new();

    let updated_files = files::Entity::find()
        .filter(files::Column::LibraryId.eq(library.id))
        .filter(files::Column::ScannedAt.eq(scan_start_time))
        .filter(files::Column::UnavailableAt.is_null())
        .all(pool)
        .await?;
    for file in updated_files {
        pending_by_id.insert(
            file.id,
            PendingFile {
                file_path: library_root.join(&file.relative_path),
                file,
            },
        );
    }

    let unattached_files = files::Entity::find()
        .join(JoinType::LeftJoin, files::Relation::Nodes.def())
        .filter(files::Column::LibraryId.eq(library.id))
        .filter(files::Column::UnavailableAt.is_null())
        .filter(nodes::Column::Id.is_null())
        .all(pool)
        .await?;
    for file in unattached_files {
        pending_by_id.entry(file.id).or_insert_with(|| PendingFile {
            file_path: library_root.join(&file.relative_path),
            file,
        });
    }

    let mut pending_files = pending_by_id.into_values().collect::<Vec<_>>();
    pending_files.sort_by_key(|pending| pending.file.id);

    for batch in pending_files.chunks(PARSE_BATCH_SIZE) {
        let relative_paths = batch
            .iter()
            .map(|pending| pending.file.relative_path.clone())
            .collect::<Vec<_>>();
        let parsed_batch = parse_files(relative_paths).await;

        for (pending, parsed) in batch.iter().zip(parsed_batch.into_iter()) {
            let relative_path = &pending.file.relative_path;

            upsert_nodes_for_file(
                pool,
                library,
                library_root,
                &pending.file_path,
                relative_path,
                &pending.file,
                &parsed,
            )
            .await?;
        }
    }

    Ok(())
}

async fn upsert_nodes_for_file(
    pool: &DatabaseConnection,
    library: &libraries::Model,
    library_root: &StdPath,
    file_path: &StdPath,
    relative_path: &str,
    file: &files::Model,
    parsed: &ParsedFile,
) -> anyhow::Result<()> {
    let Some(recommended_nodes) = get_recommended_nodes_for_file(library_root, file_path, parsed)
    else {
        return Ok(());
    };

    let txn = pool.begin().await?;

    nodes::Entity::update_many()
        .set(nodes::ActiveModel {
            file_id: Set(None),
            ..Default::default()
        })
        .filter(nodes::Column::FileId.eq(file.id))
        .exec(&txn)
        .await?;

    let mut upserted_nodes = Vec::with_capacity(recommended_nodes.len());
    for node in recommended_nodes {
        let attached_file_id = if node.attach_file {
            Some(file.id)
        } else {
            None
        };

        nodes::Entity::insert(nodes::ActiveModel {
            id: Set(node.id.clone()),
            root_id: Set(node.root_id.clone()),
            parent_id: Set(node.parent_id.clone()),
            library_id: Set(library.id),
            file_id: Set(attached_file_id),
            relative_path: Set(relative_path.to_string()),
            name: Set(node.name.clone()),
            kind: Set(node.kind),
        })
        .on_conflict(
            OnConflict::column(nodes::Column::Id)
                .update_columns([
                    nodes::Column::RootId,
                    nodes::Column::ParentId,
                    nodes::Column::LibraryId,
                    nodes::Column::FileId,
                    nodes::Column::RelativePath,
                    nodes::Column::Name,
                    nodes::Column::Kind,
                ])
                .to_owned(),
        )
        .exec(&txn)
        .await?;

        upserted_nodes.push((
            nodes::Model {
                id: node.id,
                root_id: node.root_id,
                parent_id: node.parent_id,
                library_id: library.id,
                file_id: attached_file_id,
                relative_path: relative_path.to_string(),
                name: node.name,
                kind: node.kind,
            },
            node.episode_number,
        ));
    }

    txn.commit().await?;

    for (node, episode_number_hint) in &upserted_nodes {
        if let Err(error) =
            upsert_local_metadata_for_node(pool, node, parsed, *episode_number_hint).await
        {
            tracing::warn!(
                node_id = %node.id,
                relative_path = %relative_path,
                error = %error,
                "failed to upsert local metadata"
            );
        }
    }

    Ok(())
}
pub async fn ensure_node_metadata_link(
    pool: &DatabaseConnection,
    node_id: &str,
    metadata_id: i64,
    is_primary: bool,
) -> anyhow::Result<()> {
    let txn = pool.begin().await?;

    if is_primary {
        node_metadata::Entity::update_many()
            .set(node_metadata::ActiveModel {
                is_primary: Set(false),
                ..Default::default()
            })
            .filter(node_metadata::Column::NodeId.eq(node_id.to_string()))
            .exec(&txn)
            .await?;
    }

    node_metadata::Entity::insert(node_metadata::ActiveModel {
        node_id: Set(node_id.to_string()),
        metadata_id: Set(metadata_id),
        is_primary: Set(is_primary),
    })
    .on_conflict(
        OnConflict::columns([
            node_metadata::Column::NodeId,
            node_metadata::Column::MetadataId,
        ])
        .update_columns([node_metadata::Column::IsPrimary])
        .to_owned(),
    )
    .exec(&txn)
    .await?;

    txn.commit().await?;
    Ok(())
}
