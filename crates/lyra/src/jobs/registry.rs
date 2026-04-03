use crate::jobs::semaphore::JobSemaphore;
use crate::jobs::{
    JobManager,
    handlers::{
        asset_download::AssetDownloadJob, asset_thumbhash::AssetThumbhashJob,
        file_probe::FileProbeJob, file_thumbnail::FileThumbnailJob,
        file_timeline_preview::FileTimelinePreviewJob, root_intro_segments::RootIntroSegmentsJob,
    },
};
use crate::metadata::{build_metadata_providers, job_root_sync::NodeMetadataSyncRootJob};
use sea_orm::DatabaseConnection;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

pub struct RegisteredJob {
    pub job_kind: crate::entities::jobs::JobKind,
    pub task: Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>>,
}

pub fn load_registered_jobs(
    pool: &DatabaseConnection,
    wake_signal: Arc<Notify>,
    job_semaphore: Arc<JobSemaphore>,
    startup_scans_complete: CancellationToken,
) -> Vec<RegisteredJob> {
    let metadata_providers = build_metadata_providers();

    vec![
        build_registered_job(
            Arc::new(AssetDownloadJob),
            pool,
            wake_signal.clone(),
            job_semaphore.clone(),
            startup_scans_complete.clone(),
        ),
        build_registered_job(
            Arc::new(AssetThumbhashJob),
            pool,
            wake_signal.clone(),
            job_semaphore.clone(),
            startup_scans_complete.clone(),
        ),
        build_registered_job(
            Arc::new(FileProbeJob),
            pool,
            wake_signal.clone(),
            job_semaphore.clone(),
            startup_scans_complete.clone(),
        ),
        build_registered_job(
            Arc::new(FileTimelinePreviewJob),
            pool,
            wake_signal.clone(),
            job_semaphore.clone(),
            startup_scans_complete.clone(),
        ),
        build_registered_job(
            Arc::new(FileThumbnailJob),
            pool,
            wake_signal.clone(),
            job_semaphore.clone(),
            startup_scans_complete.clone(),
        ),
        build_registered_job(
            Arc::new(RootIntroSegmentsJob),
            pool,
            wake_signal.clone(),
            job_semaphore.clone(),
            startup_scans_complete.clone(),
        ),
        build_registered_job(
            Arc::new(NodeMetadataSyncRootJob::new(metadata_providers)),
            pool,
            wake_signal,
            job_semaphore,
            startup_scans_complete,
        ),
    ]
}

fn build_registered_job<J: crate::jobs::Job>(
    job: Arc<J>,
    pool: &DatabaseConnection,
    wake_signal: Arc<Notify>,
    job_semaphore: Arc<JobSemaphore>,
    startup_scans_complete: CancellationToken,
) -> RegisteredJob {
    let manager = JobManager::new(
        job.clone(),
        pool.clone(),
        wake_signal,
        job_semaphore,
        startup_scans_complete,
    );

    RegisteredJob {
        job_kind: manager.job_kind(),
        task: Box::pin(async move { manager.start_thread().await }),
    }
}
