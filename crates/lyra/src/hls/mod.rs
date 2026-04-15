use crate::{
    AppState,
    auth::{RequestAuth, ensure_library_access},
    entities::{files, libraries},
    file_analysis,
    jobs::{self, FileProbeJob},
    signer::{sign, verify},
};
use anyhow::{Context, bail};
use axum::{
    Router,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode, header},
    response::Response,
    routing::get,
};
use lyra_packager::{
    AudioProfileSelection, Compatibility, SessionManager, SessionOptions, SessionSpec,
    VideoProfileSelection, audio_profile,
    playlist::create_fmp4_hls_playlist_from_segment_starts_pts, playlist::seconds_to_pts,
    video_profile,
};
use lyra_probe::{ProbeData, VideoKeyframes};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};
use tokio::{fs, sync::Mutex};
use tokio_util::io::ReaderStream;
#[cfg(debug_assertions)]
use tower_http::cors::CorsLayer;

const ON_DEMAND_JOB_TIMEOUT: Duration = Duration::from_secs(120);
const TARGET_SEGMENT_SECONDS: u64 = 6;

#[derive(Clone)]
pub struct PlaybackRegistry {
    pub sessions: Arc<SessionManager>,
    pub player_sessions: Arc<Mutex<HashMap<String, String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionResumable {
    pub file_id: String,
    pub video_stream_index: u32,
    pub video_profile_id: String,
    pub audio_stream_index: Option<u32>,
    pub audio_profile_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackTokenPayload {
    pub session_id: String,
    pub player_id: String,
    pub resumable: SessionResumable,
}

#[derive(Debug, Clone)]
pub struct MintPlaybackUrlInput {
    pub file_id: String,
    pub player_id: String,
    pub video_rendition_id: String,
    pub audio_stream_index: i32,
    pub audio_rendition_id: String,
}

#[derive(Debug, Clone)]
pub struct MintPlaybackUrlResult {
    pub url: String,
    pub packager_id: String,
}

#[derive(Debug, Deserialize)]
struct SegmentQuery {
    #[serde(rename = "startPts")]
    start_pts: Option<i64>,
}

struct PlaybackSessionContext {
    session: Arc<lyra_packager::Session>,
    playlist: String,
    segment_count: usize,
}

pub fn get_hls_router() -> Router<AppState> {
    let mut router = Router::new()
        .route("/v/{token}/index.m3u8", get(get_stream_playlist))
        .route("/v/{token}/{name}", get(get_segment));

    #[cfg(debug_assertions)]
    {
        router = router.layer(CorsLayer::permissive());
    }

    router
}

pub(crate) async fn mint_playback_url(
    pool: &sea_orm::DatabaseConnection,
    playback_registry: &PlaybackRegistry,
    auth: &RequestAuth,
    input: MintPlaybackUrlInput,
) -> Result<MintPlaybackUrlResult, async_graphql::Error> {
    let file = ensure_file_access(pool, auth, &input.file_id).await?;
    let (probe, keyframes) = load_probe_data_for_playback_options(pool, &input.file_id)
        .await
        .map_err(|error| async_graphql::Error::new(error.to_string()))?;
    let video_stream_index = probe
        .get_video_stream()
        .map(|stream| stream.index)
        .ok_or_else(|| async_graphql::Error::new("File has no playable video stream"))?;
    let resumable = normalize_selection(
        &probe,
        keyframes.as_ref(),
        &file.id,
        &input.video_rendition_id,
        video_stream_index,
        input.audio_stream_index,
        &input.audio_rendition_id,
    )?;

    detach_previous_player_session(playback_registry, &input.player_id)
        .await
        .map_err(|error| async_graphql::Error::new(error.to_string()))?;

    let session_options = build_session_options(pool, &resumable)
        .await
        .map_err(|error| async_graphql::Error::new(error.to_string()))?;
    let session = playback_registry
        .sessions
        .create(session_options)
        .await
        .map_err(|error| async_graphql::Error::new(error.to_string()))?;
    playback_registry
        .sessions
        .attach_player(session.id(), input.player_id.clone())
        .await
        .map_err(|error| async_graphql::Error::new(error.to_string()))?;
    playback_registry
        .player_sessions
        .lock()
        .await
        .insert(input.player_id.clone(), session.id().to_string());

    let token = sign(
        PlaybackTokenPayload {
            session_id: session.id().to_string(),
            player_id: input.player_id,
            resumable,
        },
        Duration::from_secs(6 * 60 * 60),
    )
    .map_err(|error| async_graphql::Error::new(error.to_string()))?;

    Ok(MintPlaybackUrlResult {
        url: format!("/api/hls/v/{token}/index.m3u8"),
        packager_id: session.id().to_string(),
    })
}

async fn get_stream_playlist(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Response, (StatusCode, &'static str)> {
    let (_expires_in, payload) = verify::<PlaybackTokenPayload>(&token)
        .map_err(|_| (StatusCode::NOT_FOUND, "stream not found"))?;
    let session = get_or_create_session_from_payload(&state, &payload)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "stream not found"))?;

    let mut response = Response::new(Body::from(rewrite_playlist(&token, &session.playlist)));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.apple.mpegurl"),
    );
    Ok(response)
}

async fn get_segment(
    State(state): State<AppState>,
    Path((token, name)): Path<(String, String)>,
    Query(query): Query<SegmentQuery>,
) -> Result<Response, (StatusCode, &'static str)> {
    let (_expires_in, payload) = verify::<PlaybackTokenPayload>(&token)
        .map_err(|_| (StatusCode::NOT_FOUND, "segment not found"))?;
    let session_context = get_or_create_session_from_payload(&state, &payload)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "segment generation failed",
            )
        })?;

    let _ = query.start_pts;

    let path =
        match parse_segment_name(&name) {
            Some(segment_index) => {
                if segment_index >= session_context.segment_count {
                    return Err((StatusCode::NOT_FOUND, "segment not found"));
                }
                session_context
                    .session
                    .get_segment(segment_index)
                    .await
                    .map_err(|_| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "segment generation failed",
                        )
                    })?
            }
            None if name == "init.mp4" => session_context
                .session
                .get_init_segment()
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "segment generation failed",
                    )
                })?,
            None => return Err((StatusCode::NOT_FOUND, "segment not found")),
        };

    let file = fs::File::open(&path)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "segment not found"))?;
    let body = Body::from_stream(ReaderStream::new(file));
    let mut response = Response::new(body);
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("video/mp4"));
    Ok(response)
}

async fn get_or_create_session_from_payload(
    state: &AppState,
    payload: &PlaybackTokenPayload,
) -> anyhow::Result<PlaybackSessionContext> {
    let session_options = build_session_options(&state.pool, &payload.resumable).await?;
    let playlist = build_playlist(&session_options)?;
    let session = state
        .playback_registry
        .sessions
        .get_or_create(&payload.session_id, session_options)
        .await?;

    ensure_player_binding(
        &state.playback_registry,
        &payload.session_id,
        &payload.player_id,
    )
    .await?;

    Ok(PlaybackSessionContext {
        session,
        playlist: playlist.playlist,
        segment_count: playlist.segment_count,
    })
}

async fn ensure_player_binding(
    playback_registry: &PlaybackRegistry,
    session_id: &str,
    player_id: &str,
) -> anyhow::Result<()> {
    let mut player_sessions = playback_registry.player_sessions.lock().await;
    let current = player_sessions.get(player_id).cloned();
    if let Some(existing) = current.as_deref() {
        if existing != session_id {
            bail!("player {player_id} is already bound to a different playback session");
        }
    } else {
        player_sessions.insert(player_id.to_string(), session_id.to_string());
    }
    drop(player_sessions);

    playback_registry
        .sessions
        .attach_player(session_id, player_id.to_string())
        .await?;
    Ok(())
}

async fn detach_previous_player_session(
    playback_registry: &PlaybackRegistry,
    player_id: &str,
) -> anyhow::Result<()> {
    let previous = playback_registry
        .player_sessions
        .lock()
        .await
        .remove(player_id);
    if let Some(previous_session_id) = previous {
        playback_registry
            .sessions
            .detach_player(&previous_session_id, player_id)
            .await?;
    }
    Ok(())
}

pub(crate) async fn ensure_file_access(
    pool: &sea_orm::DatabaseConnection,
    auth: &RequestAuth,
    file_id: &str,
) -> Result<files::Model, async_graphql::Error> {
    let (file, library) = files::Entity::find_by_id(file_id)
        .find_also_related(libraries::Entity)
        .one(pool)
        .await
        .map_err(|error| async_graphql::Error::new(error.to_string()))?
        .ok_or_else(|| async_graphql::Error::new("File not found"))?;

    if file.unavailable_at.is_some() {
        return Err(async_graphql::Error::new("File is unavailable"));
    }

    let library = library.ok_or_else(|| async_graphql::Error::new("Library not found"))?;
    ensure_library_access(pool, auth, &library.id)
        .await
        .map_err(|_| async_graphql::Error::new("File not found"))?;
    Ok(file)
}

pub(crate) async fn load_file_and_path(
    pool: &sea_orm::DatabaseConnection,
    file_id: &str,
) -> anyhow::Result<(files::Model, PathBuf)> {
    let (file, library) = files::Entity::find_by_id(file_id)
        .find_also_related(libraries::Entity)
        .one(pool)
        .await?
        .context("file not found")?;
    let library = library.context("library not found")?;
    Ok((
        file.clone(),
        PathBuf::from(library.path).join(file.relative_path),
    ))
}

pub(crate) async fn load_probe_data_for_playback_options(
    pool: &sea_orm::DatabaseConnection,
    file_id: &str,
) -> anyhow::Result<(ProbeData, Option<VideoKeyframes>)> {
    let mut probe = file_analysis::load_cached_probe(pool, file_id).await?;
    let keyframes = file_analysis::load_cached_keyframes(pool, file_id).await?;

    if probe.is_none() {
        let file = files::Entity::find_by_id(file_id)
            .one(pool)
            .await?
            .context("file disappeared before playback analysis job")?;
        jobs::try_run_job(pool, &FileProbeJob, file, ON_DEMAND_JOB_TIMEOUT).await?;
        probe = file_analysis::load_cached_probe(pool, file_id).await?;
    }

    Ok((
        probe.context("playback analysis finished without storing probe data")?,
        keyframes,
    ))
}

async fn load_session_analysis(
    pool: &sea_orm::DatabaseConnection,
    file_id: &str,
    video_stream_index: u32,
    video_profile_id: &str,
) -> anyhow::Result<(ProbeData, Option<VideoKeyframes>)> {
    let video_profile = video_profile(video_profile_id)
        .with_context(|| format!("unknown video profile {video_profile_id}"))?;
    let mut probe = file_analysis::load_cached_probe(pool, file_id).await?;
    if probe.is_none() {
        let file = files::Entity::find_by_id(file_id)
            .one(pool)
            .await?
            .context("file disappeared before playback analysis job")?;
        jobs::try_run_job(pool, &FileProbeJob, file, ON_DEMAND_JOB_TIMEOUT).await?;
        probe = file_analysis::load_cached_probe(pool, file_id).await?;
    }
    let probe = probe.context("playback analysis finished without storing probe data")?;
    let video_stream = probe
        .video_stream(video_stream_index)
        .with_context(|| format!("video stream {video_stream_index} not found in probe data"))?;
    let compatibility = video_profile
        .compatible_with(video_stream)
        .with_context(|| {
            format!(
                "video profile {video_profile_id} is incompatible with stream {video_stream_index}"
            )
        })?;
    let mut keyframes = file_analysis::load_cached_keyframes(pool, file_id).await?;

    if compatibility == Compatibility::KeyframeAligned && keyframes.is_none() {
        let file = files::Entity::find_by_id(file_id)
            .one(pool)
            .await?
            .context("file disappeared before playback analysis job")?;
        jobs::try_run_job(pool, &FileProbeJob, file, ON_DEMAND_JOB_TIMEOUT).await?;
        keyframes = file_analysis::load_cached_keyframes(pool, file_id).await?;
    }

    if compatibility == Compatibility::KeyframeAligned {
        let keyframes_ref = keyframes
            .as_ref()
            .context("playback analysis finished without storing keyframes")?;
        anyhow::ensure!(
            keyframes_ref.video_stream_index == video_stream_index,
            "cached keyframes are for stream {}, not requested stream {}",
            keyframes_ref.video_stream_index,
            video_stream_index
        );
    }

    Ok((probe, keyframes))
}

fn normalize_selection(
    probe: &ProbeData,
    keyframes: Option<&VideoKeyframes>,
    file_id: &str,
    video_rendition_id: &str,
    video_stream_index: u32,
    audio_stream_index: i32,
    audio_rendition_id: &str,
) -> Result<SessionResumable, async_graphql::Error> {
    let video_profile_id = api_video_rendition_to_profile_id(video_rendition_id)
        .ok_or_else(|| async_graphql::Error::new("Unsupported video rendition"))?;
    let video_profile = video_profile(video_profile_id)
        .ok_or_else(|| async_graphql::Error::new("Unsupported video rendition"))?;
    let video_stream = probe
        .video_stream(video_stream_index)
        .ok_or_else(|| async_graphql::Error::new("File has no playable video stream"))?;
    let compatibility = video_profile
        .compatible_with(video_stream)
        .ok_or_else(|| async_graphql::Error::new("Unsupported video rendition"))?;
    if compatibility == Compatibility::KeyframeAligned
        && keyframes.is_some_and(|value| value.video_stream_index != video_stream_index)
    {
        return Err(async_graphql::Error::new(
            "Cached keyframes do not match the selected video stream",
        ));
    }

    let audio_profile_id = api_audio_rendition_to_profile_id(audio_rendition_id)
        .ok_or_else(|| async_graphql::Error::new("Unsupported audio rendition"))?;
    let audio_profile = audio_profile(audio_profile_id)
        .ok_or_else(|| async_graphql::Error::new("Unsupported audio rendition"))?;
    let audio_stream_index = u32::try_from(audio_stream_index)
        .map_err(|_| async_graphql::Error::new("Invalid audio stream index"))?;
    let audio_stream = probe
        .stream(audio_stream_index)
        .ok_or_else(|| async_graphql::Error::new("Invalid audio stream index"))?;
    if audio_profile.compatible_with(audio_stream).is_none() {
        return Err(async_graphql::Error::new("Unsupported audio rendition"));
    }

    Ok(SessionResumable {
        file_id: file_id.to_string(),
        video_stream_index,
        video_profile_id: video_profile_id.to_string(),
        audio_stream_index: Some(audio_stream_index),
        audio_profile_id: Some(audio_profile_id.to_string()),
    })
}

async fn build_session_options(
    pool: &sea_orm::DatabaseConnection,
    resumable: &SessionResumable,
) -> anyhow::Result<SessionOptions> {
    let (_, file_path) = load_file_and_path(pool, &resumable.file_id).await?;
    let (probe, keyframes) = load_session_analysis(
        pool,
        &resumable.file_id,
        resumable.video_stream_index,
        &resumable.video_profile_id,
    )
    .await?;

    Ok(SessionOptions {
        spec: SessionSpec {
            file_path,
            video: VideoProfileSelection {
                stream_index: resumable.video_stream_index,
                profile_id: resumable.video_profile_id.clone(),
            },
            audio: match (
                resumable.audio_stream_index,
                resumable.audio_profile_id.as_ref(),
            ) {
                (Some(stream_index), Some(profile_id)) => Some(AudioProfileSelection {
                    stream_index,
                    profile_id: profile_id.clone(),
                }),
                _ => None,
            },
        },
        probe,
        keyframes,
    })
}

struct PlaylistData {
    playlist: String,
    segment_count: usize,
}

fn build_playlist(options: &SessionOptions) -> anyhow::Result<PlaylistData> {
    let video_stream = options
        .probe
        .video_stream(options.spec.video.stream_index)
        .context("video stream not found for playlist")?;
    let (time_base_num, time_base_den) = video_stream
        .time_base()
        .context("video stream is missing time_base metadata")?;
    let duration_secs = options
        .probe
        .duration_secs
        .context("file duration is required for HLS playlist generation")?;
    let total_duration_pts = seconds_to_pts(duration_secs, time_base_num, time_base_den);
    anyhow::ensure!(
        total_duration_pts > 0,
        "file duration is required for HLS playback"
    );

    let video_profile = video_profile(&options.spec.video.profile_id)
        .with_context(|| format!("unknown video profile {}", options.spec.video.profile_id))?;
    let compatibility = video_profile
        .compatible_with(video_stream)
        .with_context(|| {
            format!(
                "video profile {} is incompatible with stream {}",
                video_profile.id(),
                video_stream.index
            )
        })?;

    let segment_start_pts = match compatibility {
        Compatibility::KeyframeAligned => options
            .keyframes
            .as_ref()
            .context("keyframe-aligned playlist requires keyframes")?
            .segment_start_pts(Duration::from_secs(TARGET_SEGMENT_SECONDS)),
        Compatibility::Fixed => {
            let segment_len_pts =
                seconds_to_pts(TARGET_SEGMENT_SECONDS as f64, time_base_num, time_base_den);
            anyhow::ensure!(segment_len_pts > 0, "segment duration must be positive");
            let mut starts = Vec::new();
            let mut current = 0_i64;
            while current < total_duration_pts {
                starts.push(current);
                current += segment_len_pts;
            }
            if starts.is_empty() {
                starts.push(0);
            }
            starts
        }
    };

    let playlist = create_fmp4_hls_playlist_from_segment_starts_pts(
        &segment_start_pts,
        total_duration_pts,
        time_base_num,
        time_base_den,
        "",
        "",
    )
    .map_err(anyhow::Error::msg)?;

    Ok(PlaylistData {
        segment_count: segment_start_pts.len(),
        playlist,
    })
}

fn api_video_rendition_to_profile_id(rendition_id: &str) -> Option<&'static str> {
    match rendition_id {
        "original" => Some("copy"),
        "h264" => Some("h264"),
        _ => None,
    }
}

fn api_audio_rendition_to_profile_id(rendition_id: &str) -> Option<&'static str> {
    match rendition_id {
        "aac" => Some("aac"),
        _ => None,
    }
}

pub(crate) fn rewrite_playlist(token: &str, playlist: &str) -> String {
    playlist
        .lines()
        .map(|line| {
            if line.contains("URI=\"init.mp4\"") {
                line.replace(
                    "URI=\"init.mp4\"",
                    &format!("URI=\"/api/hls/v/{token}/init.mp4\""),
                )
            } else if line.contains(".m4s") {
                format!("/api/hls/v/{token}/{line}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_segment_name(name: &str) -> Option<usize> {
    name.strip_suffix(".m4s")?.parse().ok()
}
