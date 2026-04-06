mod keyframes;
mod paths;
mod probe;
mod types;

pub const PROBE_ZSTD_DICTIONARY: &[u8] = include_bytes!("../dictionary");

pub use keyframes::extract_keyframes;
pub use paths::{get_ffmpeg_path, get_ffprobe_path, get_paths, init_ffmpeg};
pub use probe::{
    decode_probe_data_json_zstd, encode_probe_data_json_zstd, probe, probe_blocking,
    probe_with_cancellation,
};
pub use types::*;
