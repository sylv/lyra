use anyhow::{Context, Result, bail};
use axum::{
    Router,
    extract::{Path as AxumPath, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use std::{
    env,
    net::SocketAddr,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
};
use serde::Deserialize;
use tokio::process::Command as TokioCommand;
use tokio_util::io::ReaderStream;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

mod keyframes;

const SEGMENT_DURATION: f64 = 6.0;

#[derive(Clone)]
struct AppState {
    input: PathBuf,
    segments: Arc<Vec<Segment>>,
    playlist: Arc<String>,
    audio_index: usize,
    init_cache: Arc<tokio::sync::Mutex<Option<Vec<u8>>>>,
}

#[derive(Clone, Copy, Debug)]
struct Segment {
    start: f64,
    duration: f64,
}

#[derive(Deserialize)]
struct ProbeInfo {
    format: Option<ProbeFormat>,
    streams: Vec<ProbeStream>,
}

#[derive(Deserialize)]
struct ProbeFormat {
    duration: Option<String>,
}

#[derive(Deserialize)]
struct ProbeStream {
    codec_type: Option<String>,
    tags: Option<ProbeTags>,
}

#[derive(Deserialize)]
struct ProbeTags {
    language: Option<String>,
}

#[derive(Debug)]
struct StreamInfo {
    duration: f64,
    has_video: bool,
    audio_index: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let input = input_path()?;
    let stream_info = probe_streams(&input)?;
    if !stream_info.has_video {
        bail!("no video stream found in input");
    }
    info!(
        duration = stream_info.duration,
        has_video = stream_info.has_video,
        audio_index = stream_info.audio_index,
        "ffprobe stream info"
    );

    let keyframes = keyframes::load_or_probe_keyframes(&input)?;
    if keyframes.is_empty() {
        warn!("no keyframes found; falling back to a single segment");
    }

    let segments = build_segments(&keyframes, stream_info.duration, SEGMENT_DURATION);
    let playlist = build_playlist(&segments);

    let state = Arc::new(AppState {
        input,
        segments: Arc::new(segments),
        playlist: Arc::new(playlist),
        audio_index: stream_info.audio_index,
        init_cache: Arc::new(tokio::sync::Mutex::new(None)),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/index.m3u8", get(playlist_handler))
        .route("/init.mp4", get(init_handler))
        .route("/segments/{id}", get(segment_handler))
        .with_state(state)
        .layer(cors);

    let addr: SocketAddr = ([127, 0, 0, 1], 4422).into();
    info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn root() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, HeaderValue::from_static("text/plain; charset=utf-8"))],
        "HLS test server. Load /index.m3u8\n",
    )
}

async fn playlist_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!(
        segments = state.segments.len(),
        "serving playlist"
    );
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/vnd.apple.mpegurl"),
        )],
        state.playlist.as_str().to_owned(),
    )
}

async fn init_handler(State(state): State<Arc<AppState>>) -> Result<Response, StatusCode> {
    let mut cache = state.init_cache.lock().await;
    if let Some(bytes) = cache.as_ref() {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "video/mp4")
            .body(axum::body::Body::from(bytes.clone()))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?);
    }

    info!("init segment generation started");
    let mut command = TokioCommand::new("ffmpeg");
    command
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
        ])
        .arg(&state.input)
        .args([
            "-map",
            "0:v:0",
            "-map",
            &format!("0:a:{}", state.audio_index),
            "-dn",
            "-sn",
            "-map_metadata",
            "-1",
            "-map_chapters",
            "-1",
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            "-ac",
            "2",
            "-f",
            "mp4",
            "-movflags",
            "+cmaf+empty_moov",
            "-t",
            "0",
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    let mut child = command.spawn().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut stdout = child.stdout.take().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut bytes = Vec::new();
    tokio::io::AsyncReadExt::read_to_end(&mut stdout, &mut bytes)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let status = child.wait().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !status.success() {
        warn!("ffmpeg init segment exited with status {status}");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    info!("init segment generation finished");
    *cache = Some(bytes.clone());
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .body(axum::body::Body::from(bytes))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
}

async fn segment_handler(
    AxumPath(id): AxumPath<usize>,
    State(state): State<Arc<AppState>>,
) -> Result<Response, StatusCode> {
    let segment = state
        .segments
        .get(id)
        .copied()
        .ok_or(StatusCode::NOT_FOUND)?;

    info!(
        segment_id = id,
        start = segment.start,
        duration = segment.duration,
        "segment generation started"
    );

    let mut command = TokioCommand::new("ffmpeg");
    command
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-ss",
            &format!("{:.3}", segment.start),
            "-i",
        ])
        .arg(&state.input)
        .args([
            "-t",
            &format!("{:.3}", segment.duration),
            "-reset_timestamps",
            "1",
            "-avoid_negative_ts",
            "make_zero",
            "-map",
            "0:v:0",
            "-map",
            &format!("0:a:{}", state.audio_index),
            "-dn",
            "-sn",
            "-map_metadata",
            "-1",
            "-map_chapters",
            "-1",
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            "-ac",
            "2",
            "-f",
            "mp4",
            "-movflags",
            "+cmaf",
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    let mut child = command.spawn().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let stdout = child.stdout.take().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let segment_id = id;
    tokio::spawn(async move {
        if let Ok(status) = child.wait().await {
            if !status.success() {
                warn!(segment_id, "ffmpeg exited with status {status}");
            } else {
                info!(segment_id, "segment generation finished");
            }
        }
    });

    let stream = ReaderStream::new(stdout);
    let body = axum::body::Body::from_stream(stream);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn input_path() -> Result<PathBuf> {
    let input = env::args().nth(1).context("usage: cargo run -- <input>")?;
    Ok(PathBuf::from(input))
}

fn probe_streams(input: &Path) -> Result<StreamInfo> {
    info!("probing stream metadata");
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-show_entries",
            "stream=codec_type:stream_tags=language",
            "-of",
            "json",
        ])
        .arg(input)
        .output()
        .context("failed to run ffprobe for stream info")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("ffprobe stream probe failed: {stderr}");
    }

    let info: ProbeInfo =
        serde_json::from_slice(&output.stdout).context("failed to parse ffprobe stream JSON")?;

    let duration = info
        .format
        .and_then(|format| format.duration)
        .context("ffprobe output missing duration")?
        .parse::<f64>()
        .context("ffprobe duration parse failed")?;

    let mut has_video = false;
    let mut audio_index = None;
    let mut audio_pos = 0usize;
    let mut audio_languages = Vec::new();
    for stream in info.streams.iter() {
        match stream.codec_type.as_deref() {
            Some("audio") => {
                if let Some(lang) = stream
                    .tags
                    .as_ref()
                    .and_then(|tags| tags.language.as_deref())
                {
                    audio_languages.push(lang.to_string());
                } else {
                    audio_languages.push("und".to_string());
                }
                if audio_index.is_none()
                    && stream
                        .tags
                        .as_ref()
                        .and_then(|tags| tags.language.as_deref())
                        == Some("eng")
                {
                    audio_index = Some(audio_pos);
                }
                audio_pos = audio_pos.saturating_add(1);
            }
            Some("video") => {
                has_video = true;
            }
            _ => {}
        }
    }

    let audio_index = audio_index.with_context(|| {
        if audio_languages.is_empty() {
            "no English (eng) audio stream found (no audio streams detected)".to_string()
        } else {
            format!(
                "no English (eng) audio stream found (languages: {})",
                audio_languages.join(", ")
            )
        }
    })?;
    Ok(StreamInfo {
        duration,
        has_video,
        audio_index,
    })
}

fn build_segments(keyframes: &[f64], duration: f64, target: f64) -> Vec<Segment> {
    if duration <= 0.0 {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut start = 0.0;
    let mut next_cut = target;

    for &kf in keyframes {
        if kf <= start {
            continue;
        }
        if kf + 0.001 >= next_cut {
            let end = kf.min(duration);
            let segment_duration = (end - start).max(0.0);
            if segment_duration > 0.05 {
                segments.push(Segment {
                    start,
                    duration: segment_duration,
                });
                start = end;
                next_cut = start + target;
            }
        }
    }

    if start < duration {
        segments.push(Segment {
            start,
            duration: duration - start,
        });
    }

    if segments.is_empty() {
        segments.push(Segment { start: 0.0, duration });
    }

    info!(
        segments = segments.len(),
        target_duration = target,
        "playlist segments built"
    );
    segments
}

fn build_playlist(segments: &[Segment]) -> String {
    let mut playlist = String::new();
    let max_duration = segments
        .iter()
        .map(|segment| segment.duration)
        .fold(0.0_f64, f64::max);
    let target_duration = max_duration.ceil().max(1.0) as u64;

    playlist.push_str("#EXTM3U\n");
    playlist.push_str("#EXT-X-VERSION:7\n");
    playlist.push_str(&format!("#EXT-X-TARGETDURATION:{target_duration}\n"));
    playlist.push_str("#EXT-X-MEDIA-SEQUENCE:0\n");
    playlist.push_str("#EXT-X-PLAYLIST-TYPE:VOD\n");
    playlist.push_str("#EXT-X-INDEPENDENT-SEGMENTS\n");
    playlist.push_str("#EXT-X-MAP:URI=\"/init.mp4\"\n");

    for (index, segment) in segments.iter().enumerate() {
        playlist.push_str(&format!("#EXTINF:{:.3},\n", segment.duration));
        playlist.push_str(&format!("/segments/{index}\n"));
    }

    playlist.push_str("#EXT-X-ENDLIST\n");
    playlist
}
