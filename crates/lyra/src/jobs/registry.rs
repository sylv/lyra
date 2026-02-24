use crate::jobs::{
    JobHandler, JobManager,
    handlers::{
        file_ffprobe::FileFfprobeJob, file_keyframes::FileKeyframesJob,
        file_thumbnail::FileThumbnailJob, file_timeline_preview::FileTimelinePreviewJob,
    },
};
use crate::reactivity::SharedSyncVersion;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Notify;

pub fn get_registered_job_handlers() -> Vec<Arc<dyn JobHandler>> {
    vec![
        Arc::new(FileFfprobeJob),
        Arc::new(FileKeyframesJob),
        Arc::new(FileTimelinePreviewJob),
        Arc::new(FileThumbnailJob),
    ]
}

pub fn get_registered_jobs(
    pool: &DatabaseConnection,
    wake_signal: Arc<Notify>,
    sync_version: SharedSyncVersion,
) -> Vec<JobManager> {
    get_registered_job_handlers()
        .into_iter()
        .map(|handler| {
            JobManager::new(
                handler,
                pool.clone(),
                wake_signal.clone(),
                sync_version.clone(),
            )
        })
        .collect()
}
