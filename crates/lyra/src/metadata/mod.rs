use lyra_metadata::MetadataProvider;
use lyra_metadata_tmdb::TmdbMetadataProvider;
use std::sync::Arc;

mod matcher;
mod shared;
mod store;
pub mod worker;

pub fn build_metadata_providers() -> Vec<Arc<dyn MetadataProvider>> {
    vec![Arc::new(TmdbMetadataProvider::new())]
}
