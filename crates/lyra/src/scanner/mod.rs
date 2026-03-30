pub mod derive_nodes;
pub mod reconcile;

use crate::activity::{ActivityHandle, ActivityKind};
use crate::config::get_config;
use crate::content_update::CONTENT_UPDATE;
use crate::entities::{files, libraries};
use crate::ids;
use crate::scanner::derive_nodes::group_parsed_files_by_root;
use crate::scanner::reconcile::{find_roots_for_file_ids, parse_file_rows, reconcile_root};
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, TransactionTrait,
};
use std::collections::{HashMap, HashSet};
use std::mem;
use std::path::{Path as StdPath, PathBuf};
use std::sync::Arc;
use tokio::sync::{Notify, RwLock};
use tokio::time::{Duration, sleep};

const MIN_FILE_SIZE_MB: u64 = 25 * 1024 * 1024;
const SCAN_BATCH_SIZE: usize = 100;
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi", "mov", "webm"];

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScannedFileCandidate {
    id: String,
    relative_path: String,
    size_bytes: i64,
}

pub async fn start_scanner(
    pool: DatabaseConnection,
    wake_signal: Arc<Notify>,
    job_startup_lock: Arc<RwLock<()>>,
) -> anyhow::Result<()> {
    let startup_guard = job_startup_lock.write().await;
    let libraries = libraries::Entity::find()
        .order_by_asc(libraries::Column::CreatedAt)
        .all(&pool)
        .await?;

    // The first full pass establishes which files are still available before any jobs can run.
    for library in libraries {
        tracing::info!(
            library_id = %library.id,
            path = %library.path,
            "running startup library scan"
        );
        scan_library(&pool, &library, &wake_signal).await?;
    }

    tracing::info!("startup scans complete");
    drop(startup_guard);

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

async fn scan_library(
    pool: &DatabaseConnection,
    library: &libraries::Model,
    wake_signal: &Arc<Notify>,
) -> anyhow::Result<()> {
    let mut activity = ActivityHandle::new(ActivityKind::LibraryScan);
    let scan_start_time = chrono::Utc::now().timestamp();
    let library_path = PathBuf::from(&library.path);
    let mut new_file_ids = HashSet::new();

    // if the library root can't be read, treat the whole library as unavailable for this pass
    // instead of crashing startup or the scanner loop.
    let scan_result = scan_directory(pool, library, &library_path, scan_start_time).await;
    if let Ok(scanned_new_file_ids) = &scan_result {
        new_file_ids.extend(scanned_new_file_ids.iter().cloned());
    }
    if let Err(error) = scan_result {
        tracing::warn!(
            library_id = %library.id,
            path = %library.path,
            error = ?error,
            "library scan could not read root; marking missing files unavailable"
        );
    }

    let newly_unavailable_file_ids = files::Entity::find()
        .filter(files::Column::LibraryId.eq(library.id.clone()))
        .filter(files::Column::ScannedAt.lt(scan_start_time))
        .filter(files::Column::UnavailableAt.is_null())
        .select_only()
        .column(files::Column::Id)
        .into_tuple::<String>()
        .all(pool)
        .await?;

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

    let mut parsed_new_files_by_root = HashMap::new();
    if !new_file_ids.is_empty() {
        let new_rows = files::Entity::find()
            .filter(files::Column::Id.is_in(new_file_ids.into_iter().collect::<Vec<_>>()))
            .all(pool)
            .await?;
        let parsed_new_rows = parse_file_rows(&new_rows).await;
        parsed_new_files_by_root = group_parsed_files_by_root(&library_path, &parsed_new_rows);

        let total_import_files = parsed_new_files_by_root
            .values()
            .map(|rows| rows.len() as i64)
            .sum::<i64>();
        if total_import_files > 0 {
            activity.set_total(total_import_files);
        }
    }

    let mut touched_root_ids = parsed_new_files_by_root
        .keys()
        .cloned()
        .collect::<HashSet<_>>();
    touched_root_ids.extend(
        find_roots_for_file_ids(pool, &newly_unavailable_file_ids)
            .await?
            .into_iter(),
    );

    let mut processed_import_files = 0_i64;
    for root_id in touched_root_ids {
        let extra_rows = parsed_new_files_by_root
            .get(root_id.as_str())
            .cloned()
            .unwrap_or_default();
        let imported_file_count = extra_rows.len() as i64;

        if let Err(error) =
            reconcile_root(pool, &library.id, &library_path, &root_id, extra_rows).await
        {
            tracing::warn!(
                library_id = %library.id,
                root_id,
                error = ?error,
                "failed to reconcile touched root"
            );
        }

        if imported_file_count > 0 {
            processed_import_files += imported_file_count;
            activity.set_progress(processed_import_files);
        }
    }

    libraries::Entity::update(libraries::ActiveModel {
        id: Set(library.id.clone()),
        last_scanned_at: Set(Some(chrono::Utc::now().timestamp())),
        ..Default::default()
    })
    .exec(pool)
    .await?;

    wake_signal.notify_waiters();
    CONTENT_UPDATE.emit();
    Ok(())
}

async fn scan_directory(
    pool: &DatabaseConnection,
    library: &libraries::Model,
    root_dir: &StdPath,
    scan_start_time: i64,
) -> anyhow::Result<HashSet<String>> {
    let mut pending = Vec::with_capacity(SCAN_BATCH_SIZE);
    let mut dirs = vec![root_dir.to_path_buf()];
    let mut new_file_ids = HashSet::new();

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
                new_file_ids
                    .extend(flush_scan_batch(pool, library, scan_start_time, &mut pending).await?);
            }
        }
    }

    if !pending.is_empty() {
        new_file_ids.extend(flush_scan_batch(pool, library, scan_start_time, &mut pending).await?);
    }

    Ok(new_file_ids)
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

    let size_bytes = metadata.len();
    Ok(Some(ScannedFileCandidate {
        id: file_id_for(&library.id, &relative_path, size_bytes),
        relative_path,
        size_bytes: size_bytes as i64,
    }))
}

fn file_id_for(library_id: &str, relative_path: &str, file_size: u64) -> String {
    ids::generate_hashid([library_id, relative_path, &file_size.to_string()])
}

async fn flush_scan_batch(
    pool: &DatabaseConnection,
    library: &libraries::Model,
    scan_start_time: i64,
    pending: &mut Vec<ScannedFileCandidate>,
) -> anyhow::Result<Vec<String>> {
    let batch = mem::take(pending);
    if batch.is_empty() {
        return Ok(Vec::new());
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
            audio_fingerprint: Set(None),
            segments_json: Set(None),
            keyframes_json: Set(None),
            scanned_at: Set(Some(scan_start_time)),
            unavailable_at: Set(None),
            discovered_at: Set(scan_start_time),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    let txn = pool.begin().await?;
    files::Entity::insert_many(file_rows)
        .on_conflict(
            OnConflict::column(files::Column::Id)
                .update_columns([
                    files::Column::RelativePath,
                    files::Column::SizeBytes,
                    files::Column::ScannedAt,
                    files::Column::UnavailableAt,
                ])
                .to_owned(),
        )
        .exec(&txn)
        .await?;
    txn.commit().await?;

    Ok(batch
        .iter()
        .filter(|candidate| !existing_ids.contains(&candidate.id))
        .map(|candidate| candidate.id.clone())
        .collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::nodes;
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
            audio_fingerprint: Set(None),
            segments_json: Set(None),
            keyframes_json: Set(None),
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
    async fn flush_scan_batch_keeps_previous_version_for_same_path() -> anyhow::Result<()> {
        let pool = setup_test_db().await?;
        insert_library(&pool).await?;
        insert_file(&pool, "existing-file", "show/existing.mkv", 111, 1).await?;

        let mut pending = vec![ScannedFileCandidate {
            id: "replacement-file".to_owned(),
            relative_path: "show/existing.mkv".to_owned(),
            size_bytes: 222,
        }];

        let new_ids = flush_scan_batch(
            &pool,
            &libraries::Entity::find_by_id("lib")
                .one(&pool)
                .await?
                .expect("library missing"),
            99,
            &mut pending,
        )
        .await?;
        assert_eq!(new_ids, vec!["replacement-file".to_owned()]);

        let existing = files::Entity::find_by_id("existing-file")
            .one(&pool)
            .await?
            .expect("existing file missing");
        assert_eq!(existing.size_bytes, 111);
        assert_eq!(existing.scanned_at, Some(1));

        let replacement = files::Entity::find_by_id("replacement-file")
            .one(&pool)
            .await?
            .expect("replacement file missing");
        assert_eq!(replacement.size_bytes, 222);
        assert_eq!(replacement.scanned_at, Some(99));

        let node_count = nodes::Entity::find().count(&pool).await?;
        assert_eq!(node_count, 0);

        Ok(())
    }

    #[test]
    fn file_ids() {
        assert_eq!(
            file_id_for("lib", "Show/Episode.mkv", 111),
            file_id_for("lib", "show/episode.mkv", 111)
        );
        assert_ne!(
            file_id_for("lib", "Show/episode.mkv", 111),
            file_id_for("lib", "show/episode.mkv", 222)
        );
    }
}
