pub mod derive_nodes;

use crate::config::get_config;
use crate::entities::{
    files, libraries, metadata_source::MetadataSource, node_closure, node_files, node_metadata,
    nodes,
};
use crate::ids;
use crate::scanner::derive_nodes::{
    RootMaterializationPlan, WantedNode, build_closure_rows, build_root_materialization_plans,
    sort_nodes_topologically, verify_root_nodes,
};
use lyra_parser::{ParsedFile, parse_files};
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    TransactionTrait,
};
use std::collections::{HashMap, HashSet};
use std::mem;
use std::path::{Path as StdPath, PathBuf};
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{Duration, sleep};

const MIN_FILE_SIZE_MB: u64 = 25 * 1024 * 1024;
const SCAN_BATCH_SIZE: usize = 100;
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi", "mov", "webm"];
const PARKED_NODE_ORDER_OFFSET: i64 = 1_000_000_000;
const TEMP_NODE_ORDER_OFFSET: i64 = 2_000_000_000;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScannedFileCandidate {
    id: String,
    relative_path: String,
    size_bytes: i64,
}

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
    let scan_result = scan_directory(pool, library, &library_path, scan_start_time).await;
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

async fn scan_directory(
    pool: &DatabaseConnection,
    library: &libraries::Model,
    root_dir: &StdPath,
    scan_start_time: i64,
) -> anyhow::Result<()> {
    let mut pending = Vec::with_capacity(SCAN_BATCH_SIZE);
    let mut dirs = vec![root_dir.to_path_buf()];

    while let Some(current_dir) = dirs.pop() {
        let mut entries = tokio::fs::read_dir(&current_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            if !path.is_file() {
                continue;
            }

            let Some(candidate) = scan_file_candidate(library, &path, root_dir).await? else {
                continue;
            };
            pending.push(candidate);

            if pending.len() >= SCAN_BATCH_SIZE {
                flush_scan_batch(pool, library, root_dir, scan_start_time, &mut pending).await?;
            }
        }
    }

    if !pending.is_empty() {
        flush_scan_batch(pool, library, root_dir, scan_start_time, &mut pending).await?;
    }

    Ok(())
}

async fn scan_file_candidate(
    library: &libraries::Model,
    path: &PathBuf,
    root_dir: &StdPath,
) -> anyhow::Result<Option<ScannedFileCandidate>> {
    let Some(extension) = path.extension().and_then(|e| e.to_str()) else {
        return Ok(None);
    };

    if !VIDEO_EXTENSIONS.contains(&extension.to_lowercase().as_str()) {
        return Ok(None);
    }

    let metadata = match tokio::fs::metadata(path).await {
        Ok(metadata) => metadata,
        Err(_) => return Ok(None),
    };

    if metadata.len() < MIN_FILE_SIZE_MB {
        return Ok(None);
    }

    let relative_path = path
        .strip_prefix(root_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();

    Ok(Some(ScannedFileCandidate {
        id: file_id_for(&library.id, &relative_path),
        relative_path,
        size_bytes: metadata.len() as i64,
    }))
}

fn file_id_for(library_id: &str, relative_path: &str) -> String {
    ids::generate_hashid([library_id, relative_path])
}

async fn flush_scan_batch(
    pool: &DatabaseConnection,
    library: &libraries::Model,
    library_root: &StdPath,
    scan_start_time: i64,
    pending: &mut Vec<ScannedFileCandidate>,
) -> anyhow::Result<()> {
    let batch = mem::take(pending);
    if batch.is_empty() {
        return Ok(());
    }

    let batch_ids = batch
        .iter()
        .map(|candidate| candidate.id.clone())
        .collect::<Vec<_>>();
    let existing_rows = files::Entity::find()
        .filter(files::Column::Id.is_in(batch_ids.clone()))
        .all(pool)
        .await?;
    let existing_ids = existing_rows
        .iter()
        .map(|row| row.id.clone())
        .collect::<HashSet<_>>();

    let file_rows = batch
        .iter()
        .map(|candidate| files::ActiveModel {
            id: Set(candidate.id.clone()),
            library_id: Set(library.id.clone()),
            relative_path: Set(candidate.relative_path.clone()),
            size_bytes: Set(candidate.size_bytes),
            audio_fingerprint: Set(Vec::new()),
            segments_json: Set(Vec::new()),
            keyframes_json: Set(Vec::new()),
            scanned_at: Set(Some(scan_start_time)),
            unavailable_at: Set(None),
            discovered_at: Set(scan_start_time),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    let txn = pool.begin().await?;
    files::Entity::insert_many(file_rows)
        .on_conflict(
            OnConflict::columns([files::Column::LibraryId, files::Column::RelativePath])
                .update_columns([
                    files::Column::SizeBytes,
                    files::Column::ScannedAt,
                    files::Column::UnavailableAt,
                ])
                .to_owned(),
        )
        .exec(&txn)
        .await?;
    txn.commit().await?;

    let new_ids = batch
        .iter()
        .filter(|candidate| !existing_ids.contains(&candidate.id))
        .map(|candidate| candidate.id.clone())
        .collect::<Vec<_>>();
    if new_ids.is_empty() {
        return Ok(());
    }

    let inserted_rows = files::Entity::find()
        .filter(files::Column::Id.is_in(new_ids.clone()))
        .all(pool)
        .await?;
    let inserted_by_id = inserted_rows
        .into_iter()
        .map(|row| (row.id.clone(), row))
        .collect::<HashMap<_, _>>();
    let ordered_new_rows = batch
        .iter()
        .filter_map(|candidate| inserted_by_id.get(&candidate.id).cloned())
        .collect::<Vec<_>>();
    if ordered_new_rows.is_empty() {
        return Ok(());
    }

    let parsed_rows = parse_new_files(&ordered_new_rows).await;
    let root_plans = build_root_materialization_plans(library_root, &parsed_rows);

    for plan in root_plans.values() {
        if let Err(error) = materialize_touched_root(pool, &library.id, plan).await {
            tracing::warn!(
                library_id = %library.id,
                root_id = %plan.root_id,
                error = ?error,
                "failed to materialize touched root"
            );
        }
    }

    Ok(())
}

async fn parse_new_files(new_rows: &[files::Model]) -> Vec<(files::Model, ParsedFile)> {
    let relative_paths = new_rows
        .iter()
        .map(|file| file.relative_path.clone())
        .collect::<Vec<_>>();
    let parsed_rows = parse_files(relative_paths).await;

    new_rows
        .iter()
        .cloned()
        .zip(parsed_rows.into_iter())
        .collect::<Vec<_>>()
}

async fn materialize_touched_root(
    pool: &DatabaseConnection,
    library_id: &str,
    plan: &RootMaterializationPlan,
) -> anyhow::Result<()> {
    let now = chrono::Utc::now().timestamp();
    let txn = pool.begin().await?;

    let existing_nodes = nodes::Entity::find()
        .filter(nodes::Column::LibraryId.eq(library_id))
        .filter(nodes::Column::RootId.eq(plan.root_id.clone()))
        .all(&txn)
        .await?;
    let existing_node_ids = existing_nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<HashSet<_>>();

    let local_metadata_rows = if existing_node_ids.is_empty() {
        Vec::new()
    } else {
        node_metadata::Entity::find()
            .filter(node_metadata::Column::NodeId.is_in(existing_node_ids.iter().cloned()))
            .filter(node_metadata::Column::Source.eq(MetadataSource::Local))
            .all(&txn)
            .await?
    };
    let existing_local_metadata = local_metadata_rows
        .into_iter()
        .map(|row| (row.node_id.clone(), row))
        .collect::<HashMap<_, _>>();

    let mut combined_nodes = existing_nodes
        .iter()
        .map(|node| {
            (
                node.id.clone(),
                node_model_to_wanted(node, existing_local_metadata.get(&node.id)),
            )
        })
        .collect::<HashMap<_, _>>();
    for wanted in plan.wanted_nodes.values() {
        merge_wanted_node(&mut combined_nodes, wanted);
    }

    let combined_nodes_vec = combined_nodes.values().cloned().collect::<Vec<_>>();
    verify_root_nodes(&combined_nodes_vec)?;

    let sorted_wanted_nodes = sort_nodes_topologically(&plan.wanted_nodes)?;
    for (temp_order, wanted) in sorted_wanted_nodes.iter().enumerate() {
        let resolved = combined_nodes
            .get(&wanted.id)
            .ok_or_else(|| anyhow::anyhow!("missing resolved node {}", wanted.id))?;

        nodes::Entity::insert(nodes::ActiveModel {
            id: Set(resolved.id.clone()),
            library_id: Set(library_id.to_owned()),
            root_id: Set(resolved.root_id.clone()),
            parent_id: Set(resolved.parent_id.clone()),
            kind: Set(resolved.kind),
            name: Set(resolved.name.clone()),
            order: Set(TEMP_NODE_ORDER_OFFSET + temp_order as i64),
            season_number: Set(resolved.season_number),
            episode_number: Set(resolved.episode_number),
            match_candidates_json: Set(None),
            last_added_at: Set(resolved.last_added_at),
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

    let new_node_ids = sorted_wanted_nodes
        .iter()
        .filter(|node| !existing_node_ids.contains(&node.id))
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    if !new_node_ids.is_empty() {
        let closure_rows = build_closure_rows(&combined_nodes, &new_node_ids)?;
        for row in closure_rows {
            node_closure::Entity::insert(node_closure::ActiveModel {
                ancestor_id: Set(row.ancestor_id),
                descendant_id: Set(row.descendant_id),
                depth: Set(row.depth),
            })
            .exec(&txn)
            .await?;
        }
    }

    let new_links = plan
        .wanted_nodes
        .values()
        .flat_map(|node| {
            node.attached_file_ids
                .iter()
                .map(move |file_id| node_files::ActiveModel {
                    node_id: Set(node.id.clone()),
                    file_id: Set(file_id.clone()),
                    order: Set(0),
                    created_at: Set(now),
                    updated_at: Set(now),
                })
        })
        .collect::<Vec<_>>();
    for link in new_links {
        node_files::Entity::insert(link)
            .on_conflict(
                OnConflict::columns([node_files::Column::NodeId, node_files::Column::FileId])
                    .update_columns([node_files::Column::Order, node_files::Column::UpdatedAt])
                    .to_owned(),
            )
            .exec(&txn)
            .await?;
    }

    let local_rows = sorted_wanted_nodes
        .iter()
        .map(|node| {
            let resolved = combined_nodes
                .get(&node.id)
                .expect("resolved node missing during local metadata upsert");
            node_metadata::ActiveModel {
                id: Set(ids::generate_ulid()),
                node_id: Set(resolved.id.clone()),
                source: Set(MetadataSource::Local),
                provider_id: Set("local".to_owned()),
                imdb_id: Set(resolved.imdb_id.clone()),
                tmdb_id: Set(resolved.tmdb_id),
                name: Set(resolved.name.clone()),
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
            }
        })
        .collect::<Vec<_>>();
    for row in local_rows {
        node_metadata::Entity::insert(row)
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
    recompute_root_orders_with_sqlx(pool, &plan.root_id, now).await?;
    Ok(())
}

fn node_model_to_wanted(
    node: &nodes::Model,
    local_metadata: Option<&node_metadata::Model>,
) -> WantedNode {
    WantedNode {
        id: node.id.clone(),
        root_id: node.root_id.clone(),
        parent_id: node.parent_id.clone(),
        kind: node.kind,
        name: local_metadata
            .map(|metadata| metadata.name.clone())
            .unwrap_or_else(|| node.name.clone()),
        season_number: node.season_number,
        episode_number: node.episode_number,
        imdb_id: local_metadata.and_then(|metadata| metadata.imdb_id.clone()),
        tmdb_id: local_metadata.and_then(|metadata| metadata.tmdb_id),
        last_added_at: node.last_added_at,
        attached_file_ids: Vec::new(),
    }
}

fn merge_wanted_node(nodes_by_id: &mut HashMap<String, WantedNode>, next: &WantedNode) {
    if let Some(existing) = nodes_by_id.get_mut(&next.id) {
        existing.name = next.name.clone();
        existing.root_id = next.root_id.clone();
        existing.parent_id = next.parent_id.clone();
        existing.kind = next.kind;
        existing.season_number = next.season_number;
        existing.episode_number = next.episode_number;
        existing.last_added_at = existing.last_added_at.max(next.last_added_at);
        if existing.imdb_id.is_none() {
            existing.imdb_id = next.imdb_id.clone();
        }
        if existing.tmdb_id.is_none() {
            existing.tmdb_id = next.tmdb_id;
        }
        for file_id in &next.attached_file_ids {
            if !existing.attached_file_ids.contains(file_id) {
                existing.attached_file_ids.push(file_id.clone());
            }
        }
        return;
    }

    nodes_by_id.insert(next.id.clone(), next.clone());
}

async fn recompute_root_orders_with_sqlx(
    pool: &DatabaseConnection,
    root_id: &str,
    now: i64,
) -> anyhow::Result<()> {
    let mut txn = pool.get_sqlite_connection_pool().begin().await?;

    sqlx::query!(
        r#"UPDATE nodes SET "order" = "order" + ? WHERE root_id = ?"#,
        PARKED_NODE_ORDER_OFFSET,
        root_id,
    )
    .execute(&mut *txn)
    .await?;

    sqlx::query!(
        r#"
        WITH ranked AS (
            SELECT
                id,
                row_number() OVER (
                    ORDER BY
                        CASE WHEN parent_id IS NULL THEN 0 ELSE 1 END,
                        COALESCE(season_number, 0),
                        CASE WHEN kind = 2 THEN 0 ELSE 1 END,
                        COALESCE(episode_number, 0),
                        id
                ) - 1 AS new_order
            FROM nodes
            WHERE root_id = ?
        )
        UPDATE nodes
        SET "order" = (
            SELECT new_order
            FROM ranked
            WHERE ranked.id = nodes.id
        )
        WHERE root_id = ?
        "#,
        root_id,
        root_id,
    )
    .execute(&mut *txn)
    .await?;

    sqlx::query!(
        r#"
        WITH ranked AS (
            SELECT
                nf.node_id,
                nf.file_id,
                row_number() OVER (
                    PARTITION BY nf.node_id
                    ORDER BY f.size_bytes DESC, nf.file_id ASC
                ) - 1 AS new_order
            FROM node_files nf
            INNER JOIN files f ON f.id = nf.file_id
            INNER JOIN nodes n ON n.id = nf.node_id
            WHERE n.root_id = ?
            AND n.kind IN (0, 3)
        )
        UPDATE node_files
        SET
            "order" = (
                SELECT new_order
                FROM ranked
                WHERE ranked.node_id = node_files.node_id
                AND ranked.file_id = node_files.file_id
            ),
            updated_at = ?
        WHERE node_id IN (
            SELECT id
            FROM nodes
            WHERE root_id = ?
            AND kind IN (0, 3)
        )
        "#,
        root_id,
        now,
        root_id,
    )
    .execute(&mut *txn)
    .await?;

    txn.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, PaginatorTrait};

    async fn setup_test_db() -> anyhow::Result<DatabaseConnection> {
        let pool = Database::connect("sqlite::memory:").await?;
        sqlx::migrate!("../../migrations")
            .run(pool.get_sqlite_connection_pool())
            .await?;

        Ok(pool)
    }

    async fn insert_library(pool: &DatabaseConnection) -> anyhow::Result<()> {
        libraries::Entity::insert(libraries::ActiveModel {
            id: Set("lib".to_owned()),
            path: Set("/library".to_owned()),
            name: Set("Library".to_owned()),
            last_scanned_at: Set(None),
            created_at: Set(0),
        })
        .exec(pool)
        .await?;
        Ok(())
    }

    async fn insert_file(
        pool: &DatabaseConnection,
        id: &str,
        relative_path: &str,
        size_bytes: i64,
        discovered_at: i64,
    ) -> anyhow::Result<()> {
        files::Entity::insert(files::ActiveModel {
            id: Set(id.to_owned()),
            library_id: Set("lib".to_owned()),
            relative_path: Set(relative_path.to_owned()),
            size_bytes: Set(size_bytes),
            audio_fingerprint: Set(Vec::new()),
            segments_json: Set(Vec::new()),
            keyframes_json: Set(Vec::new()),
            unavailable_at: Set(None),
            scanned_at: Set(Some(discovered_at)),
            discovered_at: Set(discovered_at),
            ..Default::default()
        })
        .exec(pool)
        .await?;
        Ok(())
    }

    #[tokio::test]
    async fn materialize_touched_root_recomputes_orders() -> anyhow::Result<()> {
        let pool = setup_test_db().await?;
        insert_library(&pool).await?;
        insert_file(&pool, "file-small", "show/s01e01-small.mkv", 100, 10).await?;
        insert_file(&pool, "file-large", "show/s01e01-large.mkv", 200, 11).await?;

        let root_id = "root".to_owned();
        let season_id = "season-1".to_owned();
        let episode_id = "episode-1".to_owned();
        let plan = RootMaterializationPlan {
            root_id: root_id.clone(),
            wanted_nodes: HashMap::from([
                (
                    root_id.clone(),
                    WantedNode {
                        id: root_id.clone(),
                        root_id: root_id.clone(),
                        parent_id: None,
                        kind: nodes::NodeKind::Series,
                        name: "Show".to_owned(),
                        season_number: None,
                        episode_number: None,
                        imdb_id: Some("tt1234567".to_owned()),
                        tmdb_id: Some(42),
                        last_added_at: 11,
                        attached_file_ids: Vec::new(),
                    },
                ),
                (
                    season_id.clone(),
                    WantedNode {
                        id: season_id.clone(),
                        root_id: root_id.clone(),
                        parent_id: Some(root_id.clone()),
                        kind: nodes::NodeKind::Season,
                        name: "Season 1".to_owned(),
                        season_number: Some(1),
                        episode_number: None,
                        imdb_id: None,
                        tmdb_id: None,
                        last_added_at: 11,
                        attached_file_ids: Vec::new(),
                    },
                ),
                (
                    episode_id.clone(),
                    WantedNode {
                        id: episode_id.clone(),
                        root_id: root_id.clone(),
                        parent_id: Some(season_id.clone()),
                        kind: nodes::NodeKind::Episode,
                        name: "Episode 1".to_owned(),
                        season_number: Some(1),
                        episode_number: Some(1),
                        imdb_id: None,
                        tmdb_id: None,
                        last_added_at: 11,
                        attached_file_ids: vec!["file-small".to_owned(), "file-large".to_owned()],
                    },
                ),
            ]),
        };

        materialize_touched_root(&pool, "lib", &plan).await?;

        let rows = nodes::Entity::find()
            .filter(nodes::Column::RootId.eq(root_id.clone()))
            .order_by_asc(nodes::Column::Order)
            .all(&pool)
            .await?;
        assert_eq!(
            rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
            vec!["root", "season-1", "episode-1"]
        );

        let links = node_files::Entity::find()
            .filter(node_files::Column::NodeId.eq(episode_id.clone()))
            .order_by_asc(node_files::Column::Order)
            .all(&pool)
            .await?;
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].file_id, "file-large");
        assert_eq!(links[1].file_id, "file-small");

        let metadata_rows = node_metadata::Entity::find()
            .filter(node_metadata::Column::Source.eq(MetadataSource::Local))
            .order_by_asc(node_metadata::Column::NodeId)
            .all(&pool)
            .await?;
        assert_eq!(metadata_rows.len(), 3);
        assert_eq!(
            metadata_rows
                .iter()
                .find(|row| row.node_id == root_id)
                .and_then(|row| row.imdb_id.clone())
                .as_deref(),
            Some("tt1234567")
        );

        Ok(())
    }

    #[tokio::test]
    async fn materialize_touched_root_preserves_match_candidates_and_local_row_count()
    -> anyhow::Result<()> {
        let pool = setup_test_db().await?;
        insert_library(&pool).await?;
        insert_file(&pool, "movie-file", "movie/movie.mkv", 300, 20).await?;

        nodes::Entity::insert(nodes::ActiveModel {
            id: Set("movie-root".to_owned()),
            library_id: Set("lib".to_owned()),
            root_id: Set("movie-root".to_owned()),
            parent_id: Set(None),
            kind: Set(nodes::NodeKind::Movie),
            name: Set("Old Movie".to_owned()),
            order: Set(0),
            season_number: Set(None),
            episode_number: Set(None),
            match_candidates_json: Set(Some(vec![1, 2, 3])),
            last_added_at: Set(1),
            created_at: Set(1),
            updated_at: Set(1),
        })
        .exec(&pool)
        .await?;

        let plan = RootMaterializationPlan {
            root_id: "movie-root".to_owned(),
            wanted_nodes: HashMap::from([(
                "movie-root".to_owned(),
                WantedNode {
                    id: "movie-root".to_owned(),
                    root_id: "movie-root".to_owned(),
                    parent_id: None,
                    kind: nodes::NodeKind::Movie,
                    name: "New Movie".to_owned(),
                    season_number: None,
                    episode_number: None,
                    imdb_id: Some("tt7654321".to_owned()),
                    tmdb_id: Some(7),
                    last_added_at: 20,
                    attached_file_ids: vec!["movie-file".to_owned()],
                },
            )]),
        };

        materialize_touched_root(&pool, "lib", &plan).await?;
        materialize_touched_root(&pool, "lib", &plan).await?;

        let row = nodes::Entity::find_by_id("movie-root")
            .one(&pool)
            .await?
            .expect("movie root missing");
        assert_eq!(row.match_candidates_json, Some(vec![1, 2, 3]));

        let metadata_rows = node_metadata::Entity::find()
            .filter(node_metadata::Column::NodeId.eq("movie-root"))
            .filter(node_metadata::Column::Source.eq(MetadataSource::Local))
            .all(&pool)
            .await?;
        assert_eq!(metadata_rows.len(), 1);
        assert_eq!(metadata_rows[0].name, "New Movie");
        assert_eq!(metadata_rows[0].imdb_id.as_deref(), Some("tt7654321"));

        let links = node_files::Entity::find()
            .filter(node_files::Column::NodeId.eq("movie-root"))
            .all(&pool)
            .await?;
        assert_eq!(links.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn materialize_touched_root_rejects_mixed_series_shapes() -> anyhow::Result<()> {
        let pool = setup_test_db().await?;
        insert_library(&pool).await?;
        insert_file(&pool, "new-file", "show/s01e02.mkv", 100, 30).await?;

        nodes::Entity::insert_many([
            nodes::ActiveModel {
                id: Set("root".to_owned()),
                library_id: Set("lib".to_owned()),
                root_id: Set("root".to_owned()),
                parent_id: Set(None),
                kind: Set(nodes::NodeKind::Series),
                name: Set("Show".to_owned()),
                order: Set(0),
                season_number: Set(None),
                episode_number: Set(None),
                match_candidates_json: Set(None),
                last_added_at: Set(1),
                created_at: Set(1),
                updated_at: Set(1),
            },
            nodes::ActiveModel {
                id: Set("episode-1".to_owned()),
                library_id: Set("lib".to_owned()),
                root_id: Set("root".to_owned()),
                parent_id: Set(Some("root".to_owned())),
                kind: Set(nodes::NodeKind::Episode),
                name: Set("Episode 1".to_owned()),
                order: Set(1),
                season_number: Set(None),
                episode_number: Set(Some(1)),
                match_candidates_json: Set(None),
                last_added_at: Set(1),
                created_at: Set(1),
                updated_at: Set(1),
            },
        ])
        .exec(&pool)
        .await?;

        let plan = RootMaterializationPlan {
            root_id: "root".to_owned(),
            wanted_nodes: HashMap::from([
                (
                    "root".to_owned(),
                    WantedNode {
                        id: "root".to_owned(),
                        root_id: "root".to_owned(),
                        parent_id: None,
                        kind: nodes::NodeKind::Series,
                        name: "Show".to_owned(),
                        season_number: None,
                        episode_number: None,
                        imdb_id: None,
                        tmdb_id: None,
                        last_added_at: 30,
                        attached_file_ids: Vec::new(),
                    },
                ),
                (
                    "season-1".to_owned(),
                    WantedNode {
                        id: "season-1".to_owned(),
                        root_id: "root".to_owned(),
                        parent_id: Some("root".to_owned()),
                        kind: nodes::NodeKind::Season,
                        name: "Season 1".to_owned(),
                        season_number: Some(1),
                        episode_number: None,
                        imdb_id: None,
                        tmdb_id: None,
                        last_added_at: 30,
                        attached_file_ids: Vec::new(),
                    },
                ),
                (
                    "episode-2".to_owned(),
                    WantedNode {
                        id: "episode-2".to_owned(),
                        root_id: "root".to_owned(),
                        parent_id: Some("season-1".to_owned()),
                        kind: nodes::NodeKind::Episode,
                        name: "Episode 2".to_owned(),
                        season_number: Some(1),
                        episode_number: Some(2),
                        imdb_id: None,
                        tmdb_id: None,
                        last_added_at: 30,
                        attached_file_ids: vec!["new-file".to_owned()],
                    },
                ),
            ]),
        };

        let result = materialize_touched_root(&pool, "lib", &plan).await;
        assert!(result.is_err());

        let season = nodes::Entity::find_by_id("season-1").one(&pool).await?;
        assert!(season.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn flush_scan_batch_heartbeats_existing_files_without_materializing() -> anyhow::Result<()>
    {
        let pool = setup_test_db().await?;
        insert_library(&pool).await?;
        insert_file(&pool, "existing-file", "show/existing.mkv", 111, 1).await?;

        let mut pending = vec![ScannedFileCandidate {
            id: "existing-file".to_owned(),
            relative_path: "show/existing.mkv".to_owned(),
            size_bytes: 222,
        }];

        flush_scan_batch(
            &pool,
            &libraries::Entity::find_by_id("lib")
                .one(&pool)
                .await?
                .expect("library missing"),
            StdPath::new("/library"),
            99,
            &mut pending,
        )
        .await?;

        let file = files::Entity::find_by_id("existing-file")
            .one(&pool)
            .await?
            .expect("file missing");
        assert_eq!(file.size_bytes, 222);
        assert_eq!(file.scanned_at, Some(99));

        let node_count = nodes::Entity::find().count(&pool).await?;
        assert_eq!(node_count, 0);

        Ok(())
    }
}
