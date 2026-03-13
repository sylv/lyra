use crate::jobs::{
    JobActivityRegistry, JobHandler, JobManager,
    handlers::{
        asset_download::AssetDownloadJob, asset_thumbhash::AssetThumbhashJob,
        file_ffprobe::FileFfprobeJob, file_keyframes::FileKeyframesJob,
        file_thumbnail::FileThumbnailJob, file_timeline_preview::FileTimelinePreviewJob,
        root_intro_segments::RootIntroSegmentsJob,
    },
};
use crate::metadata::{
    build_metadata_providers, job_item_batch::RootMetadataMatchGroupsJob,
    job_root::RootMetadataMatchRootJob,
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Notify;

pub struct RegisteredJobs {
    pub managers: Vec<JobManager>,
    pub activity_registry: JobActivityRegistry,
}

pub fn get_registered_job_handlers() -> Vec<Arc<dyn JobHandler>> {
    vec![
        Arc::new(AssetDownloadJob),
        Arc::new(AssetThumbhashJob),
        Arc::new(FileFfprobeJob),
        Arc::new(FileKeyframesJob),
        Arc::new(FileTimelinePreviewJob),
        Arc::new(FileThumbnailJob),
        Arc::new(RootIntroSegmentsJob),
        Arc::new(RootMetadataMatchRootJob::new(build_metadata_providers())),
        Arc::new(RootMetadataMatchGroupsJob::new(build_metadata_providers())),
    ]
}

pub fn get_registered_jobs(pool: &DatabaseConnection, wake_signal: Arc<Notify>) -> RegisteredJobs {
    let handlers = get_registered_job_handlers();
    let now = chrono::Utc::now().timestamp();
    let activity_registry =
        JobActivityRegistry::new(handlers.iter().map(|handler| handler.job_kind()), now);

    let managers = handlers
        .into_iter()
        .map(|handler| {
            let job_kind = handler.job_kind();
            let activity_state = activity_registry
                .state(job_kind)
                .expect("missing activity state for registered job kind");
            JobManager::new(handler, pool.clone(), wake_signal.clone(), activity_state)
        })
        .collect();

    RegisteredJobs {
        managers,
        activity_registry,
    }
}
