use crate::jobs::{
    JobHandler, JobManager,
    handlers::{file_thumbnail::FileThumbnailJob, file_timeline_preview::FileTimelinePreviewJob},
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Notify;

pub fn get_registered_job_handlers() -> Vec<Arc<dyn JobHandler>> {
    vec![Arc::new(FileTimelinePreviewJob), Arc::new(FileThumbnailJob)]
}

pub fn get_registered_jobs(pool: &DatabaseConnection, wake_signal: Arc<Notify>) -> Vec<JobManager> {
    get_registered_job_handlers()
        .into_iter()
        .map(|handler| JobManager::new(handler, pool.clone(), wake_signal.clone()))
        .collect()
}
