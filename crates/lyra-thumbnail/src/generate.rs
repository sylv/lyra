use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use anyhow::Context;
use tokio::process::Command;

use crate::{Thumbnail, ThumbnailOptions};

struct FrameCandidate {
    path: PathBuf,
    width: u32,
    height: u32,
    bytes: u64,
}

pub(crate) async fn select_and_encode_best(
    frame_paths: &[PathBuf],
    options: &ThumbnailOptions,
    scan_window: Duration,
) -> anyhow::Result<Thumbnail> {
    let mut best: Option<FrameCandidate> = None;

    for path in frame_paths {
        let metadata = tokio::fs::metadata(path).await?;
        let (width, height) = image_dimensions(path).await?;
        let candidate = FrameCandidate {
            path: path.clone(),
            width,
            height,
            bytes: metadata.len(),
        };

        let candidate_key = (
            candidate.width as u64 * candidate.height as u64,
            candidate.bytes,
        );
        let best_key = best
            .as_ref()
            .map(|frame| (frame.width as u64 * frame.height as u64, frame.bytes));

        if best_key.is_none_or(|key| candidate_key > key) {
            best = Some(candidate);
        }
    }

    let best = best.context("no extracted keyframes available")?;
    let image_bytes = encode_webp(&best.path, options).await?;

    Ok(Thumbnail {
        image_bytes,
        source_width: best.width,
        source_height: best.height,
        scan_window,
    })
}

async fn image_dimensions(path: &Path) -> anyhow::Result<(u32, u32)> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || image::image_dimensions(path).map_err(anyhow::Error::from))
        .await
        .context("image dimension task join failure")?
}

async fn encode_webp(frame_path: &Path, options: &ThumbnailOptions) -> anyhow::Result<Vec<u8>> {
    let filter = format!(
        "scale=w='min({},iw)':h='min({},ih)':force_original_aspect_ratio=decrease:flags=lanczos",
        options.max_dimension_px, options.max_dimension_px
    );

    let output = Command::new(&options.ffmpeg_bin)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            &frame_path.to_string_lossy(),
            "-vf",
            &filter,
            "-frames:v",
            "1",
            "-c:v",
            "libwebp",
            "-q:v",
            &options.webp_quality.to_string(),
            "-f",
            "webp",
            "pipe:1",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!(
            "ffmpeg failed to encode thumbnail webp: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(output.stdout)
}
