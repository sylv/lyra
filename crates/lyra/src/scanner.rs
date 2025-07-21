use crate::config::{Backend, get_config};
use crate::entities::file;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait};
use sea_orm::{QueryFilter, Set};
use std::path::Path as StdPath;
use std::path::PathBuf;
use std::time::Instant;
use tokio::time::{Duration, interval};
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

pub async fn start_scanner(pool: DatabaseConnection) {
    let config = get_config();
    let mut interval = interval(Duration::from_secs(4 * 60 * 60)); // 4 hours

    #[cfg(not(debug_assertions))]
    {
        // grace period in case we're in a crash loop or something, hitting each backend
        // on startup repeatedly would be rude, but in dev its convenient
        tokio::time::sleep(Duration::from_secs(10)).await;
    }

    loop {
        interval.tick().await;
        info!("Starting file scan");
        for backend in &config.backends {
            scan_backend(&pool, backend)
                .await
                .expect("failed to scan backend")
        }
    }
}

async fn scan_backend(pool: &DatabaseConnection, backend: &Backend) -> anyhow::Result<()> {
    let scan_start_time = chrono::Utc::now().timestamp();
    let mut progress = ScanProgress::new();

    info!(
        "Scanning directory: {} for backend: {}",
        &backend.root_dir.display(),
        backend.name
    );

    scan_directory(
        pool,
        &backend.name,
        &backend.root_dir,
        &backend.root_dir,
        scan_start_time,
        &mut progress,
    )
    .await?;

    mark_missing_files_unavailable(pool, &backend.name, scan_start_time).await?;

    progress.log_progress_if_needed(&backend.name);
    tracing::info!("Scan completed for backend '{}'", backend.name);

    Ok(())
}

async fn scan_directory(
    pool: &DatabaseConnection,
    backend_name: &str,
    root_dir: &StdPath,
    current_dir: &StdPath,
    scan_start_time: i64,
    progress: &mut ScanProgress,
) -> anyhow::Result<()> {
    let mut entries = tokio::fs::read_dir(current_dir).await?;
    progress.directories_seen += 1;
    progress.log_progress_if_needed(backend_name);

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            progress.files_seen += 1;
            Box::pin(scan_directory(
                pool,
                backend_name,
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
                backend_name,
                &path,
                root_dir,
                scan_start_time,
                progress,
            )
            .await?;
        }

        progress.log_progress_if_needed(backend_name);
    }

    Ok(())
}

async fn scan_file(
    pool: &DatabaseConnection,
    backend_name: &str,
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
            backend_name: Set(backend_name.to_string()),
            key: Set(relative_path),
            size_bytes: Set(Some(size_bytes_i64)),
            scanned_at: Set(scan_start_time),
            ..Default::default()
        };
        // file.insert(pool).await?;
        file::Entity::insert(file)
            .on_conflict(
                OnConflict::columns([file::Column::BackendName, file::Column::Key])
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
    backend_name: &str,
    scan_start_time: i64,
) -> anyhow::Result<()> {
    file::Entity::update_many()
        .set(file::ActiveModel {
            unavailable_since: Set(Some(scan_start_time)),
            ..Default::default()
        })
        .filter(file::Column::BackendName.eq(backend_name))
        .filter(file::Column::ScannedAt.lt(scan_start_time))
        .filter(file::Column::UnavailableSince.is_null())
        .exec(pool)
        .await?;

    Ok(())
}
