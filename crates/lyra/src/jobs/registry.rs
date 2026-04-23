use crate::jobs::job::JobScheduling;
use crate::jobs::semaphore::HeavyJobController;
use crate::jobs::{
    HeavyJobRunner, HeavyJobScheduler, LightJobWorker, manager::GenericHeavyJobRunner,
};
use crate::{assets, media, metadata, segment_markers, subtitles};
use sea_orm::DatabaseConnection;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

pub(crate) struct RegisteredJob {
    pub(crate) job_kind: Option<crate::entities::jobs::JobKind>,
    pub(crate) task: Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>>,
}

pub(crate) fn load_registered_jobs(
    pool: &DatabaseConnection,
    wake_signal: Arc<Notify>,
    heavy_job_controller: Arc<HeavyJobController>,
    startup_scans_complete: CancellationToken,
) -> Vec<RegisteredJob> {
    let mut heavy_jobs = Vec::<Arc<dyn HeavyJobRunner>>::new();
    let mut jobs = Vec::new();

    assets::register_jobs(
        &mut jobs,
        &mut heavy_jobs,
        pool,
        wake_signal.clone(),
        startup_scans_complete.clone(),
    );
    media::register_jobs(
        &mut jobs,
        &mut heavy_jobs,
        pool,
        wake_signal.clone(),
        startup_scans_complete.clone(),
    );
    subtitles::register_jobs(
        &mut jobs,
        &mut heavy_jobs,
        pool,
        wake_signal.clone(),
        startup_scans_complete.clone(),
    );
    segment_markers::register_jobs(
        &mut jobs,
        &mut heavy_jobs,
        pool,
        wake_signal.clone(),
        startup_scans_complete.clone(),
    );
    metadata::register_jobs(
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

pub(crate) fn register_job<J: crate::jobs::Job>(
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
