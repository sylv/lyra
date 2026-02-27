use crate::{
    binaries::configured_ffmpeg_bin,
    profiles::ProfileContext,
    state::{FfmpegState, StreamProfileState},
};
use anyhow::{Context, Result, bail};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Arc, atomic::Ordering},
    time::Duration,
};
use tokio::{
    fs,
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    time::{Instant, sleep},
};

const THROTTLE_AHEAD_SEGMENTS: i64 = 4;
const UNTHROTTLE_WITHIN_SEGMENTS: i64 = 1;
const SEEK_RESTART_THRESHOLD_SECONDS: f64 = 24.0;

pub async fn ensure_ffmpeg_for_init(state: &Arc<StreamProfileState>) -> Result<()> {
    let _guard = state.ffmpeg_ops.lock().await;
    let needs_start = {
        let mut ffmpeg = state.ffmpeg.lock().await;
        update_child_status(state, &mut ffmpeg)?;
        ffmpeg.child.is_none()
    };

    if needs_start {
        tracing::info!(
            stream_id = state.stream.stream_id,
            profile = state.profile.id_name(),
            "starting ffmpeg for init segment"
        );
        clean_segments(&state.segment_dir).await?;
        start_ffmpeg(state, 0).await?;
    }
    Ok(())
}

pub async fn ensure_ffmpeg_for_segment(
    state: &Arc<StreamProfileState>,
    requested_segment: i64,
    requested_start_pts: Option<i64>,
) -> Result<()> {
    if requested_segment < 0 {
        return Ok(());
    }
    tracing::debug!(
        stream_id = state.stream.stream_id,
        profile = state.profile.id_name(),
        requested_segment,
        requested_start_pts,
        "segment request received"
    );
    let segment_count = state.segment_start_pts.len() as i64;
    if requested_segment >= segment_count {
        bail!("segment {requested_segment} out of range");
    }

    if let (Some(start_pts), Some(expected)) = (
        requested_start_pts,
        state.segment_start_pts.get(requested_segment as usize),
    ) {
        if *expected != start_pts {
            tracing::warn!(expected, start_pts, "segment startPts mismatch");
        }
    }

    let _guard = state.ffmpeg_ops.lock().await;
    let last_generated = find_last_generated(&state.segment_dir).await.unwrap_or(-1);
    state
        .last_generated
        .store(last_generated, Ordering::Relaxed);
    let segment_ready = {
        let path = state.segment_dir.join(format!("{requested_segment}.m4s"));
        match fs::metadata(&path).await {
            Ok(metadata) => metadata.len() > 0,
            Err(_) => false,
        }
    };

    let action = {
        let mut ffmpeg = state.ffmpeg.lock().await;
        update_child_status(state, &mut ffmpeg)?;

        if ffmpeg.child.is_none() {
            ffmpeg.last_requested_segment = requested_segment;
            if segment_ready {
                FfmpegAction::None
            } else {
                FfmpegAction::Start
            }
        } else if should_restart_ffmpeg(state, &ffmpeg, requested_segment, last_generated) {
            tracing::info!(
                requested_segment,
                start_segment = ffmpeg.start_segment,
                last_generated,
                "restart required for seek"
            );
            ffmpeg.last_requested_segment = requested_segment;
            FfmpegAction::Restart
        } else {
            ffmpeg.last_requested_segment = requested_segment;
            FfmpegAction::None
        }
    };

    match action {
        FfmpegAction::None => Ok(()),
        FfmpegAction::Start => {
            tracing::info!(start_segment = requested_segment, "starting ffmpeg");
            clean_segments(&state.segment_dir).await?;
            start_ffmpeg(state, requested_segment).await
        }
        FfmpegAction::Restart => {
            tracing::info!(start_segment = requested_segment, "restarting ffmpeg");
            stop_ffmpeg(state).await?;
            clean_segments(&state.segment_dir).await?;
            start_ffmpeg(state, requested_segment).await
        }
    }
}

pub async fn wait_for_file(path: &Path, timeout: Duration) -> Result<()> {
    let start = Instant::now();
    loop {
        match fs::metadata(path).await {
            Ok(metadata) if metadata.len() > 0 => return Ok(()),
            Ok(_) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(err.into()),
        }
        if start.elapsed() > timeout {
            bail!("timed out waiting for {}", path.display());
        }
        sleep(Duration::from_millis(50)).await;
    }
}

pub fn parse_segment_index(name: &str) -> Option<i64> {
    if name == "init.mp4" || name == "-1.mp4" {
        return Some(-1);
    }
    let name = name.strip_suffix(".m4s")?;
    name.parse::<i64>().ok()
}

async fn start_ffmpeg(state: &Arc<StreamProfileState>, start_segment: i64) -> Result<()> {
    let start_pts = state.segment_start_pts[start_segment as usize];
    let start_seconds = pts_to_seconds(
        start_pts,
        state.timeline_time_base_num,
        state.timeline_time_base_den,
    );

    tracing::info!(
        stream_id = state.stream.stream_id,
        profile = state.profile.id_name(),
        start_segment,
        start_pts,
        start_seconds,
        "starting ffmpeg"
    );

    let ctx = ProfileContext {
        input: state.input.clone(),
        stream: state.stream.clone(),
        stream_info: state.stream_info.clone(),
        keyframes: state.keyframes.clone(),
    };

    let args = state
        .profile
        .build_args(&ctx, start_segment, start_seconds, &state.hls_cuts);
    let ffmpeg_bin = resolve_ffmpeg_bin();

    tracing::debug!(
        cwd = %state.segment_dir.display(),
        ffmpeg_bin = %ffmpeg_bin.display(),
        args = ?args,
        "ffmpeg args"
    );

    let mut command = Command::new(&ffmpeg_bin);
    command.current_dir(&state.segment_dir);
    command
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .with_context(|| format!("failed to start ffmpeg with {}", ffmpeg_bin.display()))?;
    let pid = child.id().context("ffmpeg missing pid")?;

    if let Some(stdout) = child.stdout.take() {
        let last_generated = Arc::clone(&state.last_generated);
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(index) = parse_segment_index_from_line(&line) {
                    last_generated.store(index, Ordering::Relaxed);
                }
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                tracing::debug!(target: "ffmpeg", "{line}");
            }
        });
    }

    {
        let mut ffmpeg = state.ffmpeg.lock().await;
        ffmpeg.child = Some(child);
        ffmpeg.pid = Some(pid);
        ffmpeg.start_segment = start_segment;
        ffmpeg.last_requested_segment = start_segment;
        ffmpeg.throttled = false;
    }

    let state_clone = state.clone();
    let handle = tokio::spawn(async move {
        throttle_loop(state_clone).await;
    });
    let mut ffmpeg = state.ffmpeg.lock().await;
    ffmpeg.throttle_task = Some(handle);

    Ok(())
}

async fn stop_ffmpeg(state: &Arc<StreamProfileState>) -> Result<()> {
    let (mut child, pid, handle) = {
        let mut ffmpeg = state.ffmpeg.lock().await;
        (
            ffmpeg.child.take(),
            ffmpeg.pid.take(),
            ffmpeg.throttle_task.take(),
        )
    };

    if let Some(handle) = handle {
        handle.abort();
    }

    if let Some(pid) = pid {
        tracing::info!(pid, "resuming ffmpeg before shutdown");
        let _ = send_signal(pid, libc::SIGCONT);
    }

    if let Some(mut child) = child.take() {
        tracing::info!("killing ffmpeg process");
        let _ = child.kill().await;
        let _ = child.wait().await;
    }

    Ok(())
}

async fn throttle_loop(state: Arc<StreamProfileState>) {
    loop {
        sleep(Duration::from_millis(300)).await;
        let last_generated = match find_last_generated(&state.segment_dir).await {
            Ok(value) => value,
            Err(err) => {
                tracing::warn!(error = %err, "failed to scan segments");
                continue;
            }
        };
        state
            .last_generated
            .store(last_generated, Ordering::Relaxed);

        let mut ffmpeg = state.ffmpeg.lock().await;
        if ffmpeg.child.is_none() {
            break;
        }

        update_child_status(&state, &mut ffmpeg).ok();
        if ffmpeg.child.is_none() {
            break;
        }

        let delta = last_generated - ffmpeg.last_requested_segment;

        if let Some(pid) = ffmpeg.pid {
            if delta > THROTTLE_AHEAD_SEGMENTS && !ffmpeg.throttled {
                if send_signal(pid, libc::SIGSTOP).is_ok() {
                    tracing::info!(
                        pid,
                        last_generated,
                        last_requested = ffmpeg.last_requested_segment,
                        "ffmpeg throttled (SIGSTOP)"
                    );
                    ffmpeg.throttled = true;
                }
            } else if delta <= UNTHROTTLE_WITHIN_SEGMENTS && ffmpeg.throttled {
                if send_signal(pid, libc::SIGCONT).is_ok() {
                    tracing::info!(
                        pid,
                        last_generated,
                        last_requested = ffmpeg.last_requested_segment,
                        "ffmpeg resumed (SIGCONT)"
                    );
                    ffmpeg.throttled = false;
                }
            }
        }
    }
}

fn send_signal(pid: u32, signal: i32) -> Result<()> {
    let result = unsafe { libc::kill(pid as i32, signal) };
    if result == 0 {
        Ok(())
    } else {
        let err = std::io::Error::last_os_error();
        bail!("failed to signal ffmpeg pid {pid}: {err}")
    }
}

async fn clean_segments(dir: &Path) -> Result<()> {
    tracing::debug!(dir = %dir.display(), "cleaning segment files");
    let mut entries = fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "mp4" || ext == "m4s" || ext == "m3u8" {
                    let _ = fs::remove_file(&path).await;
                }
            }
        }
    }
    Ok(())
}

async fn find_last_generated(dir: &Path) -> Result<i64> {
    let mut entries = fs::read_dir(dir).await?;
    let mut max_index = -1;
    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.ends_with(".m4s") {
            continue;
        }
        let stem = &name[..name.len() - 4];
        if let Ok(index) = stem.parse::<i64>() {
            if index > max_index {
                max_index = index;
            }
        }
    }
    Ok(max_index)
}

fn update_child_status(state: &StreamProfileState, ffmpeg: &mut FfmpegState) -> Result<()> {
    if let Some(child) = ffmpeg.child.as_mut() {
        if let Ok(Some(status)) = child.try_wait() {
            let last_generated = state.last_generated.load(Ordering::Relaxed);
            let last_index = state.segment_start_pts.len() as i64 - 1;
            if last_index >= 0 && last_generated >= last_index {
                tracing::info!(
                    ?status,
                    last_generated,
                    last_index,
                    "ffmpeg exited after segment generation"
                );
            } else {
                tracing::error!(
                    ?status,
                    last_generated,
                    last_index,
                    "ffmpeg exited prematurely"
                );
            }
            ffmpeg.child = None;
            ffmpeg.pid = None;
            ffmpeg.throttled = false;
            ffmpeg.throttle_task = None;
        }
    }
    Ok(())
}

fn should_restart_ffmpeg(
    state: &StreamProfileState,
    ffmpeg: &FfmpegState,
    requested_segment: i64,
    last_generated: i64,
) -> bool {
    if requested_segment < ffmpeg.start_segment {
        return true;
    }

    let baseline = if last_generated >= 0 {
        last_generated
    } else {
        ffmpeg.start_segment
    };
    if requested_segment <= baseline {
        return false;
    }

    let requested_start = pts_to_av_time(
        state.segment_start_pts[requested_segment as usize],
        state.timeline_time_base_num,
        state.timeline_time_base_den,
    );
    let baseline_start = pts_to_av_time(
        state.segment_start_pts[baseline as usize],
        state.timeline_time_base_num,
        state.timeline_time_base_den,
    );
    requested_start - baseline_start > (SEEK_RESTART_THRESHOLD_SECONDS * 1_000_000.0) as i64
}

fn parse_segment_index_from_line(line: &str) -> Option<i64> {
    let trimmed = line.trim();
    let candidate = trimmed.strip_suffix(".m4s")?;
    candidate.parse::<i64>().ok()
}

enum FfmpegAction {
    None,
    Start,
    Restart,
}

fn resolve_ffmpeg_bin() -> PathBuf {
    if let Some(path) = configured_ffmpeg_bin() {
        return path;
    }

    if let Ok(path) = std::env::var("LYRA_FFMPEG_BIN") {
        return PathBuf::from(path);
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let local_candidate = manifest_dir.join("bin/ffmpeg");
    if local_candidate.exists() {
        return local_candidate;
    }

    let workspace_candidate = manifest_dir.join("../../bin/ffmpeg");
    if workspace_candidate.exists() {
        return workspace_candidate;
    }

    PathBuf::from("ffmpeg")
}

fn pts_to_seconds(pts: i64, time_base_num: i64, time_base_den: i64) -> f64 {
    (pts as f64) * (time_base_num as f64) / (time_base_den as f64)
}

fn pts_to_av_time(pts: i64, time_base_num: i64, time_base_den: i64) -> i64 {
    let num = pts as i128 * time_base_num as i128 * 1_000_000i128;
    let den = time_base_den as i128;
    (num / den) as i64
}
