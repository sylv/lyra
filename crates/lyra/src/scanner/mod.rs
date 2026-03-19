pub mod derive_nodes;
pub mod local;

use crate::config::get_config;
use crate::entities::{
    files, libraries, metadata_source::MetadataSource, node_closure, node_files, node_metadata,
    nodes,
};
use crate::ids;
use crate::scanner::derive_nodes::derive_library_media;
use crate::scanner::local::insert_local_node_metadata;
use lyra_parser::{ParsedFile, parse_files};
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, Order, QueryFilter, QueryOrder,
    QuerySelect, TransactionTrait,
};
use std::path::Path as StdPath;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{Duration, sleep};

const MIN_FILE_SIZE_MB: u64 = 25 * 1024 * 1024;
const PARSE_BATCH_SIZE: usize = 100;
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi", "mov", "webm"];

pub async fn start_scanner(
    pool: DatabaseConnection,
    wake_signal: Arc<Notify>,
) -> anyhow::Result<()> {
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
            scan_library(&pool, &library, &wake_signal).await?;
        } else {
            sleep(Duration::from_secs(5)).await;
        }
    }
}

pub async fn run_startup_scan(
    pool: &DatabaseConnection,
    wake_signal: &Arc<Notify>,
) -> anyhow::Result<()> {
    let libraries = libraries::Entity::find()
        .order_by_asc(libraries::Column::CreatedAt)
        .all(pool)
        .await?;

    for library in libraries {
        tracing::info!(library_id = %library.id, path = %library.path, "running startup library scan");
        scan_library(pool, &library, wake_signal).await?;
    }

    Ok(())
}

async fn scan_library(
    pool: &DatabaseConnection,
    library: &libraries::Model,
    wake_signal: &Arc<Notify>,
) -> anyhow::Result<()> {
    let scan_start_time = chrono::Utc::now().timestamp();
    let library_path = PathBuf::from(&library.path);

    // if the library root can't be read, treat the whole library as unavailable for this pass
    // instead of crashing startup or the scanner loop.
    let scan_result = scan_directory(pool, library, &library_path, &library_path, scan_start_time).await;
    if let Err(error) = scan_result {
        tracing::warn!(
            library_id = %library.id,
            path = %library.path,
            error = ?error,
            "library scan could not read root; marking missing files unavailable"
        );
    }

    files::Entity::update_many()
        .set(files::ActiveModel {
            unavailable_at: Set(Some(scan_start_time)),
            ..Default::default()
        })
        .filter(files::Column::LibraryId.eq(library.id.clone()))
        .filter(files::Column::ScannedAt.lt(scan_start_time))
        .filter(files::Column::UnavailableAt.is_null())
        .exec(pool)
        .await?;

    rebuild_library_media(pool, library, &library_path).await?;

    libraries::Entity::update(libraries::ActiveModel {
        id: Set(library.id.clone()),
        last_scanned_at: Set(Some(chrono::Utc::now().timestamp())),
        ..Default::default()
    })
    .exec(pool)
    .await?;

    wake_signal.notify_waiters();
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
        Err(_) => return Ok(()),
    };

    if metadata.len() < MIN_FILE_SIZE_MB {
        return Ok(());
    }

    let relative_path = path
        .strip_prefix(root_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();

    files::Entity::insert(files::ActiveModel {
        id: Set(ids::generate_hashid([
            library.id.as_str(),
            relative_path.as_str(),
        ])),
        library_id: Set(library.id.clone()),
        relative_path: Set(relative_path),
        size_bytes: Set(metadata.len() as i64),
        audio_fingerprint: Set(Vec::new()),
        segments_json: Set(Vec::new()),
        keyframes_json: Set(Vec::new()),
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

async fn rebuild_library_media(
    pool: &DatabaseConnection,
    library: &libraries::Model,
    library_root: &StdPath,
) -> anyhow::Result<()> {
    let available_files = files::Entity::find()
        .filter(files::Column::LibraryId.eq(library.id.clone()))
        .filter(files::Column::UnavailableAt.is_null())
        .order_by(files::Column::Id, Order::Asc)
        .all(pool)
        .await?;

    let mut parsed_files = Vec::with_capacity(available_files.len());
    for batch in available_files.chunks(PARSE_BATCH_SIZE) {
        let relative_paths = batch
            .iter()
            .map(|file| file.relative_path.clone())
            .collect::<Vec<_>>();
        let parsed_batch = parse_files(relative_paths).await;
        parsed_files.extend(
            batch
                .iter()
                .cloned()
                .zip(parsed_batch.into_iter())
                .collect::<Vec<(files::Model, ParsedFile)>>(),
        );
    }

    let derived = derive_library_media(library_root, &parsed_files)?;
    upsert_derived_media(pool, library.id.clone(), derived).await
}

async fn upsert_derived_media(
    pool: &DatabaseConnection,
    library_id: String,
    derived: derive_nodes::DerivedLibraryMedia,
) -> anyhow::Result<()> {
    let now = chrono::Utc::now().timestamp();
    let txn = pool.begin().await?;

    let node_ids = derived
        .nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    let playable_node_ids = derived
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, nodes::NodeKind::Movie | nodes::NodeKind::Episode))
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();

    if node_ids.is_empty() {
        nodes::Entity::delete_many()
            .filter(nodes::Column::LibraryId.eq(library_id))
            .exec(&txn)
            .await?;
        txn.commit().await?;
        return Ok(());
    }

    for node in &derived.nodes {
        let existing = nodes::Entity::find_by_id(node.id.clone()).one(&txn).await?;
        let match_candidates_json = existing.and_then(|row| {
            if matches!(node.kind, nodes::NodeKind::Movie | nodes::NodeKind::Series) {
                row.match_candidates_json
            } else {
                None
            }
        });

        nodes::Entity::insert(nodes::ActiveModel {
            id: Set(node.id.clone()),
            library_id: Set(library_id.clone()),
            root_id: Set(node.root_id.clone()),
            parent_id: Set(node.parent_id.clone()),
            kind: Set(node.kind),
            name: Set(node.name.clone()),
            order: Set(node.order),
            season_number: Set(node.season_number),
            episode_number: Set(node.episode_number),
            match_candidates_json: Set(match_candidates_json),
            last_added_at: Set(node.last_added_at),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .on_conflict(
            OnConflict::column(nodes::Column::Id)
                .update_columns([
                    nodes::Column::LibraryId,
                    nodes::Column::RootId,
                    nodes::Column::ParentId,
                    nodes::Column::Kind,
                    nodes::Column::Name,
                    nodes::Column::Order,
                    nodes::Column::SeasonNumber,
                    nodes::Column::EpisodeNumber,
                    nodes::Column::LastAddedAt,
                    nodes::Column::UpdatedAt,
                ])
                .to_owned(),
        )
        .exec(&txn)
        .await?;
    }

    nodes::Entity::delete_many()
        .filter(nodes::Column::LibraryId.eq(&library_id))
        .filter(nodes::Column::Id.is_not_in(node_ids.clone()))
        .exec(&txn)
        .await?;

    let library_node_ids = nodes::Entity::find()
        .filter(nodes::Column::LibraryId.eq(&library_id))
        .select_only()
        .column(nodes::Column::Id)
        .into_tuple::<String>()
        .all(&txn)
        .await?;

    if !library_node_ids.is_empty() {
        node_closure::Entity::delete_many()
            .filter(node_closure::Column::AncestorId.is_in(library_node_ids.clone()))
            .exec(&txn)
            .await?;
    }

    for row in &derived.closure {
        node_closure::Entity::insert(node_closure::ActiveModel {
            ancestor_id: Set(row.ancestor_id.clone()),
            descendant_id: Set(row.descendant_id.clone()),
            depth: Set(row.depth),
        })
        .exec(&txn)
        .await?;
    }

    if !playable_node_ids.is_empty() {
        node_files::Entity::delete_many()
            .filter(node_files::Column::NodeId.is_in(playable_node_ids.clone()))
            .exec(&txn)
            .await?;

        for row in &derived.node_files {
            node_files::Entity::insert(node_files::ActiveModel {
                node_id: Set(row.node_id.clone()),
                file_id: Set(row.file_id.clone()),
                order: Set(row.order),
                created_at: Set(now),
                updated_at: Set(now),
            })
            .exec(&txn)
            .await?;
        }
    }

    node_metadata::Entity::delete_many()
        .filter(node_metadata::Column::NodeId.is_in(node_ids.clone()))
        .filter(node_metadata::Column::Source.eq(MetadataSource::Local))
        .exec(&txn)
        .await?;

    for node in &derived.nodes {
        insert_local_node_metadata(
            &txn,
            &node.id,
            &node.name,
            node.imdb_id.clone(),
            node.tmdb_id,
            now,
        )
        .await?;
    }

    txn.commit().await?;
    Ok(())
}
