use crate::config::{get_config};
use crate::entities::{file, library};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait};
use sea_orm::{QueryFilter, Set};
use std::path::Path as StdPath;
use std::path::PathBuf;
use std::time::Instant;
use tokio::time::{sleep, Duration};
use tracing::info;

const MIN_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50MB 
const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "3gp", "ts", "m2ts",
];

struct ScanProgress {
    files_imported: u64,
    bytes_imported: u64,
    files_seen: u64,
    directories_seen: u64,
    last_log_time: Instant,
}

impl ScanProgress {
    fn new() -> Self {
        Self {
            files_imported: 0,
            bytes_imported: 0,
            files_seen: 0,
            directories_seen: 0,
            last_log_time: Instant::now(),
        }
    }

    fn log_progress_if_needed(&mut self, backend_name: &str) {
        if self.last_log_time.elapsed() >= Duration::from_secs(5) {
            info!(
                "Scan progress for backend '{}': {} files imported ({:.2} GB), {} files seen, {} directories seen",
                backend_name,
                self.files_imported,
                self.bytes_imported as f64 / (1024.0 * 1024.0 * 1024.0),
                self.files_seen,
                self.directories_seen
            );
            self.last_log_time = Instant::now();
        }
    }
}

pub async fn start_scanner(pool: DatabaseConnection) -> anyhow::Result<()> {
    loop {
        let config = get_config();
        let scan_ago_filter = chrono::Utc::now()
            - chrono::Duration::seconds(config.library_scan_interval);
        let to_scan = library::Entity::find()
            .filter(library::Column::LastScannedAt.lt(scan_ago_filter))
            .one(&pool)
            .await?;

        if let Some(library) = to_scan {
            scan_backend(&pool, &library).await?;
        } else {
            sleep(Duration::from_secs(30)).await;
        }
    }
}

async fn scan_backend(pool: &DatabaseConnection, library: &library::Model) -> anyhow::Result<()> {
    let scan_start_time = chrono::Utc::now().timestamp();
    let mut progress = ScanProgress::new();
    let library_path = PathBuf::from(&library.path);

    info!(
        "Scanning directory: {} for library: {}",
        library_path.display(),
        library.name
    );

    scan_directory(
        pool,
        &library,
        &library_path,
        &library_path,
        scan_start_time,
        &mut progress,
    )
    .await?;

    mark_missing_files_unavailable(pool, &library, scan_start_time).await?;

    progress.log_progress_if_needed(&library.name);
    tracing::info!("Scan completed for backend '{}'", library.name);

    Ok(())
}

async fn scan_directory(
    pool: &DatabaseConnection,
    library: &library::Model,
    root_dir: &StdPath,
    current_dir: &StdPath,
    scan_start_time: i64,
    progress: &mut ScanProgress,
) -> anyhow::Result<()> {
    let mut entries = tokio::fs::read_dir(current_dir).await?;
    progress.directories_seen += 1;
    progress.log_progress_if_needed(&library.name);

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            progress.files_seen += 1;
            Box::pin(scan_directory(
                pool,
                library,
                root_dir,
                &path,
                scan_start_time,
                progress,
            ))
            .await?;
        } else if path.is_file() {
            progress.files_seen += 1;
            scan_file(
                pool,
                library,
                &path,
                root_dir,
                scan_start_time,
                progress,
            )
            .await?;
        }

        progress.log_progress_if_needed(&library.name);
    }

    Ok(())
}

async fn scan_file(
    pool: &DatabaseConnection,
    library: &library::Model,
    path: &PathBuf,
    root_dir: &StdPath,
    scan_start_time: i64,
    progress: &mut ScanProgress,
) -> anyhow::Result<()> {
    let Some(extension) = path.extension() else {
        return Ok(());
    };

    let ext_str = extension.to_str().unwrap();
    if !VIDEO_EXTENSIONS.contains(&ext_str.to_lowercase().as_str()) {
        return Ok(());
    }

    let metadata = tokio::fs::metadata(&path).await;
    let Ok(metadata) = metadata else {
        tracing::error!(
            "error getting metadata for file {}, ignoring",
            path.display()
        );
        return Ok(());
    };

    if metadata.len() >= MIN_FILE_SIZE {
        let relative_path = path
            .strip_prefix(root_dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        let size_bytes_i64 = metadata.len() as i64;
        let file = file::ActiveModel {
            library_id: Set(library.id),
            relative_path: Set(relative_path),
            size_bytes: Set(Some(size_bytes_i64)),
            scanned_at: Set(scan_start_time),
            ..Default::default()
        };
        // file.insert(pool).await?;
        file::Entity::insert(file)
            .on_conflict(
                OnConflict::columns([file::Column::LibraryId, file::Column::RelativePath])
                    .update_columns([file::Column::SizeBytes, file::Column::ScannedAt])
                    .to_owned(),
            )
            .exec_with_returning(pool)
            .await?;

        progress.files_imported += 1;
        progress.bytes_imported += metadata.len();
    }

    Ok(())
}

async fn mark_missing_files_unavailable(
    pool: &DatabaseConnection,
    library: &library::Model,
    scan_start_time: i64,
) -> anyhow::Result<()> {
    file::Entity::update_many()
        .set(file::ActiveModel {
            unavailable_at: Set(Some(scan_start_time)),
            ..Default::default()
        })
        .filter(file::Column::LibraryId.eq(library.id))
        .filter(file::Column::ScannedAt.lt(scan_start_time))
        .filter(file::Column::UnavailableAt.is_null())
        .exec(pool)
        .await?;

    Ok(())
}
