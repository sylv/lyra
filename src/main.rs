mod config;
mod ffmpeg;
mod keyframes;
mod model;
mod playlist;
mod profiles;
mod state;

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
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::fs;
use tokio_util::io::ReaderStream;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

use crate::{
    ffmpeg::{
        ensure_ffmpeg_for_init, ensure_ffmpeg_for_segment, parse_segment_index, wait_for_file,
    },
    profiles::{AudioAacProfile, Profile, VideoCopyProfile},
    state::{
        AppState, StreamProfileKey, build_master_playlist, build_stream_profiles,
        create_process_segment_dir, load_keyframes_if_needed, prepare_segments_root, probe_streams,
    },
};

#[derive(Deserialize)]
struct SegmentQuery {
    #[serde(rename = "startPts")]
    start_pts: Option<i64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let input = parse_input_path()?;
    let segments_root = prepare_segments_root()?;
    let process_dir = create_process_segment_dir(&segments_root)?;

    let (streams, primary_video_info, duration_seconds) = probe_streams(&input)?;
    let keyframes = load_keyframes_if_needed(&input, primary_video_info.is_some())?;

    let profiles: Vec<Arc<dyn Profile>> = vec![
        Arc::new(VideoCopyProfile),
        // Arc::new(VideoH264Profile),
        Arc::new(AudioAacProfile),
    ];

    let stream_profiles = build_stream_profiles(
        &input,
        &process_dir,
        &streams,
        primary_video_info.as_ref(),
        keyframes,
        duration_seconds,
        &profiles,
    )?;

    let master_playlist = build_master_playlist(&stream_profiles, &streams)?;

    let state = Arc::new(AppState {
        master_playlist,
        stream_profiles,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/master.m3u8", get(master_handler))
        .route(
            "/stream/{stream_id}/{profile_id}/index.m3u8",
            get(stream_index_handler),
        )
        .route(
            "/stream/{stream_id}/{profile_id}/segment/{name}",
            get(stream_segment_handler),
        )
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4422").await?;
    info!("listening on http://0.0.0.0:4422");
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

async fn master_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")],
        state.master_playlist.clone(),
    )
}

async fn stream_index_handler(
    State(state): State<Arc<AppState>>,
    Path((stream_id, profile_id)): Path<(u32, String)>,
) -> Result<Response, StatusCode> {
    let key = StreamProfileKey {
        stream_id,
        profile_id,
    };
    let stream_profile = state
        .stream_profiles
        .get(&key)
        .ok_or(StatusCode::BAD_REQUEST)?;

    let mut response = Response::new(Body::from(stream_profile.playlist.clone()));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.apple.mpegurl"),
    );
    Ok(response)
}

async fn stream_segment_handler(
    State(state): State<Arc<AppState>>,
    Path((stream_id, profile_id, name)): Path<(u32, String, String)>,
    Query(query): Query<SegmentQuery>,
) -> Result<Response, StatusCode> {
    let key = StreamProfileKey {
        stream_id,
        profile_id,
    };
    let stream_profile = state
        .stream_profiles
        .get(&key)
        .ok_or(StatusCode::BAD_REQUEST)?
        .clone();

    let segment_index = parse_segment_index(&name).ok_or(StatusCode::NOT_FOUND)?;
    if segment_index >= 0 {
        if segment_index as usize >= stream_profile.segment_start_seconds.len() {
            return Err(StatusCode::NOT_FOUND);
        }
        ensure_ffmpeg_for_segment(&stream_profile, segment_index, query.start_pts)
            .await
            .map_err(|err| {
                warn!(error = %err, "segment request failed");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    } else {
        ensure_ffmpeg_for_init(&stream_profile)
            .await
            .map_err(|err| {
                warn!(error = %err, "init request failed");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    let path = stream_profile.segment_dir.join(&name);
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
