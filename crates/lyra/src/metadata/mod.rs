use lyra_metadata::MetadataProvider;
use lyra_metadata_tmdb::TmdbMetadataProvider;
use std::sync::Arc;

pub mod job_item_batch;
pub mod job_root;
mod store;

pub(crate) const METADATA_RETRY_BACKOFF_SECONDS: &[i64] = &[
    48 * 60 * 60,
    7 * 24 * 60 * 60,
    30 * 24 * 60 * 60,
    90 * 24 * 60 * 60,
];

pub fn build_metadata_providers() -> Vec<Arc<dyn MetadataProvider>> {
    vec![Arc::new(TmdbMetadataProvider::new())]
}
