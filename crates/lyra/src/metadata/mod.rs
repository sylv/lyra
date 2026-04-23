use lyra_metadata::MetadataProvider;
use lyra_metadata_tmdb::TmdbMetadataProvider;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

mod job_root_sync;
mod local;
mod read;
mod remote;
mod store;
mod sync;

pub(crate) use local::{
    LocalMetadataPlan, NodeLocalMetadataInput, replace_local_metadata_for_root,
    upsert_node_local_metadata_input,
};
pub(crate) use read::join_preferred_node_metadata;
pub(crate) use sync::mark_root_dirty;

pub(crate) const METADATA_RETRY_BACKOFF_SECONDS: &[i64] = &[
    48 * 60 * 60,
    7 * 24 * 60 * 60,
    30 * 24 * 60 * 60,
    90 * 24 * 60 * 60,
];

fn build_metadata_providers() -> Vec<Arc<dyn MetadataProvider>> {
    vec![Arc::new(TmdbMetadataProvider::new())]
}

pub(crate) fn register_jobs(
    jobs: &mut Vec<crate::jobs::RegisteredJob>,
    heavy_jobs: &mut Vec<Arc<dyn crate::jobs::HeavyJobRunner>>,
    pool: &DatabaseConnection,
    wake_signal: Arc<Notify>,
    startup_scans_complete: CancellationToken,
) {
    crate::jobs::register_job(
        Arc::new(job_root_sync::NodeMetadataSyncRootJob::new(
            build_metadata_providers(),
        )),
        jobs,
        heavy_jobs,
        pool,
        wake_signal,
        startup_scans_complete,
    );
}
