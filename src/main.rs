mod keyframes;
mod playlist;

use anyhow::{Context, Result, bail};
use axum::{
    Router,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Deserialize;
use std::{
    path::{Path as StdPath, PathBuf},
    process::Stdio,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    fs,
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::Mutex,
    time::{Instant, sleep},
};
use tokio_util::io::ReaderStream;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, info, warn};

const TARGET_SEGMENT_SECONDS: f64 = 6.0;
const THROTTLE_AHEAD_SEGMENTS: i64 = 4;
const UNTHROTTLE_WITHIN_SEGMENTS: i64 = 1;
const SEEK_RESTART_THRESHOLD_SECONDS: f64 = 24.0;

struct AppState {
    input: PathBuf,
    segment_dir: PathBuf,
    playlist: String,
    segment_start_pts: Vec<i64>,
    segment_start_seconds: Vec<f64>,
    ffmpeg: Mutex<FfmpegState>,
    ffmpeg_ops: Mutex<()>,
}

struct FfmpegState {
    child: Option<tokio::process::Child>,
    pid: Option<u32>,
    start_segment: i64,
    last_requested_segment: i64,
    last_generated_segment: i64,
    throttled: bool,
    throttle_task: Option<tokio::task::JoinHandle<()>>,
}

impl Default for FfmpegState {
    fn default() -> Self {
        Self {
            child: None,
            pid: None,
            start_segment: 0,
            last_requested_segment: 0,
            last_generated_segment: -1,
            throttled: false,
            throttle_task: None,
        }
    }
}

#[derive(Deserialize)]
struct SegmentQuery {
    #[serde(rename = "startPts")]
    start_pts: Option<i64>,
}

#[derive(Deserialize)]
struct FfprobeOutput {
    streams: Vec<FfprobeStream>,
    format: Option<FfprobeFormat>,
}

#[derive(Deserialize)]
struct FfprobeStream {
    time_base: Option<String>,
}

#[derive(Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
}

struct StreamInfo {
    time_base_num: i64,
    time_base_den: i64,
    duration_seconds: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let input = parse_input_path()?;
    let keyframes = keyframes::load_or_probe_keyframes(&input)?;
    let stream_info = probe_stream_info(&input)?;

    let playlist = playlist::create_fmp4_hls_playlist_from_keyframes_seconds(
        &keyframes,
        stream_info.duration_seconds,
        TARGET_SEGMENT_SECONDS,
        stream_info.time_base_num,
        stream_info.time_base_den,
        "/segment/",
        "",
    )
    .map_err(|err| anyhow::anyhow!(err))?;

    let (segment_start_pts, segment_start_seconds) = compute_segment_starts(
        &keyframes,
        stream_info.duration_seconds,
        stream_info.time_base_num,
        stream_info.time_base_den,
        TARGET_SEGMENT_SECONDS,
    )?;

    let segment_dir = create_segment_dir()?;
    info!(dir = %segment_dir.display(), "segment directory ready");

    let state = Arc::new(AppState {
        input,
        segment_dir,
        playlist,
        segment_start_pts,
        segment_start_seconds,
        ffmpeg: Mutex::new(FfmpegState::default()),
        ffmpeg_ops: Mutex::new(()),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/index.m3u8", get(index_handler))
        .route("/segment/{name}", get(segment_handler))
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

fn parse_input_path() -> Result<PathBuf> {
    let mut args = std::env::args().skip(1);
    let input = args.next().context("usage: cargo run -- <input-file>")?;
    if args.next().is_some() {
        bail!("only a single input file is supported");
    }
    let path = PathBuf::from(input);
    if !path.exists() {
        bail!("input file does not exist: {}", path.display());
    }
    let canonical = std::fs::canonicalize(&path)
        .with_context(|| format!("failed to canonicalize input path {}", path.display()))?;
    Ok(canonical)
}

fn create_segment_dir() -> Result<PathBuf> {
    let pid = std::process::id();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let dir = std::env::temp_dir().join(format!("lyra-hls-{pid}-{timestamp}"));
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create segment dir {}", dir.display()))?;
    Ok(dir)
}

async fn index_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")],
        state.playlist.clone(),
    )
}

async fn segment_handler(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(query): Query<SegmentQuery>,
) -> Result<Response, StatusCode> {
    let segment_index = parse_segment_index(&name).ok_or(StatusCode::NOT_FOUND)?;
    if segment_index >= 0 {
        if segment_index as usize >= state.segment_start_seconds.len() {
            return Err(StatusCode::NOT_FOUND);
        }
        ensure_ffmpeg_for_segment(&state, segment_index, query.start_pts)
            .await
            .map_err(|err| {
                warn!(error = %err, "segment request failed");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    } else {
        ensure_ffmpeg_for_init(&state).await.map_err(|err| {
            warn!(error = %err, "init request failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    let path = state.segment_dir.join(&name);
    wait_for_file(&path, Duration::from_secs(10))
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let file = fs::File::open(&path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    let mut response = Response::new(body);
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("video/mp4"));
    Ok(response)
}

fn parse_segment_index(name: &str) -> Option<i64> {
    if name == "init.mp4" || name == "-1.mp4" {
        return Some(-1);
    }
    let name = name.strip_suffix(".m4s")?;
    name.parse::<i64>().ok()
}

async fn ensure_ffmpeg_for_init(state: &Arc<AppState>) -> Result<()> {
    let _guard = state.ffmpeg_ops.lock().await;
    let needs_start = {
        let mut ffmpeg = state.ffmpeg.lock().await;
        update_child_status(&mut ffmpeg)?;
        ffmpeg.child.is_none()
    };

    if needs_start {
        info!("starting ffmpeg for init segment");
        clean_segments(&state.segment_dir).await?;
        start_ffmpeg(state, 0).await?;
    }
    Ok(())
}

async fn ensure_ffmpeg_for_segment(
    state: &Arc<AppState>,
    requested_segment: i64,
    requested_start_pts: Option<i64>,
) -> Result<()> {
    if requested_segment < 0 {
        return Ok(());
    }
    debug!(
        requested_segment,
        requested_start_pts, "segment request received"
    );
    let segment_count = state.segment_start_seconds.len() as i64;
    if requested_segment >= segment_count {
        bail!("segment {requested_segment} out of range");
    }

    if let Some(start_pts) = requested_start_pts {
        if let Some(expected) = state.segment_start_pts.get(requested_segment as usize) {
            if *expected != start_pts {
                warn!(expected, start_pts, "segment startPts mismatch");
            }
        }
    }

    let _guard = state.ffmpeg_ops.lock().await;
    let last_generated = find_last_generated(&state.segment_dir).await.unwrap_or(-1);
    let action = {
        let mut ffmpeg = state.ffmpeg.lock().await;
        update_child_status(&mut ffmpeg)?;

        if ffmpeg.child.is_none() {
            ffmpeg.last_requested_segment = requested_segment;
            ffmpeg.last_generated_segment = last_generated;
            FfmpegAction::Start
        } else if should_restart_ffmpeg(state, &ffmpeg, requested_segment, last_generated) {
            info!(
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
            info!(start_segment = requested_segment, "starting ffmpeg");
            clean_segments(&state.segment_dir).await?;
            start_ffmpeg(state, requested_segment).await
        }
        FfmpegAction::Restart => {
            info!(start_segment = requested_segment, "restarting ffmpeg");
            stop_ffmpeg(state).await?;
            clean_segments(&state.segment_dir).await?;
            start_ffmpeg(state, requested_segment).await
        }
    }
}

fn should_restart_ffmpeg(
    state: &AppState,
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

    let requested_start = state.segment_start_seconds[requested_segment as usize];
    let baseline_start = state.segment_start_seconds[baseline as usize];
    requested_start - baseline_start > SEEK_RESTART_THRESHOLD_SECONDS
}

fn update_child_status(ffmpeg: &mut FfmpegState) -> Result<()> {
    if let Some(child) = ffmpeg.child.as_mut() {
        if let Ok(Some(status)) = child.try_wait() {
            warn!(?status, "ffmpeg exited");
            ffmpeg.child = None;
            ffmpeg.pid = None;
            ffmpeg.throttled = false;
            ffmpeg.throttle_task = None;
        }
    }
    Ok(())
}

async fn start_ffmpeg(state: &Arc<AppState>, start_segment: i64) -> Result<()> {
    let start_seconds = state.segment_start_seconds[start_segment as usize];
    let hls_time = TARGET_SEGMENT_SECONDS.to_string();

    info!(
        input = %state.input.display(),
        start_segment,
        start_seconds,
        "starting ffmpeg"
    );

    let mut args: Vec<std::ffi::OsString> = Vec::new();

    // seeking on the first segment can cause weirdness
    if start_segment > 0 {
        // when copying, -ss seeks to the first keyframe before the target position.
        // that means, since we are using exact keyframe positions, we will seek to the keyframe before
        // the one we want. this gets around that by seeking slightly ahead of it.
        let seek_seconds = start_seconds + 0.5;
        args.push("-ss".into());
        args.push(format!("{seek_seconds:.6}").into());
        args.push("-noaccurate_seek".into());
    }

    args.extend(["-i".into(), state.input.clone().into_os_string()]);

    {
        #[rustfmt::skip]
        args.extend([
            "-map_metadata", "-1",
            "-map_chapters", "-1",
            "-threads", "4",
            "-map", "0:v:0",
            "-map", "0:a:0?",
            "-codec:v:0", "copy",
            "-codec:a:0", "copy",
            // "-start_at_zero",
            "-copyts",
            "-avoid_negative_ts", "disabled",
            "-hls_segment_type", "fmp4",
        ].into_iter().map(Into::into));
    }

    args.extend(["-hls_time".into(), hls_time.clone().into()]);
    args.extend(["-start_number".into(), start_segment.to_string().into()]);

    {
        #[rustfmt::skip]
        args.extend([
            "-hls_segment_filename", "%d.m4s",
            "-hls_fmp4_init_filename", "init.mp4",
            "-hls_segment_options", "movflags=+frag_discont",
            "-hls_playlist_type", "vod",
            "-hls_list_size", "0",
            "-y",
            "ffmpeg.m3u8",
        ].into_iter().map(Into::into));
    }
    debug!(
        cwd = %state.segment_dir.display(),
        args = ?args,
        "ffmpeg args"
    );

    let mut command = Command::new("ffmpeg");
    command.current_dir(&state.segment_dir);
    command
        .args(&args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let mut child = command.spawn().context("failed to start ffmpeg")?;
    let pid = child.id().context("ffmpeg missing pid")?;
    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                info!(target: "ffmpeg", "{line}");
            }
        });
    }

    {
        let mut ffmpeg = state.ffmpeg.lock().await;
        ffmpeg.child = Some(child);
        ffmpeg.pid = Some(pid);
        ffmpeg.start_segment = start_segment;
        ffmpeg.last_requested_segment = start_segment;
        ffmpeg.last_generated_segment = start_segment - 1;
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

async fn stop_ffmpeg(state: &Arc<AppState>) -> Result<()> {
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
        info!(pid, "resuming ffmpeg before shutdown");
        let _ = send_signal(pid, libc::SIGCONT);
    }

    if let Some(mut child) = child.take() {
        info!("killing ffmpeg process");
        let _ = child.kill().await;
        let _ = child.wait().await;
    }

    Ok(())
}

async fn throttle_loop(state: Arc<AppState>) {
    loop {
        sleep(Duration::from_millis(300)).await;
        let last_generated = match find_last_generated(&state.segment_dir).await {
            Ok(value) => value,
            Err(err) => {
                warn!(error = %err, "failed to scan segments");
                continue;
            }
        };

        let mut ffmpeg = state.ffmpeg.lock().await;
        if ffmpeg.child.is_none() {
            break;
        }

        update_child_status(&mut ffmpeg).ok();
        if ffmpeg.child.is_none() {
            break;
        }

        ffmpeg.last_generated_segment = last_generated;
        let delta = last_generated - ffmpeg.last_requested_segment;

        if let Some(pid) = ffmpeg.pid {
            if delta > THROTTLE_AHEAD_SEGMENTS && !ffmpeg.throttled {
                if send_signal(pid, libc::SIGSTOP).is_ok() {
                    info!(
                        pid,
                        last_generated,
                        last_requested = ffmpeg.last_requested_segment,
                        "ffmpeg throttled (SIGSTOP)"
                    );
                    ffmpeg.throttled = true;
                }
            } else if delta <= UNTHROTTLE_WITHIN_SEGMENTS && ffmpeg.throttled {
                if send_signal(pid, libc::SIGCONT).is_ok() {
                    info!(
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

async fn clean_segments(dir: &StdPath) -> Result<()> {
    debug!(dir = %dir.display(), "cleaning segment files");
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

async fn find_last_generated(dir: &StdPath) -> Result<i64> {
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

async fn wait_for_file(path: &StdPath, timeout: Duration) -> Result<()> {
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

fn probe_stream_info(input: &StdPath) -> Result<StreamInfo> {
    let output = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=time_base",
            "-show_entries",
            "format=duration",
            "-of",
            "json",
        ])
        .arg(input)
        .output()
        .context("failed to run ffprobe for stream info")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("ffprobe stream info failed: {stderr}");
    }

    let parsed: FfprobeOutput =
        serde_json::from_slice(&output.stdout).context("failed to parse ffprobe JSON")?;
    let time_base = parsed
        .streams
        .first()
        .and_then(|stream| stream.time_base.as_ref())
        .context("missing time_base from ffprobe")?;
    let (time_base_num, time_base_den) = parse_time_base(time_base)?;

    let duration = parsed
        .format
        .as_ref()
        .and_then(|format| format.duration.as_ref())
        .context("missing duration from ffprobe")?;
    let duration_seconds: f64 = duration
        .parse()
        .context("failed to parse duration from ffprobe")?;

    Ok(StreamInfo {
        time_base_num,
        time_base_den,
        duration_seconds,
    })
}

fn parse_time_base(value: &str) -> Result<(i64, i64)> {
    let mut parts = value.split('/');
    let num = parts
        .next()
        .context("invalid time_base numerator")?
        .parse::<i64>()
        .context("invalid time_base numerator")?;
    let den = parts
        .next()
        .context("invalid time_base denominator")?
        .parse::<i64>()
        .context("invalid time_base denominator")?;
    if parts.next().is_some() {
        bail!("invalid time_base format: {value}");
    }
    if num <= 0 || den <= 0 {
        bail!("invalid time_base values: {value}");
    }
    Ok((num, den))
}

fn compute_segment_starts(
    keyframes_seconds: &[f64],
    total_duration_seconds: f64,
    time_base_num: i64,
    time_base_den: i64,
    desired_segment_seconds: f64,
) -> Result<(Vec<i64>, Vec<f64>)> {
    let mut keyframes_pts: Vec<i64> = keyframes_seconds
        .iter()
        .map(|&s| playlist::seconds_to_pts(s, time_base_num, time_base_den))
        .collect();
    keyframes_pts.sort_unstable();
    keyframes_pts.dedup();

    let total_duration_pts =
        playlist::seconds_to_pts(total_duration_seconds, time_base_num, time_base_den);
    let desired_segment_length_pts =
        playlist::seconds_to_pts(desired_segment_seconds, time_base_num, time_base_den);

    let segments_pts = playlist::compute_segments_from_keyframes_pts(
        &keyframes_pts,
        total_duration_pts,
        desired_segment_length_pts,
    )
    .map_err(|err| anyhow::anyhow!(err))?;

    let mut start_pts = Vec::with_capacity(segments_pts.len());
    let mut cursor = 0i64;
    for len in segments_pts {
        start_pts.push(cursor);
        cursor += len;
    }

    let start_seconds = start_pts
        .iter()
        .map(|&pts| (pts as f64) * (time_base_num as f64) / (time_base_den as f64))
        .collect();

    Ok((start_pts, start_seconds))
}

enum FfmpegAction {
    None,
    Start,
    Restart,
}
