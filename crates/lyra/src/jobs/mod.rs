mod handlers;
mod job;
mod manager;
mod on_demand;
mod registry;
mod semaphore;

pub use handlers::{
    asset_download::AssetDownloadJob, file_ffprobe::FileFfprobeJob,
    file_keyframes::FileKeyframesJob,
};
pub use job::{Job, JobExecutionPolicy, JobOutcome};
pub use manager::JobManager;
pub(crate) use manager::delete_job_row;
pub use on_demand::try_run_job;
pub use registry::load_registered_jobs;
pub use semaphore::{JobLease, JobSemaphore};
