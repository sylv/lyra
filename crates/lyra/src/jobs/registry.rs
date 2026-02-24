use crate::jobs::{
    JobHandler, JobManager, JobRunner,
    handlers::{file_thumbnail::FileThumbnailJob, file_timeline_preview::FileTimelinePreviewJob},
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Notify;

pub fn get_registered_job_handlers() -> Vec<Arc<dyn JobHandler>> {
    vec![Arc::new(FileTimelinePreviewJob), Arc::new(FileThumbnailJob)]
}

pub fn get_registered_jobs(
    pool: &DatabaseConnection,
    wake_signal: Arc<Notify>,
) -> Vec<Box<dyn JobRunner>> {
    get_registered_job_handlers()
        .into_iter()
        .map(|handler| {
            Box::new(JobManager::new(handler, pool.clone(), wake_signal.clone()))
                as Box<dyn JobRunner>
        })
        .collect()
}
