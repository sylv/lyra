use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tracing::{info, warn};

#[derive(Deserialize)]
struct ProbeFrames {
    frames: Vec<ProbeFrame>,
}

#[derive(Deserialize)]
struct ProbeFrame {
    best_effort_timestamp_time: Option<String>,
    pkt_pts_time: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct KeyframeCache {
    file_size: u64,
    keyframes: Vec<f64>,
}

pub fn load_or_probe_keyframes(input: &Path) -> Result<Vec<f64>> {
    let file_size = fs::metadata(input)
        .with_context(|| format!("failed to stat {}", input.display()))?
        .len();
    let cache_path = cache_path_for(input);

    if let Some(keyframes) = try_load_cache(&cache_path, file_size)? {
        info!(
            keyframes = keyframes.len(),
            cache_path = %cache_path.display(),
            "using cached keyframes"
        );
        return Ok(keyframes);
    }

    let keyframes = probe_keyframes(input)?;
    write_cache(&cache_path, file_size, &keyframes)?;
    Ok(keyframes)
}

fn cache_path_for(input: &Path) -> PathBuf {
    let path = input.to_string_lossy();
    PathBuf::from(format!("{path}-keyframes.json"))
}

fn try_load_cache(cache_path: &Path, file_size: u64) -> Result<Option<Vec<f64>>> {
    let data = match fs::read(cache_path) {
        Ok(data) => data,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(err)
                .with_context(|| format!("failed to read keyframe cache {}", cache_path.display()));
        }
    };

    let cache: KeyframeCache = serde_json::from_slice(&data)
        .with_context(|| format!("failed to parse keyframe cache {}", cache_path.display()))?;

    if cache.file_size != file_size {
        warn!(
            cache_path = %cache_path.display(),
            cached_size = cache.file_size,
            current_size = file_size,
            "keyframe cache size mismatch"
        );
        return Ok(None);
    }

    if cache.keyframes.is_empty() {
        warn!(
            cache_path = %cache_path.display(),
            "keyframe cache is empty"
        );
        return Ok(None);
    }

    Ok(Some(cache.keyframes))
}

fn write_cache(cache_path: &Path, file_size: u64, keyframes: &[f64]) -> Result<()> {
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

fn probe_keyframes(input: &Path) -> Result<Vec<f64>> {
    info!("extracting keyframes");
    let output = Command::new("ffprobe")
        .args([
            "-fflags",
            "+genpts",
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-skip_frame",
            "nokey",
            "-show_frames",
            "-show_entries",
            "frame=best_effort_timestamp_time,pkt_pts_time",
            "-of",
            "json",
        ])
        .arg(input)
        .output()
        .context("failed to run ffprobe for keyframes")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("ffprobe keyframe scan failed: {stderr}");
    }

    let frames: ProbeFrames =
        serde_json::from_slice(&output.stdout).context("failed to parse ffprobe keyframes JSON")?;

    let mut times = Vec::new();
    for frame in frames.frames {
        let value = frame.best_effort_timestamp_time.or(frame.pkt_pts_time);
        if let Some(value) = value {
            if let Ok(parsed) = value.parse::<f64>() {
                times.push(parsed);
            }
        }
    }

    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    info!(keyframes = times.len(), "keyframe extraction complete");
    Ok(times)
}
