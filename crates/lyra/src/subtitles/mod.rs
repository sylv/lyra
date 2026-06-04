mod files;
pub(crate) mod job_extract;
pub mod language;

use lyra_probe::StreamDisposition;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

pub fn subtitle_path(subtitle_id: &str) -> std::path::PathBuf {
    crate::config::get_config()
        .get_subtitle_dir()
        .join(format!("{subtitle_id}.vtt.zst"))
}

pub fn sign_subtitle_url(subtitle_id: &str) -> String {
    format!("/api/subtitles/{subtitle_id}")
}

pub fn subtitle_disposition_bits(disposition: StreamDisposition) -> i64 {
    i64::from(disposition.bits())
}

pub(crate) fn register_jobs(
    jobs: &mut Vec<crate::jobs::RegisteredJob>,
    heavy_jobs: &mut Vec<Arc<dyn crate::jobs::HeavyJobRunner>>,
    pool: &DatabaseConnection,
    wake_signal: Arc<Notify>,
    startup_scans_complete: CancellationToken,
) {
    crate::jobs::register_job(
        Arc::new(job_extract::FileSubtitleExtractJob),
        jobs,
        heavy_jobs,
        pool,
        wake_signal.clone(),
        startup_scans_complete.clone(),
    );
}
