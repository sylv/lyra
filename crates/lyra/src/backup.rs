use crate::config::get_config;
use anyhow::Context;
use chrono::{Local, TimeZone};
use sqlx::{SqlitePool, migrate::Migrate};
use std::borrow::Cow;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tokio::fs;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, info, warn};

const DAILY_RETENTION_MIN: u32 = 1;
const ONE_DAY: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Clone)]
pub enum BackupReason {
    Daily,
    BeforeMigration { migration_name: String },
}

impl BackupReason {
    fn describe(&self) -> Cow<'_, str> {
        match self {
            Self::Daily => Cow::Borrowed("daily"),
            Self::BeforeMigration { migration_name } => {
                Cow::Owned(format!("before migration {migration_name}"))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackupFileKind {
    Daily,
    Persistent,
}

pub struct BackupManager {
    pool: SqlitePool,
    lock: Mutex<()>,
}

impl BackupManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            lock: Mutex::new(()),
        }
    }

    pub fn is_enabled(&self) -> bool {
        get_config().backup.enabled
    }

    pub async fn backup_now(&self, reason: BackupReason) -> Result<PathBuf, BackupError> {
        if !self.is_enabled() {
            return Err(BackupError::Disabled);
        }

        let _guard = self.lock.lock().await;
        let now = Local::now();
        let (filename, kind) = self.filename_for_reason(&reason, now)?;
        let backup_dir = get_config().backup.get_directory(&get_config().data_dir);
        let destination = backup_dir.join(filename);
        info!(
            reason = %reason.describe(),
            path = %destination.display(),
            "starting sqlite backup"
        );

        fs::create_dir_all(&backup_dir).await?;

        if let Ok(metadata) = fs::metadata(&destination).await {
            if metadata.is_file() {
                fs::remove_file(&destination).await?;
            }
        }

        self.run_backup_process(&destination).await?;

        if kind == BackupFileKind::Daily {
            self.apply_retention().await?;
        }

        info!(
            reason = %reason.describe(),
            path = %destination.display(),
            "backup complete"
        );

        Ok(destination)
    }

    fn filename_for_reason(
        &self,
        reason: &BackupReason,
        now: chrono::DateTime<Local>,
    ) -> Result<(String, BackupFileKind), BackupError> {
        match reason {
            BackupReason::Daily => {
                let ts = now.format("%Y-%m-%d_%H-%M-%S");
                Ok((
                    format!("daily-{}-{}.db.zst", ts, now.timestamp()),
                    BackupFileKind::Daily,
                ))
            }
            BackupReason::BeforeMigration { migration_name } => {
                let sanitized = migration_name.replace(' ', "_");
                Ok((
                    format!("before-{}.db.zst", sanitized),
                    BackupFileKind::Persistent,
                ))
            }
        }
    }

    async fn run_backup_process(&self, destination: &Path) -> Result<(), BackupError> {
        let mut temp_destination = destination.to_path_buf();
        temp_destination.set_extension("sqlite.tmp");

        if let Err(err) = fs::remove_file(&temp_destination).await {
            if err.kind() != ErrorKind::NotFound {
                return Err(err.into());
            }
        }

        let temp_path = temp_destination.to_string_lossy().into_owned();

        let compress_result = async {
            let mut connection = self.pool.acquire().await?;
            sqlx::query("VACUUM INTO ?")
                .bind(&temp_path)
                .execute(&mut *connection)
                .await?;

            let data = fs::read(&temp_destination).await?;
            let compressed = zstd::encode_all(data.as_slice(), 13)?;
            fs::write(destination, compressed).await?;
            Ok::<(), BackupError>(())
        }
        .await;

        if let Err(err) = fs::remove_file(&temp_destination).await {
            if err.kind() != ErrorKind::NotFound {
                warn!(
                    error = %err,
                    file = %temp_destination.display(),
                    "failed to delete temporary sqlite backup"
                );
            }
        }

        compress_result
    }

    async fn apply_retention(&self) -> Result<(), BackupError> {
        let config = get_config();
        let backup_dir = config.backup.get_directory(&config.data_dir);
        let mut entries = match fs::read_dir(&backup_dir).await {
            Ok(entries) => entries,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(()),
            Err(err) => return Err(err.into()),
        };
        let mut daily_files = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };

            if !name.starts_with("daily-") {
                continue;
            }

            if let Some(timestamp) = self.parse_daily_timestamp(name, &path).await? {
                daily_files.push((timestamp, path));
            }
        }

        daily_files.sort_by_key(|(timestamp, _)| *timestamp);
        let retention_target = config.backup.retention_days.max(DAILY_RETENTION_MIN) as usize;

        while daily_files.len() > retention_target {
            if let Some((_, path)) = daily_files.first() {
                if let Err(err) = fs::remove_file(path).await {
                    warn!(
                        error = %err,
                        file = %path.display(),
                        "failed to delete old backup"
                    );
                    break;
                }
            }
            daily_files.remove(0);
        }

        Ok(())
    }

    async fn parse_daily_timestamp(
        &self,
        name: &str,
        path: &Path,
    ) -> Result<Option<u64>, BackupError> {
        if let Some(timestamp) = timestamp_from_filename(name) {
            return Ok(Some(timestamp));
        }

        match fs::metadata(path).await {
            Ok(metadata) => {
                if let Ok(created) = metadata.created() {
                    return Ok(Some(
                        created
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or(Duration::ZERO)
                            .as_secs(),
                    ));
                }

                if let Ok(modified) = metadata.modified() {
                    return Ok(Some(
                        modified
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or(Duration::ZERO)
                            .as_secs(),
                    ));
                }

                Ok(None)
            }
            Err(err) => {
                warn!(
                    error = %err,
                    file = %path.display(),
                    "failed to read metadata for backup"
                );
                Ok(None)
            }
        }
    }

    pub async fn latest_daily_backup(&self) -> Result<Option<SystemTime>, BackupError> {
        let config = get_config();
        let backup_dir = config.backup.get_directory(&config.data_dir);
        let mut entries = match fs::read_dir(&backup_dir).await {
            Ok(entries) => entries,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(err.into()),
        };
        let mut newest = None;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };

            if !name.starts_with("daily-") {
                continue;
            }

            let Some(epoch_secs) = self.parse_daily_timestamp(name, &path).await? else {
                continue;
            };

            let candidate = SystemTime::UNIX_EPOCH + Duration::from_secs(epoch_secs);

            if newest.map(|current| candidate > current).unwrap_or(true) {
                newest = Some(candidate);
            }
        }

        Ok(newest)
    }

    pub async fn ensure_recent_daily_backup(&self) -> Result<(), BackupError> {
        if self.is_daily_backup_stale().await? {
            self.backup_now(BackupReason::Daily).await?;
        }

        Ok(())
    }

    async fn is_daily_backup_stale(&self) -> Result<bool, BackupError> {
        let last = self.latest_daily_backup().await?;
        let now = SystemTime::now();

        Ok(match last {
            None => true,
            Some(timestamp) => now.duration_since(timestamp).unwrap_or(Duration::ZERO) >= ONE_DAY,
        })
    }

    pub fn duration_until_next_midnight(&self) -> Duration {
        let now = Local::now();
        let today = now.date_naive();
        let Some(tomorrow) = today.succ_opt() else {
            return ONE_DAY;
        };
        let midnight_naive = tomorrow.and_hms_opt(0, 0, 0).unwrap();
        let midnight = Local
            .from_local_datetime(&midnight_naive)
            .single()
            .unwrap_or(now + chrono::Duration::days(1));

        (midnight - now)
            .to_std()
            .unwrap_or_else(|_| Duration::from_secs(60))
    }
}

pub async fn run_backup_worker(manager: Arc<BackupManager>) -> anyhow::Result<()> {
    if !manager.is_enabled() {
        info!("database backups are disabled");
        return Ok(());
    }

    if let Err(err) = manager.ensure_recent_daily_backup().await {
        error!(error = %err, "failed to create startup daily backup");
    }

    loop {
        sleep(manager.duration_until_next_midnight()).await;

        match manager.is_daily_backup_stale().await {
            Ok(true) => {
                if let Err(err) = manager.backup_now(BackupReason::Daily).await {
                    error!(error = %err, "daily backup failed");
                }
            }
            Ok(false) => {
                info!("skipping daily backup; last run still fresh");
            }
            Err(err) => {
                error!(error = %err, "failed to evaluate daily backup freshness");
            }
        }
    }
}

pub async fn run_migrations_with_backups(
    pool: &SqlitePool,
    backup_manager: &BackupManager,
) -> anyhow::Result<()> {
    let migrator = sqlx::migrate!("../../migrations");

    let mut connection = pool.acquire().await?;
    connection.ensure_migrations_table().await?;
    let applied = connection.list_applied_migrations().await?;
    drop(connection);

    let applied_versions: std::collections::HashSet<_> = applied
        .into_iter()
        .map(|migration| migration.version)
        .collect();

    // Back up once before the first pending migration so the backup matches the last known-good
    // schema instead of producing a file per migration step.
    let mut pending_migrations: Vec<_> = migrator
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .filter(|migration| !applied_versions.contains(&migration.version))
        .collect();
    pending_migrations.sort_by_key(|migration| migration.version);

    if backup_manager.is_enabled() {
        if let Some(first_migration) = pending_migrations.first() {
            let reason = BackupReason::BeforeMigration {
                migration_name: first_migration.description.to_string(),
            };

            backup_manager.backup_now(reason).await.with_context(|| {
                format!(
                    "failed to create backup before migration {}",
                    first_migration.description
                )
            })?;
        }
    }

    migrator.run(pool).await?;
    Ok(())
}

fn timestamp_from_filename(name: &str) -> Option<u64> {
    if !name.starts_with("daily-") || !name.ends_with(".db.zst") {
        return None;
    }

    let trimmed = name.strip_prefix("daily-")?.strip_suffix(".db.zst")?;
    let unix_part = trimmed.rsplit_once('-')?.1;
    unix_part
        .parse::<i64>()
        .ok()
        .and_then(|timestamp| (timestamp >= 0).then_some(timestamp as u64))
}

#[derive(Debug, Error)]
pub enum BackupError {
    #[error("backups are disabled by configuration")]
    Disabled,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),
}
