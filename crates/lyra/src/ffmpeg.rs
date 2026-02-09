use anyhow::{Result, bail};
use std::{path::PathBuf, sync::OnceLock};

static FFMPEG_CONFIG: OnceLock<FfmpegConfig> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct FfmpegConfig {
    pub ffmpeg_path: String,
    pub ffprobe_path: String,
}

pub fn get_ffmpeg_path() -> String {
    FFMPEG_CONFIG
        .get()
        .expect("ffmpeg not initialized - call ensure_ffmpeg() first")
        .ffmpeg_path
        .clone()
}

pub fn get_ffprobe_path() -> String {
    FFMPEG_CONFIG
        .get()
        .expect("ffprobe not initialized - call ensure_ffmpeg() first")
        .ffprobe_path
        .clone()
}

/// Initializes ffmpeg/ffprobe paths from static local binaries.
///
/// Expected files:
/// - ./bin/ffmpeg
///
/// ffprobe is expected to come from the host PATH.
pub async fn ensure_ffmpeg() -> Result<()> {
    let ffmpeg_path = PathBuf::from("./bin/ffmpeg");
    let ffprobe_path = "ffprobe".to_string();

    if !ffmpeg_path.exists() {
        bail!(
            "ffmpeg binary not found at {}",
            ffmpeg_path.as_path().display()
        );
    }

    let ffmpeg_path = std::fs::canonicalize(&ffmpeg_path)?;

    FFMPEG_CONFIG.get_or_init(|| FfmpegConfig {
        ffmpeg_path: ffmpeg_path.to_string_lossy().to_string(),
        ffprobe_path: ffprobe_path.clone(),
    });

    tracing::info!(
        ffmpeg = %ffmpeg_path.display(),
        ffprobe = %ffprobe_path,
        "using static local ffmpeg with host ffprobe"
    );

    Ok(())
}
