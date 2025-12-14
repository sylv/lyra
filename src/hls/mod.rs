use std::fmt::Write as _;
use std::path::PathBuf;

use anyhow::{Context, Result};
use axum::{
    Router,
    body::Body,
    extract::{Path as AxumPath, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use nightfall::{
    error::NightfallError,
    patch::{init_segment::patch_init_segment, segment::patch_segment},
};
use tokio::{
    fs::{create_dir_all, read, remove_dir_all},
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use tower_http::cors::{Any, CorsLayer};

use crate::{AppState, error::AppError, ffprobe::probe};

const SEGMENT_DURATION: f64 = 5.0;
const TEST_FILE: &str = "placeholder.mkv";
const SEGMENT_ROOT: &str = "/tmp/lyra-hls";

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
    State(_state): State<AppState>,
    AxumPath(file_id): AxumPath<String>,
) -> Result<impl IntoResponse, AppError> {
    let _ = file_id; // TODO: wire file IDs to real paths
    let info = probe(TEST_FILE)?;
    let total_seconds = info.duration.as_secs_f64();
    let segments = (total_seconds / SEGMENT_DURATION).ceil() as u32;

    let mut playlist = String::new();
    writeln!(
        &mut playlist,
        "#EXTM3U\n#EXT-X-VERSION:7\n#EXT-X-TARGETDURATION:{SEGMENT_DURATION:.0}\n#EXT-X-MEDIA-SEQUENCE:0\n#EXT-X-MAP:URI=\"init.mp4\""
    )
    .unwrap();

    for idx in 0..segments {
        writeln!(
            &mut playlist,
            "#EXTINF:{SEGMENT_DURATION:.3},\nsegments/segment_{idx}.m4s"
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
    State(_state): State<AppState>,
    AxumPath(file_id): AxumPath<String>,
) -> Result<impl IntoResponse, AppError> {
    let _ = file_id;
    let (init_path, _) = ensure_segment(TEST_FILE, 0).await?;
    let bytes = read(init_path).await?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .body(Body::from(bytes))
        .unwrap())
}

async fn segment_handler(
    State(_state): State<AppState>,
    AxumPath((file_id, segment_name)): AxumPath<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let _ = file_id;
    let segment_num = segment_name
        .trim_start_matches("segment_")
        .trim_end_matches(".m4s")
        .parse::<u64>()
        .context("invalid segment number")?;

    let (_, seg_path) = ensure_segment(TEST_FILE, segment_num).await?;
    let bytes = read(seg_path).await?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/iso.segment")
        .body(Body::from(bytes))
        .unwrap())
}

pub async fn ensure_segment(file_path: &str, segment: u64) -> Result<(PathBuf, PathBuf)> {
    let outdir = segment_dir(segment);
    if outdir.exists() {
        remove_dir_all(&outdir).await?;
    }
    create_dir_all(&outdir).await?;

    let seg_template = outdir.join("segment_%d.m4s");
    let init_path = outdir.join("init.mp4");

    let start_ss = (segment as f64 * SEGMENT_DURATION).to_string();

    let mut args = vec![
        "-y".into(),
        "-ss".into(),
        start_ss,
        "-i".into(),
        file_path.to_string(),
        "-copyts".into(),
        "-map".into(),
        "0:0".into(),
        "-c:0".into(),
        "copy".into(),
        "-start_at_zero".into(),
        "-vsync".into(),
        "passthrough".into(),
        "-avoid_negative_ts".into(),
        "disabled".into(),
        "-max_muxing_queue_size".into(),
        "2048".into(),
        "-f".into(),
        "hls".into(),
        "-start_number".into(),
        segment.to_string(),
        "-hls_flags".into(),
        "temp_file".into(),
        "-max_delay".into(),
        "5000000".into(),
        "-hls_fmp4_init_filename".into(),
        init_path.to_string_lossy().into_owned(),
        "-hls_time".into(),
        SEGMENT_DURATION.to_string(),
        "-hls_segment_type".into(),
        "fmp4".into(),
        "-hls_segment_filename".into(),
        seg_template.to_string_lossy().into_owned(),
    ];

    if segment > 0 {
        args.push("-hls_segment_options".into());
        args.push("movflags=frag_custom+dash+delay_moov+frag_discont".into());
    } else {
        args.push("-hls_segment_options".into());
        args.push("movflags=frag_custom+dash+delay_moov".into());
    }

    args.extend(["-loglevel".into(), "info".into(), "pipe:1".into()]);

    let mut child = Command::new("ffmpeg")
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().context("missing ffmpeg stdout")?;
    let mut reader = BufReader::new(stdout).lines();

    let mut init_done = false;
    let mut seg_done = false;
    let mut next_seg_done = false;
    while let Some(line) = reader.next_line().await? {
        match get_line_type(&line) {
            Some(LineKind::Init) => {
                tracing::debug!("Received init segment");
                init_done = true;
            }
            Some(LineKind::Segment(seg_num)) if seg_num == segment => {
                tracing::debug!("Received target segment: {}", seg_num);
                seg_done = true
            }
            Some(LineKind::Segment(seg_num)) if seg_num == segment + 1 => {
                tracing::debug!("Received next segment: {}", seg_num);
                next_seg_done = true
            }
            Some(LineKind::Segment(seg_num)) => assert!(
                seg_num <= segment,
                "received segment {seg_num} > requested {segment}"
            ),
            None => {}
        }

        if init_done && seg_done && next_seg_done {
            tracing::debug!("All segments received, breaking");
            break;
        }
    }

    let seg_path = outdir.join(format!("segment_{segment}.m4s"));
    assert!(init_path.exists(), "init not created");
    assert!(seg_path.exists(), "segment not created");

    child.kill().await?;
    apply_patches(&init_path, &seg_path, segment as u32).await?;

    Ok((init_path, seg_path))
}

fn segment_dir(segment: u64) -> PathBuf {
    PathBuf::from(format!("{SEGMENT_ROOT}/seg_{segment}"))
}

async fn apply_patches(init: &PathBuf, segment: &PathBuf, seq: u32) -> Result<()> {
    match patch_segment(segment.clone(), seq).await {
        Ok(_) => Ok(()),
        Err(NightfallError::PartialSegment(_)) => {
            patch_init_segment(init.clone(), segment.clone(), seq).await?;
            Ok(())
        }
        Err(e) => Err(anyhow::Error::new(e)),
    }
}

enum LineKind {
    Init,
    Segment(u64),
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
            if let Ok(num) = num_str.parse::<u64>() {
                return Some(LineKind::Segment(num));
            }
        }
    }
    None
}
