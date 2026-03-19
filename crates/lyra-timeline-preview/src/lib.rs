use std::{path::PathBuf, time::Duration};
use tokio_util::sync::CancellationToken;

mod extract;
mod generate;

pub const FRAME_INTERVAL_SECONDS: f64 = 6.0;
pub const GAP_PX: u32 = 1;
pub const THUMBNAIL_WIDTH_PX: u32 = 280;
pub const OUTPUT_FILE_NAME: &str = "generated_preview.webp";
pub const WEBP_QUALITY: f32 = 48.0;
pub const MAX_UNCOMPRESSED_SIZE_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_FRAMES_PER_SHEET: usize = 256;

#[derive(Clone, Debug)]
pub struct PreviewOptions {
    pub ffmpeg_bin: PathBuf,
    pub frame_interval_seconds: f64,
    pub width_px: u32,
    pub working_dir: PathBuf,
}

impl Default for PreviewOptions {
    fn default() -> Self {
        Self {
            ffmpeg_bin: PathBuf::from("ffmpeg"),
            frame_interval_seconds: FRAME_INTERVAL_SECONDS,
            width_px: THUMBNAIL_WIDTH_PX,
            working_dir: std::env::temp_dir().join("lyra-timeline-preview"),
        }
    }
}

#[derive(Clone)]
pub struct TimelinePreview {
    pub preview_bytes: Vec<u8>,
    pub start_time: Duration,
    pub end_time: Duration,
    pub frame_interval: Duration,
    pub width_px: u32,
}

pub async fn generate_previews(
    video_path: &PathBuf,
    options: &PreviewOptions,
    cancellation_token: Option<&CancellationToken>,
) -> anyhow::Result<Option<Vec<TimelinePreview>>> {
    let owned_cancellation_token;
    let cancellation_token = match cancellation_token {
        Some(cancellation_token) => cancellation_token,
        None => {
            owned_cancellation_token = CancellationToken::new();
            &owned_cancellation_token
        }
    };
    let Some(frame_paths) =
        extract::extract_frame_paths(video_path, options, cancellation_token).await?
    else {
        return Ok(None);
    };

    Ok(Some(
        generate::generate_sheets(&frame_paths, options).await?,
    ))
}
