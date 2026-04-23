mod file_path;
mod job_file_probe;
mod job_file_thumbnail;
mod job_file_timeline_preview;
mod probe;

use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

pub(crate) use file_path::get_job_file_path;
pub(crate) use job_file_probe::FileProbeJob;
pub(crate) use probe::{load_cached_keyframes, load_cached_probe};

pub(crate) fn register_jobs(
    jobs: &mut Vec<crate::jobs::RegisteredJob>,
    heavy_jobs: &mut Vec<Arc<dyn crate::jobs::HeavyJobRunner>>,
    pool: &DatabaseConnection,
    wake_signal: Arc<Notify>,
    startup_scans_complete: CancellationToken,
) {
    crate::jobs::register_job(
        Arc::new(job_file_probe::FileProbeJob),
        jobs,
        heavy_jobs,
        pool,
        wake_signal.clone(),
        startup_scans_complete.clone(),
    );
    crate::jobs::register_job(
        Arc::new(job_file_thumbnail::FileThumbnailJob),
        jobs,
        heavy_jobs,
        pool,
        wake_signal.clone(),
        startup_scans_complete.clone(),
    );
    crate::jobs::register_job(
        Arc::new(job_file_timeline_preview::FileTimelinePreviewJob),
        jobs,
        heavy_jobs,
        pool,
        wake_signal,
        startup_scans_complete,
    );
}
