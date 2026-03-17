use std::path::PathBuf;

pub const TARGET_SEGMENT_SECONDS: f64 = 6.0;

#[derive(Clone, Debug)]
pub struct BuildOptions {
    pub transcode_cache_dir: PathBuf,
}
