use anyhow::{Result, bail};
use std::sync::OnceLock;

static FFMPEG_CONFIG: OnceLock<Result<FfmpegConfig, String>> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct FfmpegConfig {
    pub ffmpeg_path: String,
    pub ffprobe_path: String,
}

pub fn get_paths() -> FfmpegConfig {
    let result = FFMPEG_CONFIG
        .get_or_init(|| init_ffmpeg_config().map_err(|err| err.to_string()))
        .as_ref()
        .expect("failed to init ffmpeg config");
    result.clone()
}

pub fn get_ffmpeg_path() -> String {
    get_paths().ffmpeg_path
}

pub fn get_ffprobe_path() -> String {
    get_paths().ffprobe_path
}

pub fn init_ffmpeg() -> Result<()> {
    let _ = get_paths();
    Ok(())
}

fn init_ffmpeg_config() -> Result<FfmpegConfig> {
    let ffmpeg_path = std::env::var("LYRA_FFMPEG_PATH");
    let ffprobe_path = std::env::var("LYRA_FFPROBE_PATH");

    if let (Ok(ffmpeg_path), Ok(ffprobe_path)) = (ffmpeg_path, ffprobe_path) {
        tracing::info!(
            ffmpeg = %ffmpeg_path,
            ffprobe = %ffprobe_path,
            "configured ffmpeg/ffprobe binaries from environment variables"
        );

        return Ok(FfmpegConfig {
            ffmpeg_path,
            ffprobe_path,
        });
    }

    let try_paths = vec![
        format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "../../bin"),
        "bin".to_string(),
        "/usr/local/bin".to_string(),
        "/usr/bin".to_string(),
    ];

    for base_path in try_paths {
        let ffmpeg_path = std::path::Path::new(&base_path).join("lyra-ffmpeg");
        let ffprobe_path = std::path::Path::new(&base_path).join("lyra-ffprobe");
        tracing::debug!(
            "checking for ffmpeg at {} and ffprobe at {}",
            ffmpeg_path.display(),
            ffprobe_path.display()
        );

        if ffmpeg_path.exists() && ffprobe_path.exists() {
            tracing::info!(
                ffmpeg = %ffmpeg_path.display(),
                ffprobe = %ffprobe_path.display(),
                "configured ffmpeg/ffprobe binaries from candidate paths"
            );

            return Ok(FfmpegConfig {
                ffmpeg_path: ffmpeg_path.to_string_lossy().to_string(),
                ffprobe_path: ffprobe_path.to_string_lossy().to_string(),
            });
        }
    }

    bail!(
        "could not find lyra-ffmpeg/lyra-ffprobe binaries, maybe try setting LYRA_FFMPEG_PATH and LYRA_FFPROBE_PATH"
    )
}
