use anyhow::{Context, Result, bail};
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

static FFMPEG_CONFIG: OnceLock<Result<FfmpegConfig, String>> = OnceLock::new();

const FFMPEG_BIN_NAME: &str = "lyra-ffmpeg";
const FFPROBE_BIN_NAME: &str = "lyra-ffprobe";
const FFMPEG_ENV_VAR: &str = "LYRA_FFMPEG_PATH";
const FFPROBE_ENV_VAR: &str = "LYRA_FFPROBE_PATH";

#[derive(Debug, Clone)]
pub struct FfmpegConfig {
    pub ffmpeg_path: String,
    pub ffprobe_path: String,
}

pub fn init_ffmpeg() -> Result<()> {
    let _ = ffmpeg_config()?;
    Ok(())
}

pub fn get_ffmpeg_path() -> Result<String> {
    Ok(ffmpeg_config()?.ffmpeg_path.clone())
}

pub fn get_ffprobe_path() -> Result<String> {
    Ok(ffmpeg_config()?.ffprobe_path.clone())
}

fn ffmpeg_config() -> Result<&'static FfmpegConfig> {
    let result = FFMPEG_CONFIG.get_or_init(|| init_ffmpeg_config().map_err(|err| err.to_string()));
    match result {
        Ok(config) => Ok(config),
        Err(error) => bail!("{error}"),
    }
}

fn init_ffmpeg_config() -> Result<FfmpegConfig> {
    let ffmpeg_path = resolve_binary_path(FFMPEG_BIN_NAME, FFMPEG_ENV_VAR);
    let ffprobe_path = resolve_binary_path(FFPROBE_BIN_NAME, FFPROBE_ENV_VAR);

    let (ffmpeg_path, ffprobe_path) = match (ffmpeg_path, ffprobe_path) {
        (Ok(ffmpeg_path), Ok(ffprobe_path)) => (ffmpeg_path, ffprobe_path),
        _ => bail!("failed to configure ffmpeg/ffprobe binaries"),
    };

    tracing::info!(
        ffmpeg = %ffmpeg_path.display(),
        ffprobe = %ffprobe_path.display(),
        "configured ffmpeg/ffprobe binaries"
    );

    Ok(FfmpegConfig {
        ffmpeg_path: ffmpeg_path.to_string_lossy().to_string(),
        ffprobe_path: ffprobe_path.to_string_lossy().to_string(),
    })
}

fn resolve_binary_path(binary_name: &str, env_var: &str) -> Result<PathBuf> {
    let candidates = build_candidates(binary_name, env_var)?;

    for candidate in &candidates {
        if candidate.is_file() {
            return std::fs::canonicalize(candidate)
                .with_context(|| format!("failed to canonicalize {}", candidate.display()));
        }
    }

    let candidate_list = candidates
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();

    tracing::error!(
        binary = binary_name,
        env_var,
        checked_paths = ?candidate_list,
        "required binary is missing"
    );

    bail!("could not find required binary '{binary_name}'");
}

fn build_candidates(binary_name: &str, env_var: &str) -> Result<Vec<PathBuf>> {
    let mut candidates = Vec::new();

    if let Ok(value) = std::env::var(env_var) {
        let value = value.trim();
        if !value.is_empty() {
            candidates.push(PathBuf::from(value));
        }
    }

    if cfg!(debug_assertions) {
        if let Some(workspace_root) = workspace_root() {
            candidates.push(workspace_root.join("bin").join(binary_name));
        }

        let current_dir = std::env::current_dir().context("failed to resolve current directory")?;
        candidates.push(current_dir.join("bin").join(binary_name));
    }

    Ok(candidates)
}

fn workspace_root() -> Option<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf)
}
