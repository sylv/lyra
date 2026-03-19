pub mod derive_nodes;

use crate::config::get_config;
use crate::entities::{
    files, libraries, metadata_source::MetadataSource, node_closure, node_files, node_metadata,
    nodes,
};
use crate::ids;
use crate::scanner::derive_nodes::derive_library_media;
use lyra_parser::{ParsedFile, parse_files};
use sea_orm::sea_query::{Expr, OnConflict};
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
const PARKED_NODE_ORDER_OFFSET: i64 = 1_000_000_000;

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
    let scan_result =
        scan_directory(pool, library, &library_path, &library_path, scan_start_time).await;
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

    let root_ids = derived
        .nodes
        .iter()
        .map(|node| node.root_id.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    // move current rows out of the final order range before applying reordered siblings.
    // sqlite enforces the unique (root_id, order) index immediately, so a direct in-place
    // update can fail during harmless swaps or insert-before-existing cases.
    park_existing_root_orders(&txn, &library_id, &root_ids).await?;

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

    // keep local metadata in sync with the derived nodes, but avoid delete-then-reinsert churn.
    // that pattern makes sqlite touch every row twice and fires the fts delete trigger for the
    // whole batch inside the scan transaction.
    for batch in derived.nodes.chunks(PARSE_BATCH_SIZE) {
        node_metadata::Entity::insert_many(batch.iter().map(|node| node_metadata::ActiveModel {
            id: Set(ids::generate_ulid()),
            node_id: Set(node.id.clone()),
            source: Set(MetadataSource::Local),
            provider_id: Set("local".to_owned()),
            imdb_id: Set(node.imdb_id.clone()),
            tmdb_id: Set(node.tmdb_id),
            name: Set(node.name.clone()),
            description: Set(None),
            score_display: Set(None),
            score_normalized: Set(None),
            released_at: Set(None),
            ended_at: Set(None),
            poster_asset_id: Set(None),
            thumbnail_asset_id: Set(None),
            background_asset_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        }))
        .on_conflict(
            OnConflict::columns([node_metadata::Column::NodeId, node_metadata::Column::Source])
                .update_columns([
                    node_metadata::Column::ProviderId,
                    node_metadata::Column::ImdbId,
                    node_metadata::Column::TmdbId,
                    node_metadata::Column::Name,
                    node_metadata::Column::Description,
                    node_metadata::Column::ScoreDisplay,
                    node_metadata::Column::ScoreNormalized,
                    node_metadata::Column::ReleasedAt,
                    node_metadata::Column::EndedAt,
                    node_metadata::Column::PosterAssetId,
                    node_metadata::Column::ThumbnailAssetId,
                    node_metadata::Column::BackgroundAssetId,
                    node_metadata::Column::UpdatedAt,
                ])
                .to_owned(),
        )
        .exec(&txn)
        .await?;
    }

    txn.commit().await?;
    Ok(())
}

async fn park_existing_root_orders(
    txn: &impl sea_orm::ConnectionTrait,
    library_id: &str,
    root_ids: &[String],
) -> anyhow::Result<()> {
    if root_ids.is_empty() {
        return Ok(());
    }

    nodes::Entity::update_many()
        .col_expr(
            nodes::Column::Order,
            Expr::col(nodes::Column::Order).add(PARKED_NODE_ORDER_OFFSET),
        )
        .filter(nodes::Column::LibraryId.eq(library_id))
        .filter(nodes::Column::RootId.is_in(root_ids.iter().cloned()))
        .exec(txn)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{libraries, node_metadata, nodes};
    use sea_orm::{Database, QueryFilter};

    async fn setup_test_db() -> anyhow::Result<DatabaseConnection> {
        let pool = Database::connect("sqlite::memory:").await?;
        sqlx::migrate!("../../migrations")
            .run(pool.get_sqlite_connection_pool())
            .await?;

        Ok(pool)
    }

    #[tokio::test]
    async fn upsert_derived_media_handles_reordered_root_children() -> anyhow::Result<()> {
        let pool = setup_test_db().await?;

        libraries::Entity::insert(libraries::ActiveModel {
            id: Set("lib".to_owned()),
            path: Set("/library".to_owned()),
            name: Set("Library".to_owned()),
            last_scanned_at: Set(None),
            created_at: Set(0),
        })
        .exec(&pool)
        .await?;

        let root_id = "root".to_owned();
        let episode_id = "episode-1".to_owned();
        nodes::Entity::insert_many([
            nodes::ActiveModel {
                id: Set(root_id.clone()),
                library_id: Set("lib".to_owned()),
                root_id: Set(root_id.clone()),
                parent_id: Set(None),
                kind: Set(nodes::NodeKind::Series),
                name: Set("Show".to_owned()),
                order: Set(0),
                season_number: Set(None),
                episode_number: Set(None),
                match_candidates_json: Set(None),
                last_added_at: Set(0),
                created_at: Set(0),
                updated_at: Set(0),
            },
            nodes::ActiveModel {
                id: Set(episode_id.clone()),
                library_id: Set("lib".to_owned()),
                root_id: Set(root_id.clone()),
                parent_id: Set(Some(root_id.clone())),
                kind: Set(nodes::NodeKind::Episode),
                name: Set("Episode 1".to_owned()),
                order: Set(1),
                season_number: Set(None),
                episode_number: Set(Some(1)),
                match_candidates_json: Set(None),
                last_added_at: Set(0),
                created_at: Set(0),
                updated_at: Set(0),
            },
        ])
        .exec(&pool)
        .await?;

        let derived = derive_nodes::DerivedLibraryMedia {
            nodes: vec![
                derive_nodes::DerivedNode {
                    id: root_id.clone(),
                    root_id: root_id.clone(),
                    parent_id: None,
                    kind: nodes::NodeKind::Series,
                    name: "Show".to_owned(),
                    order: 0,
                    season_number: None,
                    episode_number: None,
                    imdb_id: None,
                    tmdb_id: None,
                    last_added_at: 0,
                },
                derive_nodes::DerivedNode {
                    id: "season-1".to_owned(),
                    root_id: root_id.clone(),
                    parent_id: Some(root_id.clone()),
                    kind: nodes::NodeKind::Season,
                    name: "Season 1".to_owned(),
                    order: 1,
                    season_number: Some(1),
                    episode_number: None,
                    imdb_id: None,
                    tmdb_id: None,
                    last_added_at: 0,
                },
                derive_nodes::DerivedNode {
                    id: episode_id.clone(),
                    root_id: root_id.clone(),
                    parent_id: Some("season-1".to_owned()),
                    kind: nodes::NodeKind::Episode,
                    name: "Episode 1".to_owned(),
                    order: 2,
                    season_number: Some(1),
                    episode_number: Some(1),
                    imdb_id: None,
                    tmdb_id: None,
                    last_added_at: 0,
                },
            ],
            node_files: Vec::new(),
            closure: vec![
                node_closure::Model {
                    ancestor_id: root_id.clone(),
                    descendant_id: root_id.clone(),
                    depth: 0,
                },
                node_closure::Model {
                    ancestor_id: "season-1".to_owned(),
                    descendant_id: "season-1".to_owned(),
                    depth: 0,
                },
                node_closure::Model {
                    ancestor_id: root_id.clone(),
                    descendant_id: "season-1".to_owned(),
                    depth: 1,
                },
                node_closure::Model {
                    ancestor_id: episode_id.clone(),
                    descendant_id: episode_id.clone(),
                    depth: 0,
                },
                node_closure::Model {
                    ancestor_id: "season-1".to_owned(),
                    descendant_id: episode_id.clone(),
                    depth: 1,
                },
                node_closure::Model {
                    ancestor_id: root_id.clone(),
                    descendant_id: episode_id.clone(),
                    depth: 2,
                },
            ],
        };

        upsert_derived_media(&pool, "lib".to_owned(), derived).await?;

        let rows = nodes::Entity::find()
            .filter(nodes::Column::RootId.eq(root_id.clone()))
            .order_by_asc(nodes::Column::Order)
            .all(&pool)
            .await?;

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].id, root_id);
        assert_eq!(rows[0].order, 0);
        assert_eq!(rows[1].id, "season-1");
        assert_eq!(rows[1].order, 1);
        assert_eq!(rows[2].id, episode_id);
        assert_eq!(rows[2].order, 2);

        Ok(())
    }

    #[tokio::test]
    async fn upsert_derived_media_keeps_one_local_metadata_row_per_node() -> anyhow::Result<()> {
        let pool = setup_test_db().await?;

        libraries::Entity::insert(libraries::ActiveModel {
            id: Set("lib".to_owned()),
            path: Set("/library".to_owned()),
            name: Set("Library".to_owned()),
            last_scanned_at: Set(None),
            created_at: Set(0),
        })
        .exec(&pool)
        .await?;

        let root_id = "root".to_owned();
        let episode_id = "episode-1".to_owned();
        let derived = derive_nodes::DerivedLibraryMedia {
            nodes: vec![
                derive_nodes::DerivedNode {
                    id: root_id.clone(),
                    root_id: root_id.clone(),
                    parent_id: None,
                    kind: nodes::NodeKind::Series,
                    name: "Show".to_owned(),
                    order: 0,
                    season_number: None,
                    episode_number: None,
                    imdb_id: Some("tt1234567".to_owned()),
                    tmdb_id: Some(42),
                    last_added_at: 0,
                },
                derive_nodes::DerivedNode {
                    id: episode_id.clone(),
                    root_id: root_id.clone(),
                    parent_id: Some(root_id.clone()),
                    kind: nodes::NodeKind::Episode,
                    name: "Episode 1".to_owned(),
                    order: 1,
                    season_number: Some(1),
                    episode_number: Some(1),
                    imdb_id: None,
                    tmdb_id: None,
                    last_added_at: 0,
                },
            ],
            node_files: Vec::new(),
            closure: vec![
                node_closure::Model {
                    ancestor_id: root_id.clone(),
                    descendant_id: root_id.clone(),
                    depth: 0,
                },
                node_closure::Model {
                    ancestor_id: episode_id.clone(),
                    descendant_id: episode_id.clone(),
                    depth: 0,
                },
                node_closure::Model {
                    ancestor_id: root_id.clone(),
                    descendant_id: episode_id.clone(),
                    depth: 1,
                },
            ],
        };

        upsert_derived_media(&pool, "lib".to_owned(), derived.clone()).await?;
        upsert_derived_media(&pool, "lib".to_owned(), derived).await?;

        let metadata_rows = node_metadata::Entity::find()
            .filter(node_metadata::Column::Source.eq(MetadataSource::Local))
            .all(&pool)
            .await?;

        assert_eq!(metadata_rows.len(), 2);
        assert_eq!(
            metadata_rows
                .iter()
                .filter(|row| row.node_id == root_id && row.name == "Show")
                .count(),
            1
        );
        assert_eq!(
            metadata_rows
                .iter()
                .filter(|row| row.node_id == episode_id && row.name == "Episode 1")
                .count(),
            1
        );

        Ok(())
    }
}
