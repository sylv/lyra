use crate::jobs::semaphore::JobSemaphore;
use crate::jobs::{
    JobManager,
    handlers::{
        asset_download::AssetDownloadJob, asset_thumbhash::AssetThumbhashJob,
        file_ffprobe::FileFfprobeJob, file_keyframes::FileKeyframesJob,
        file_thumbnail::FileThumbnailJob, file_timeline_preview::FileTimelinePreviewJob,
        root_intro_segments::RootIntroSegmentsJob,
    },
};
use crate::metadata::{build_metadata_providers, job_root_sync::NodeMetadataSyncRootJob};
use sea_orm::DatabaseConnection;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Notify;

pub struct RegisteredJob {
    pub job_kind: crate::entities::jobs::JobKind,
    pub task: Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>>,
}

pub fn load_registered_jobs(
    pool: &DatabaseConnection,
    wake_signal: Arc<Notify>,
    job_semaphore: Arc<JobSemaphore>,
) -> Vec<RegisteredJob> {
    let metadata_providers = build_metadata_providers();

    vec![
        build_registered_job(
            Arc::new(AssetDownloadJob),
            pool,
            wake_signal.clone(),
            job_semaphore.clone(),
        ),
        build_registered_job(
            Arc::new(AssetThumbhashJob),
            pool,
            wake_signal.clone(),
            job_semaphore.clone(),
        ),
        build_registered_job(
            Arc::new(FileFfprobeJob),
            pool,
            wake_signal.clone(),
            job_semaphore.clone(),
        ),
        build_registered_job(
            Arc::new(FileKeyframesJob),
            pool,
            wake_signal.clone(),
            job_semaphore.clone(),
        ),
        build_registered_job(
            Arc::new(FileTimelinePreviewJob),
            pool,
            wake_signal.clone(),
            job_semaphore.clone(),
        ),
        build_registered_job(
            Arc::new(FileThumbnailJob),
            pool,
            wake_signal.clone(),
            job_semaphore.clone(),
        ),
        build_registered_job(
            Arc::new(RootIntroSegmentsJob),
            pool,
            wake_signal.clone(),
            job_semaphore.clone(),
        ),
        build_registered_job(
            Arc::new(NodeMetadataSyncRootJob::new(metadata_providers)),
            pool,
            wake_signal,
            job_semaphore,
        ),
    ]
}

fn build_registered_job<J: crate::jobs::Job>(
    job: Arc<J>,
    pool: &DatabaseConnection,
    wake_signal: Arc<Notify>,
    job_semaphore: Arc<JobSemaphore>,
) -> RegisteredJob {
    let manager = JobManager::new(job.clone(), pool.clone(), wake_signal, job_semaphore);

    RegisteredJob {
        job_kind: manager.job_kind(),
        task: Box::pin(async move { manager.start_thread().await }),
    }
}
