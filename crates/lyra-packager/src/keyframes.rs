use anyhow::{Context, Result};
use lyra_ffprobe::probe_keyframes_pts;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::{info, warn};

use crate::binaries::configured_ffprobe_bin;

#[derive(Deserialize, Serialize)]
struct KeyframeCache {
    file_size: u64,
    keyframes: Vec<i64>,
}

#[derive(Deserialize)]
struct LegacyKeyframeCache {
    file_size: u64,
    keyframes: Vec<f64>,
}

pub fn load_or_probe_keyframes(input: &Path) -> Result<Vec<i64>> {
    let file_size = fs::metadata(input)
        .with_context(|| format!("failed to stat {}", input.display()))?
        .len();
    let cache_path = cache_path_for(input);

    if let Some(keyframes) = try_load_cache(&cache_path, file_size)? {
        if keyframes.is_empty() {
            warn!(
                cache_path = %cache_path.display(),
                "cached keyframes are empty; probing with ffprobe"
            );
        } else {
            info!(
                keyframes = keyframes.len(),
                cache_path = %cache_path.display(),
                "using cached keyframes"
            );
            return Ok(keyframes);
        }
    }

    let keyframes = probe_keyframes(input)?;
    write_cache(&cache_path, file_size, &keyframes)?;
    Ok(keyframes)
}

pub fn load_cached_keyframes(input: &Path) -> Result<Option<Vec<i64>>> {
    let file_size = fs::metadata(input)
        .with_context(|| format!("failed to stat {}", input.display()))?
        .len();
    let cache_path = cache_path_for(input);

    let Some(keyframes) = try_load_cache(&cache_path, file_size)? else {
        return Ok(None);
    };

    if keyframes.is_empty() {
        warn!(
            cache_path = %cache_path.display(),
            "cached keyframes are empty"
        );
        return Ok(None);
    }

    info!(
        keyframes = keyframes.len(),
        cache_path = %cache_path.display(),
        "using cached keyframes"
    );
    Ok(Some(keyframes))
}

fn cache_path_for(input: &Path) -> PathBuf {
    let path = input.to_string_lossy();
    PathBuf::from(format!("{path}-keyframes.json"))
}

fn try_load_cache(cache_path: &Path, file_size: u64) -> Result<Option<Vec<i64>>> {
    let data = match fs::read(cache_path) {
        Ok(data) => data,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(err).with_context(|| {
                format!("failed to read keyframe cache {}", cache_path.display())
            });
        }
    };

    let cache: KeyframeCache = match serde_json::from_slice(&data) {
        Ok(cache) => cache,
        Err(_) => {
            let legacy: LegacyKeyframeCache = serde_json::from_slice(&data).with_context(|| {
                format!("failed to parse keyframe cache {}", cache_path.display())
            })?;
            KeyframeCache {
                file_size: legacy.file_size,
                keyframes: legacy
                    .keyframes
                    .into_iter()
                    .map(|value| value.round() as i64)
                    .collect(),
            }
        }
    };

    if cache.file_size != file_size {
        warn!(
            cache_path = %cache_path.display(),
            cached_size = cache.file_size,
            current_size = file_size,
            "keyframe cache size mismatch"
        );
        return Ok(None);
    }

    // if cache.keyframes.is_empty() {
    //     warn!(
    //         cache_path = %cache_path.display(),
    //         "keyframe cache is empty"
    //     );
    //     return Ok(None);
    // }

    Ok(Some(cache.keyframes))
}

fn write_cache(cache_path: &Path, file_size: u64, keyframes: &[i64]) -> Result<()> {
    let cache = KeyframeCache {
        file_size,
        keyframes: keyframes.to_vec(),
    };
    let json = serde_json::to_vec_pretty(&cache).context("failed to serialize keyframe cache")?;
    fs::write(cache_path, json)
        .with_context(|| format!("failed to write keyframe cache {}", cache_path.display()))?;
    info!(
        keyframes = keyframes.len(),
        cache_path = %cache_path.display(),
        "wrote keyframe cache"
    );
    Ok(())
}

fn probe_keyframes(input: &Path) -> Result<Vec<i64>> {
    info!("extracting keyframes");
    let ffprobe_bin = resolve_ffprobe_bin();
    let times = probe_keyframes_pts(&ffprobe_bin, input)?;
    info!(keyframes = times.len(), "keyframe extraction complete");
    Ok(times)
}

fn resolve_ffprobe_bin() -> PathBuf {
    if let Some(path) = configured_ffprobe_bin() {
        return path;
    }

    if let Ok(path) = std::env::var("LYRA_FFPROBE_BIN") {
        return PathBuf::from(path);
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let local_candidate = manifest_dir.join("bin/ffprobe");
    if local_candidate.exists() {
        return local_candidate;
    }

    let workspace_candidate = manifest_dir.join("../../bin/ffprobe");
    if workspace_candidate.exists() {
        return workspace_candidate;
    }

    PathBuf::from("ffprobe")
}
