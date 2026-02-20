use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, Instant},
};

use anyhow::Context;
use lyra_ffprobe::probe_streams;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    time::timeout,
};

use crate::{
    MAX_EXTRACTION_RUNTIME_SECONDS, MAX_SCAN_SECONDS, MIN_SCAN_SECONDS, SCAN_RATIO,
    SNAPSHOT_INTERVAL_SECONDS, ThumbnailOptions,
};

pub(crate) async fn resolve_scan_window(
    video_path: &Path,
    options: &ThumbnailOptions,
) -> anyhow::Result<Duration> {
    let ffprobe_bin = options.ffprobe_bin.clone();
    let input = video_path.to_path_buf();

    let duration_seconds = tokio::task::spawn_blocking(move || {
        let probe = probe_streams(&ffprobe_bin, &input)?;
        Ok::<_, anyhow::Error>(probe.duration_seconds)
    })
    .await
    .context("ffprobe task join failure")??;

    let scan_seconds = duration_seconds
        .map(|seconds| {
            (seconds * SCAN_RATIO)
                .max(MIN_SCAN_SECONDS)
                .min(MAX_SCAN_SECONDS)
        })
        .unwrap_or(MIN_SCAN_SECONDS);

    Ok(Duration::from_secs_f64(scan_seconds))
}

pub(crate) async fn extract_snapshots(
    video_path: &Path,
    options: &ThumbnailOptions,
    scan_window: Duration,
) -> anyhow::Result<Vec<PathBuf>> {
    let frames_dir = &options.working_dir;
    if frames_dir.exists() {
        std::fs::remove_dir_all(frames_dir)?;
    }
    std::fs::create_dir_all(frames_dir)?;

    let args = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-progress".to_string(),
        "pipe:1".to_string(),
        "-i".to_string(),
        video_path.to_string_lossy().to_string(),
        "-map".to_string(),
        "0:v:0".to_string(),
        "-an".to_string(),
        "-sn".to_string(),
        "-dn".to_string(),
        "-t".to_string(),
        format!("{:.3}", scan_window.as_secs_f64()),
        "-vf".to_string(),
        format!("fps=1/{SNAPSHOT_INTERVAL_SECONDS}"),
        "-vsync".to_string(),
        "vfr".to_string(),
        "-q:v".to_string(),
        "2".to_string(),
        "-pix_fmt".to_string(),
        "yuvj420p".to_string(),
        frames_dir.join("%08d.jpg").to_string_lossy().to_string(),
    ];

    tracing::info!("extracting snapshot frames with args: {}", args.join(" "));
    let mut child = Command::new(&options.ffmpeg_bin)
        .args(&args)
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to get ffmpeg stdout"))?;

    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut last_log = Instant::now();
        let mut lines = reader.lines();
        let mut speed = None;
        let mut time = None;

        while let Some(line) = lines
            .next_line()
            .await
            .expect("failed to read ffmpeg progress line")
        {
            tracing::trace!(line);
            let mut parts = line.splitn(2, '=');
            let key = parts.next().unwrap_or_default();
            let value = parts.next().unwrap_or_default();
            match key {
                "speed" => speed = Some(value.to_string()),
                "out_time" => time = Some(value.to_string()),
                _ => {}
            }

            if let (Some(speed_i), Some(time_i)) = (&speed, &time) {
                if last_log.elapsed() > Duration::from_secs(10) {
                    tracing::info!(
                        "extracting snapshots for thumbnail, speed={}, time={}",
                        speed_i,
                        time_i
                    );
                    last_log = Instant::now();
                }
                speed = None;
                time = None;
            }
        }
    });

    let status = timeout(
        Duration::from_secs(MAX_EXTRACTION_RUNTIME_SECONDS),
        child.wait(),
    )
    .await;

    let status = match status {
        Ok(result) => result?,
        Err(_) => {
            let _ = child.kill().await;
            anyhow::bail!(
                "ffmpeg snapshot extraction timed out after {}s",
                MAX_EXTRACTION_RUNTIME_SECONDS
            );
        }
    };

    if !status.success() {
        anyhow::bail!("ffmpeg failed to extract thumbnail snapshots: {}", status);
    }

    let mut entries = tokio::fs::read_dir(frames_dir).await?;
    let mut frame_paths = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str());
        if matches!(ext, Some("jpg") | Some("jpeg")) {
            frame_paths.push(path);
        }
    }

    frame_paths.sort();
    if frame_paths.is_empty() {
        anyhow::bail!(
            "no snapshots extracted from {} within {:.3}s window",
            video_path.display(),
            scan_window.as_secs_f64()
        );
    }

    Ok(frame_paths)
}
