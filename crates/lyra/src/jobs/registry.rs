use crate::jobs::{
    JobHandler, JobManager,
    handlers::{
        asset_download::AssetDownloadJob, asset_thumbhash::AssetThumbhashJob,
        file_ffprobe::FileFfprobeJob, file_keyframes::FileKeyframesJob,
        file_thumbnail::FileThumbnailJob, file_timeline_preview::FileTimelinePreviewJob,
        root_intro_segments::RootIntroSegmentsJob,
    },
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Notify;

pub fn get_registered_job_handlers() -> Vec<Arc<dyn JobHandler>> {
    vec![
        Arc::new(AssetDownloadJob),
        Arc::new(AssetThumbhashJob),
        Arc::new(FileFfprobeJob),
        Arc::new(FileKeyframesJob),
        Arc::new(FileTimelinePreviewJob),
        Arc::new(FileThumbnailJob),
        Arc::new(RootIntroSegmentsJob),
    ]
}

pub fn get_registered_jobs(pool: &DatabaseConnection, wake_signal: Arc<Notify>) -> Vec<JobManager> {
    get_registered_job_handlers()
        .into_iter()
        .map(|handler| JobManager::new(handler, pool.clone(), wake_signal.clone()))
        .collect()
}
