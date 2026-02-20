use std::{path::PathBuf, time::Duration};

mod extract;
mod generate;

pub const SCAN_RATIO: f64 = 0.2;
pub const MIN_SCAN_SECONDS: f64 = 120.0;
pub const MAX_SCAN_SECONDS: f64 = 600.0;
pub const SNAPSHOT_INTERVAL_SECONDS: f64 = 15.0;
pub const MAX_DIMENSION_PX: u32 = 1200;
pub const WEBP_QUALITY: f32 = 72.0;
pub const MAX_EXTRACTION_RUNTIME_SECONDS: u64 = 90;

#[derive(Clone, Debug)]
pub struct ThumbnailOptions {
    pub ffmpeg_bin: PathBuf,
    pub ffprobe_bin: PathBuf,
    pub working_dir: PathBuf,
    pub max_dimension_px: u32,
    pub webp_quality: f32,
}

impl Default for ThumbnailOptions {
    fn default() -> Self {
        Self {
            ffmpeg_bin: PathBuf::from("ffmpeg"),
            ffprobe_bin: PathBuf::from("ffprobe"),
            working_dir: std::env::temp_dir().join("lyra-thumbnail"),
            max_dimension_px: MAX_DIMENSION_PX,
            webp_quality: WEBP_QUALITY,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Thumbnail {
    pub image_bytes: Vec<u8>,
    pub source_width: u32,
    pub source_height: u32,
    pub scan_window: Duration,
}

pub async fn generate_thumbnail(
    video_path: &PathBuf,
    options: &ThumbnailOptions,
) -> anyhow::Result<Thumbnail> {
    let scan_window = extract::resolve_scan_window(video_path, options).await?;
    let frame_paths = extract::extract_snapshots(video_path, options, scan_window).await?;
    generate::select_and_encode_best(&frame_paths, options, scan_window).await
}
