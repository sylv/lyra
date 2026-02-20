use std::{
    path::PathBuf,
    process::Stdio,
    time::{Duration, Instant},
};

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

use crate::PreviewOptions;

pub(crate) async fn extract_frame_paths(
    video_path: &PathBuf,
    options: &PreviewOptions,
) -> anyhow::Result<Vec<(u32, PathBuf)>> {
    let frames_dir = extract_frames(video_path, options).await?;
    tracing::info!("frame dir: {}", frames_dir.display());

    let mut handle = tokio::fs::read_dir(&frames_dir).await?;
    let mut frame_paths = Vec::new();

    while let Some(entry) = handle.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("png") {
            continue;
        }

        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let frame_num = match file_stem.parse::<u32>() {
            Ok(frame_num) => frame_num,
            Err(_) => {
                tracing::warn!("skipping non-frame file: {}", path.display());
                continue;
            }
        };
        frame_paths.push((frame_num, path));
    }

    frame_paths.sort_by_key(|(frame_num, _)| *frame_num);
    tracing::info!("discovered {} extracted frames", frame_paths.len());
    Ok(frame_paths)
}

async fn extract_frames(video_path: &PathBuf, options: &PreviewOptions) -> anyhow::Result<PathBuf> {
    let start = Instant::now();
    if options.working_dir.exists() {
        std::fs::remove_dir_all(&options.working_dir)?;
    }

    std::fs::create_dir_all(&options.working_dir)?;

    let args = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-progress".to_string(),
        "pipe:1".to_string(),
        "-i".to_string(),
        video_path.to_string_lossy().to_string(),
        "-vf".to_string(),
        format!(
            "fps=1/{},scale={}:-2:flags=lanczos",
            options.frame_interval_seconds, options.width_px
        ),
        options
            .working_dir
            .join("%08d.png")
            .to_string_lossy()
            .to_string(),
    ];

    tracing::info!("running ffmpeg with args: {}", args.join(" "));
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
            .expect("Failed to read line from ffmpeg progress")
        {
            tracing::trace!(line);
            let mut parts = line.splitn(2, '=');
            let key = parts.next().unwrap_or_default();
            let value = parts.next().unwrap_or_default();
            match key {
                "speed" => {
                    speed = Some(value.to_string());
                }
                "out_time" => {
                    time = Some(value.to_string());
                }
                _ => {}
            }

            if let (Some(speed_i), Some(time_i)) = (&speed, &time) {
                if last_log.elapsed() > Duration::from_secs(10) {
                    tracing::info!(
                        "generating timeline preview, speed={}, time={}",
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

    let status = child.wait().await?;
    if !status.success() {
        anyhow::bail!(
            "ffmpeg failed to extract timeline preview frames: {}",
            status
        );
    }
    tracing::info!(
        "extracted frames for timeline preview to {} in {:?}",
        options.working_dir.display(),
        start.elapsed()
    );
    Ok(options.working_dir.clone())
}
