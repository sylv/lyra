use crate::jobs::job::JobScheduling;
use crate::jobs::semaphore::HeavyJobController;
use crate::jobs::{
    HeavyJobRunner, HeavyJobScheduler, LightJobWorker,
    handlers::{
        asset_download::AssetDownloadJob, asset_thumbhash::AssetThumbhashJob,
        file_probe::FileProbeJob, file_thumbnail::FileThumbnailJob,
        file_timeline_preview::FileTimelinePreviewJob, root_intro_segments::RootIntroSegmentsJob,
    },
    manager::GenericHeavyJobRunner,
};
use crate::metadata::{build_metadata_providers, job_root_sync::NodeMetadataSyncRootJob};
use sea_orm::DatabaseConnection;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

pub struct RegisteredJob {
    pub job_kind: Option<crate::entities::jobs::JobKind>,
    pub task: Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>>,
}

pub fn load_registered_jobs(
    pool: &DatabaseConnection,
    wake_signal: Arc<Notify>,
    heavy_job_controller: Arc<HeavyJobController>,
    startup_scans_complete: CancellationToken,
) -> Vec<RegisteredJob> {
    let metadata_providers = build_metadata_providers();
    let mut heavy_jobs = Vec::<Arc<dyn HeavyJobRunner>>::new();
    let mut jobs = Vec::new();

    register_job(
        Arc::new(AssetDownloadJob),
        &mut jobs,
        &mut heavy_jobs,
        pool,
        wake_signal.clone(),
        startup_scans_complete.clone(),
    );
    register_job(
        Arc::new(AssetThumbhashJob),
        &mut jobs,
        &mut heavy_jobs,
        pool,
        wake_signal.clone(),
        startup_scans_complete.clone(),
    );
    register_job(
        Arc::new(FileProbeJob),
        &mut jobs,
        &mut heavy_jobs,
        pool,
        wake_signal.clone(),
        startup_scans_complete.clone(),
    );
    register_job(
        Arc::new(FileTimelinePreviewJob),
        &mut jobs,
        &mut heavy_jobs,
        pool,
        wake_signal.clone(),
        startup_scans_complete.clone(),
    );
    register_job(
        Arc::new(FileThumbnailJob),
        &mut jobs,
        &mut heavy_jobs,
        pool,
        wake_signal.clone(),
        startup_scans_complete.clone(),
    );
    register_job(
        Arc::new(RootIntroSegmentsJob),
        &mut jobs,
        &mut heavy_jobs,
        pool,
        wake_signal.clone(),
        startup_scans_complete.clone(),
    );
    register_job(
        Arc::new(NodeMetadataSyncRootJob::new(metadata_providers)),
        &mut jobs,
        &mut heavy_jobs,
        pool,
        wake_signal.clone(),
        startup_scans_complete.clone(),
    );

    let pool = pool.clone();
    jobs.push(RegisteredJob {
        job_kind: None,
        task: Box::pin(async move {
            let scheduler = HeavyJobScheduler::new(
                heavy_jobs,
                pool,
                wake_signal,
                heavy_job_controller,
                startup_scans_complete,
            );
            scheduler.start_thread().await
        }),
    });

    jobs
}

fn register_job<J: crate::jobs::Job>(
    job: Arc<J>,
    jobs: &mut Vec<RegisteredJob>,
    heavy_jobs: &mut Vec<Arc<dyn HeavyJobRunner>>,
    pool: &DatabaseConnection,
    wake_signal: Arc<Notify>,
    startup_scans_complete: CancellationToken,
) {
    match J::SCHEDULING {
        JobScheduling::Light => {
            let worker = LightJobWorker::new(
                job.clone(),
                pool.clone(),
                wake_signal,
                startup_scans_complete,
            );
            jobs.push(RegisteredJob {
                job_kind: Some(worker.job_kind()),
                task: Box::pin(async move { worker.start_thread().await }),
            });
        }
        JobScheduling::Heavy(priority) => {
            heavy_jobs.push(Arc::new(GenericHeavyJobRunner::new(job, priority)));
        }
    }
}
