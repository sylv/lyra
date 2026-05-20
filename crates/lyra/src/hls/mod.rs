use crate::{
    AppState,
    auth::{RequestAuth, ensure_library_access},
    entities::{files, libraries},
    jobs,
    media::{self, FileProbeJob},
    signer::{sign, verify},
};
use anyhow::Context;
use axum::{
    Router,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode, header},
    response::Response,
    routing::get,
};
use lyra_packager::{
    AudioProfileSelection, Compatibility, SessionOptions, SessionSpec, VideoProfileSelection,
    audio_profile, playlist::create_fmp4_hls_playlist_from_segment_starts_pts,
    playlist::seconds_to_pts, video_profile,
};
use lyra_probe::{ProbeData, VideoKeyframes};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::fs;
use tokio_util::io::ReaderStream;
#[cfg(debug_assertions)]
use tower_http::cors::CorsLayer;

const ON_DEMAND_JOB_TIMEOUT: Duration = Duration::from_secs(120);
const TARGET_SEGMENT_SECONDS: u64 = 6;
pub(crate) const AUDIO_NONE_PAIR_ID: &str = "none";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackTokenPayload {
    pub file_id: String,
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
        .route(
            "/{file_id}/{token}/{video_pair_id}/{audio_pair_id}/index.m3u8",
            get(get_stream_playlist),
        )
        .route(
            "/{file_id}/{token}/{video_pair_id}/{audio_pair_id}/{name}",
            get(get_segment),
        );

    #[cfg(debug_assertions)]
    {
        router = router.layer(CorsLayer::permissive());
    }

    router
}

pub(crate) fn sign_playback_url_template(file_id: &str) -> anyhow::Result<String> {
    let token = sign(
        PlaybackTokenPayload {
            file_id: file_id.to_string(),
        },
        Duration::from_secs(6 * 60 * 60),
    )?;

    Ok(format!(
        "/api/hls/{file_id}/{token}/{{VIDEO_PAIR_ID}}/{{AUDIO_PAIR_ID}}/index.m3u8"
    ))
}

async fn get_stream_playlist(
    State(state): State<AppState>,
    Path((file_id, token, video_pair_id, audio_pair_id)): Path<(String, String, String, String)>,
) -> Result<Response, (StatusCode, &'static str)> {
    verify_playback_token(&token, &file_id)
        .map_err(|_| (StatusCode::NOT_FOUND, "stream not found"))?;
    let session =
        get_or_create_session_for_selection(&state, &file_id, &video_pair_id, &audio_pair_id)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "stream not found"))?;

    let mut response = Response::new(Body::from(rewrite_playlist(
        &file_id,
        &token,
        &video_pair_id,
        &audio_pair_id,
        &session.playlist,
    )));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.apple.mpegurl"),
    );
    Ok(response)
}

async fn get_segment(
    State(state): State<AppState>,
    Path((file_id, token, video_pair_id, audio_pair_id, name)): Path<(
        String,
        String,
        String,
        String,
        String,
    )>,
    Query(query): Query<SegmentQuery>,
) -> Result<Response, (StatusCode, &'static str)> {
    verify_playback_token(&token, &file_id)
        .map_err(|_| (StatusCode::NOT_FOUND, "segment not found"))?;
    let session_context =
        get_or_create_session_for_selection(&state, &file_id, &video_pair_id, &audio_pair_id)
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
    file_id: &str,
    video_pair_id: &str,
    audio_pair_id: &str,
) -> anyhow::Result<PlaybackSessionContext> {
    let (session_id, session_options) =
        build_session_options_for_selection(&state.pool, file_id, video_pair_id, audio_pair_id)
            .await?;
    let playlist = build_playlist(&session_options)?;
    let session = state
        .packager_sessions
        .get_or_create(&session_id, session_options)
        .await?;

    Ok(PlaybackSessionContext {
        session,
        playlist: playlist.playlist,
        segment_count: playlist.segment_count,
    })
}

async fn get_or_create_session_for_selection(
    state: &AppState,
    file_id: &str,
    video_pair_id: &str,
    audio_pair_id: &str,
) -> anyhow::Result<PlaybackSessionContext> {
    get_or_create_session_from_payload(state, file_id, video_pair_id, audio_pair_id).await
}

fn verify_playback_token(token: &str, file_id: &str) -> anyhow::Result<()> {
    let (_expires_in, payload) = verify::<PlaybackTokenPayload>(token)?;
    anyhow::ensure!(payload.file_id == file_id, "stream not found");
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
    anyhow::ensure!(file.unavailable_at.is_none(), "file is unavailable");
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
    let mut probe = media::load_cached_probe(pool, file_id).await?;
    let keyframes = media::load_cached_keyframes(pool, file_id).await?;

    if probe.is_none() || needs_default_video_keyframes(probe.as_ref(), keyframes.as_ref()) {
        let file = files::Entity::find_by_id(file_id)
            .one(pool)
            .await?
            .context("file disappeared before playback analysis job")?;
        jobs::try_run_job(pool, &FileProbeJob, file, ON_DEMAND_JOB_TIMEOUT).await?;
        probe = media::load_cached_probe(pool, file_id).await?;
        return Ok((
            probe.context("playback analysis finished without storing probe data")?,
            media::load_cached_keyframes(pool, file_id).await?,
        ));
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
    let mut probe = media::load_cached_probe(pool, file_id).await?;
    if probe.is_none() {
        let file = files::Entity::find_by_id(file_id)
            .one(pool)
            .await?
            .context("file disappeared before playback analysis job")?;
        jobs::try_run_job(pool, &FileProbeJob, file, ON_DEMAND_JOB_TIMEOUT).await?;
        probe = media::load_cached_probe(pool, file_id).await?;
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
    let mut keyframes = media::load_cached_keyframes(pool, file_id).await?;

    if compatibility == Compatibility::KeyframeAligned && keyframes.is_none() {
        let file = files::Entity::find_by_id(file_id)
            .one(pool)
            .await?
            .context("file disappeared before playback analysis job")?;
        jobs::try_run_job(pool, &FileProbeJob, file, ON_DEMAND_JOB_TIMEOUT).await?;
        keyframes = media::load_cached_keyframes(pool, file_id).await?;
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

fn needs_default_video_keyframes(
    probe: Option<&ProbeData>,
    keyframes: Option<&VideoKeyframes>,
) -> bool {
    let Some(probe) = probe else {
        return false;
    };
    let Some(video_stream) = probe.get_video_stream() else {
        return false;
    };
    video_profile("copy")
        .and_then(|profile| profile.compatible_with(video_stream))
        .is_some_and(|compatibility| {
            compatibility == Compatibility::KeyframeAligned
                && keyframes
                    .is_none_or(|keyframes| keyframes.video_stream_index != video_stream.index)
        })
}

fn normalize_selection(
    probe: &ProbeData,
    keyframes: Option<&VideoKeyframes>,
    video_pair_id: &str,
    audio_pair_id: &str,
) -> anyhow::Result<(VideoProfileSelection, Option<AudioProfileSelection>)> {
    let video_selection = parse_video_pair_id(video_pair_id).context("invalid video pair")?;
    let video_profile = video_profile(&video_selection.profile_id).context("invalid video pair")?;
    let video_stream = probe
        .video_stream(video_selection.stream_index)
        .context("invalid video pair")?;
    let compatibility = video_profile
        .compatible_with(video_stream)
        .context("invalid video pair")?;
    if compatibility == Compatibility::KeyframeAligned
        && keyframes.is_none_or(|value| value.video_stream_index != video_selection.stream_index)
    {
        anyhow::bail!("invalid video pair");
    }

    let audio_selection = if audio_pair_id == AUDIO_NONE_PAIR_ID {
        None
    } else {
        let audio_selection = parse_audio_pair_id(audio_pair_id).context("invalid audio pair")?;
        let audio_profile =
            audio_profile(&audio_selection.profile_id).context("invalid audio pair")?;
        let audio_stream = probe
            .stream(audio_selection.stream_index)
            .filter(|stream| stream.kind() == lyra_probe::StreamKind::Audio)
            .context("invalid audio pair")?;
        anyhow::ensure!(
            audio_profile.compatible_with(audio_stream).is_some(),
            "invalid audio pair"
        );
        Some(audio_selection)
    };

    Ok((video_selection, audio_selection))
}

async fn build_session_options_for_selection(
    pool: &sea_orm::DatabaseConnection,
    file_id: &str,
    video_pair_id: &str,
    audio_pair_id: &str,
) -> anyhow::Result<(String, SessionOptions)> {
    let (_, file_path) = load_file_and_path(pool, file_id).await?;
    let (probe_for_selection, keyframes_for_selection) =
        load_probe_data_for_playback_options(pool, file_id).await?;
    let (video_selection, audio_selection) = normalize_selection(
        &probe_for_selection,
        keyframes_for_selection.as_ref(),
        video_pair_id,
        audio_pair_id,
    )?;
    let (probe, keyframes) = load_session_analysis(
        pool,
        file_id,
        video_selection.stream_index,
        &video_selection.profile_id,
    )
    .await?;

    Ok((
        package_session_id(file_id, video_pair_id, audio_pair_id),
        SessionOptions {
            spec: SessionSpec {
                file_path,
                video: video_selection,
                audio: audio_selection,
            },
            probe,
            keyframes,
        },
    ))
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

pub(crate) fn video_pair_id(stream_index: u32, profile_id: &str) -> String {
    format!("v{stream_index}-{profile_id}")
}

pub(crate) fn audio_pair_id(stream_index: u32, profile_id: &str) -> String {
    format!("a{stream_index}-{profile_id}")
}

fn parse_video_pair_id(pair_id: &str) -> Option<VideoProfileSelection> {
    parse_pair_id(pair_id, 'v').map(|(stream_index, profile_id)| VideoProfileSelection {
        stream_index,
        profile_id: profile_id.to_string(),
    })
}

fn parse_audio_pair_id(pair_id: &str) -> Option<AudioProfileSelection> {
    parse_pair_id(pair_id, 'a').map(|(stream_index, profile_id)| AudioProfileSelection {
        stream_index,
        profile_id: profile_id.to_string(),
    })
}

fn parse_pair_id(pair_id: &str, prefix: char) -> Option<(u32, &str)> {
    let pair_id = pair_id.strip_prefix(prefix)?;
    let (stream_index, profile_id) = pair_id.split_once('-')?;
    Some((stream_index.parse().ok()?, profile_id))
}

fn package_session_id(file_id: &str, video_pair_id: &str, audio_pair_id: &str) -> String {
    format!("pp-{file_id}-{video_pair_id}-{audio_pair_id}")
}

pub(crate) fn rewrite_playlist(
    file_id: &str,
    token: &str,
    video_pair_id: &str,
    audio_pair_id: &str,
    playlist: &str,
) -> String {
    playlist
        .lines()
        .map(|line| {
            if line.contains("URI=\"init.mp4\"") {
                line.replace(
                    "URI=\"init.mp4\"",
                    &format!(
                        "URI=\"/api/hls/{file_id}/{token}/{video_pair_id}/{audio_pair_id}/init.mp4\""
                    ),
                )
            } else if line.contains(".m4s") {
                format!(
                    "/api/hls/{file_id}/{token}/{video_pair_id}/{audio_pair_id}/{line}"
                )
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
