use std::fmt::Write as _;
use std::path::PathBuf;

use crate::{AppState, error::AppError};
use anyhow::{Context, Result};
use axum::{
    Router,
    body::Body,
    extract::{Path as AxumPath, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use tokio::{
    fs::{create_dir_all, read, remove_dir_all},
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use tower_http::cors::{Any, CorsLayer};

pub struct Segment {
    pub id: usize,
    pub start: f64,
    pub duration: f64,
}

const TARGET_SEGMENT_DURATION: f64 = 6.0;
pub const TEST_FILE: &str = "test.mkv";
pub const SEGMENT_ROOT: &str = "/tmp/lyra-hls";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/hls/{file_id}/index.m3u8", get(playlist_handler))
        .route("/hls/{file_id}/init.mp4", get(init_handler))
        .route(
            "/hls/{file_id}/segments/{segment_name}",
            get(segment_handler),
        )
        .layer(cors_layer())
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_headers(Any)
        .allow_methods(Any)
        .allow_origin(Any)
}

async fn playlist_handler(
    State(state): State<AppState>,
    AxumPath(_file_id): AxumPath<String>,
) -> Result<impl IntoResponse, AppError> {
    let mut playlist = String::new();
    let segments = get_segment_times(&state.keyframes).await?;
    let max_segment_duration = segments
        .iter()
        .map(|s| s.duration)
        .fold(0.0_f64, f64::max)
        .ceil();

    writeln!(
        &mut playlist,
        "#EXTM3U\n#EXT-X-VERSION:7\n#EXT-X-TARGETDURATION:{max_segment_duration:.0}\n#EXT-X-MEDIA-SEQUENCE:0\n#EXT-X-MAP:URI=\"init.mp4\""
    )
    .unwrap();
    for segment in segments.iter() {
        writeln!(&mut playlist, "#EXT-X-DISCONTINUITY").unwrap();
        writeln!(
            &mut playlist,
            "#EXTINF:{:.3},\nsegments/segment_{}.m4s",
            segment.duration, segment.id
        )
        .unwrap();
    }

    playlist.push_str("#EXT-X-ENDLIST\n");

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
        .body(Body::from(playlist))
        .unwrap())
}

async fn init_handler(
    State(state): State<AppState>,
    AxumPath(file_id): AxumPath<String>,
) -> Result<impl IntoResponse, AppError> {
    let _ = file_id;
    let first_seg = get_segment_from_id(&state.keyframes, 0).await?;
    let (init_path, _) = ensure_segment(TEST_FILE, &first_seg).await?;
    let bytes = read(init_path).await?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .body(Body::from(bytes))
        .unwrap())
}

async fn segment_handler(
    State(state): State<AppState>,
    AxumPath((file_id, segment_name)): AxumPath<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let _ = file_id;
    let segment_num = segment_name
        .trim_start_matches("segment_")
        .trim_end_matches(".m4s")
        .parse::<usize>()
        .context("invalid segment number")?;

    let segment = get_segment_from_id(&state.keyframes, segment_num).await?;
    let (_, seg_path) = ensure_segment(TEST_FILE, &segment).await?;
    let bytes = read(seg_path).await?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/iso.segment")
        .body(Body::from(bytes))
        .unwrap())
}

async fn get_segment_times(keyframes: &[f64]) -> Result<Vec<Segment>> {
    // ffmpeg cuts on the first keyframe *after* hls_time has elapsed from the
    // start of the current segment. We mimic that by measuring elapsed time
    // from the current segment start to each subsequent keyframe and cutting
    // when the threshold is met or exceeded.
    if keyframes.is_empty() {
        return Ok(vec![]);
    }

    let mut segments = Vec::new();
    let mut segment_start = keyframes[0];
    let mut segment_id = 0;

    for &keyframe_ts in keyframes.iter().skip(1) {
        let elapsed = keyframe_ts - segment_start;
        if elapsed >= TARGET_SEGMENT_DURATION {
            segments.push(Segment {
                id: segment_id,
                start: segment_start,
                duration: elapsed,
            });

            segment_id += 1;
            segment_start = keyframe_ts;
        }
    }

    if let Some(&last_keyframe) = keyframes.last() {
        let elapsed = last_keyframe - segment_start;
        if elapsed > 0.0 {
            segments.push(Segment {
                id: segment_id,
                start: segment_start,
                duration: elapsed,
            });
        }
    }

    Ok(segments)
}

async fn get_segment_from_id(keyframes: &[f64], id: usize) -> Result<Segment> {
    let segment_times = get_segment_times(keyframes).await?;
    Ok(segment_times
        .into_iter()
        .find(|s| s.id == id)
        .expect("segment does not exist"))
}

pub async fn ensure_segment(file_path: &str, segment: &Segment) -> Result<(PathBuf, PathBuf)> {
    let outdir = segment_dir(segment.id);
    if outdir.exists() {
        remove_dir_all(&outdir).await?;
    }
    create_dir_all(&outdir).await?;

    let seg_template = outdir.join("segment_%d.m4s");
    let init_path = outdir.join("init.mp4");

    let start_ss = segment.start;
    tracing::info!("segment {} starts at {}", segment.id, start_ss);

    #[rustfmt::skip]
    let mut args: Vec<String> = vec![
        "-y".into(),
        "-ss".into(), start_ss.to_string(),
        "-i".into(), file_path.to_string(),
        "-map".into(), "0:0".into(),
        "-c:0".into(), "copy".into(),
        "-copyts".into(),
        "-start_at_zero".into(),
        "-vsync".into(), "passthrough".into(),
        "-avoid_negative_ts".into(), "disabled".into(),
        "-max_muxing_queue_size".into(), "2048".into(),
        "-f".into(), "hls".into(),
        "-start_number".into(), segment.id.to_string(),
        "-hls_flags".into(), "temp_file".into(),
        "-hls_time".into(), TARGET_SEGMENT_DURATION.to_string(),
        "-max_delay".into(), "5000000".into(),
        "-hls_fmp4_init_filename".into(), init_path.to_string_lossy().into_owned(),
        "-hls_segment_type".into(), "fmp4".into(),
        "-hls_segment_filename".into(),
        seg_template.to_string_lossy().into_owned(),
    ];

    args.extend([
        "-hls_segment_options".into(),
        "movflags=faststart:use_editlist=0".into(),
    ]);
    args.extend(["-loglevel".into(), "info".into()]);
    args.push("pipe:1".into());

    let mut child = Command::new("ffmpeg")
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().context("missing ffmpeg stdout")?;
    let mut reader = BufReader::new(stdout).lines();
    let mut playlist_raw = String::new();

    let mut init_done = false;
    let mut seg_done = false;
    let mut next_seg_done = false;
    while let Some(line) = reader.next_line().await? {
        playlist_raw.push_str(&line);
        playlist_raw.push('\n');

        match get_line_type(&line) {
            Some(LineKind::Init) => {
                tracing::debug!("Received init segment");
                init_done = true;
            }
            Some(LineKind::Segment(seg_num)) if seg_num == segment.id => {
                tracing::debug!("Received target segment: {}", seg_num);
                seg_done = true
            }
            Some(LineKind::Segment(seg_num)) if seg_num == segment.id + 1 => {
                tracing::debug!("Received next segment: {}", seg_num);
                next_seg_done = true
            }
            Some(LineKind::Segment(seg_num)) => assert!(
                seg_num <= segment.id,
                "received segment {} > requested {}",
                seg_num,
                segment.id
            ),
            None => {}
        }

        if init_done && seg_done && next_seg_done {
            tracing::debug!("All segments received, breaking");
            break;
        }
    }

    let seg_path = outdir.join(format!("segment_{}.m4s", segment.id));
    assert!(init_path.exists(), "init not created");
    assert!(seg_path.exists(), "segment not created");

    child.kill().await?;

    Ok((init_path, seg_path))
}

fn segment_dir(segid: usize) -> PathBuf {
    PathBuf::from(format!("{SEGMENT_ROOT}/seg_{segid}"))
}

enum LineKind {
    Init,
    Segment(usize),
}

fn get_line_type(line: &str) -> Option<LineKind> {
    if line.starts_with("#EXT-X-MAP:URI=") {
        return Some(LineKind::Init);
    }
    if line.starts_with("segment_") && line.ends_with(".m4s") {
        if let Some(num_str) = line
            .strip_prefix("segment_")
            .and_then(|s| s.strip_suffix(".m4s"))
        {
            if let Ok(num) = num_str.parse::<usize>() {
                return Some(LineKind::Segment(num));
            }
        }
    }
    None
}
