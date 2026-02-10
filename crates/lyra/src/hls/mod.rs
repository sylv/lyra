use crate::{
    AppState,
    auth::RequestAuth,
    config::get_config,
    entities::{file, library},
};
use axum::{
    Router,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode, header},
    response::Response,
    routing::get,
};
use lyra_packager::{
    KeyframePolicy, Package, Session, build_package_with_keyframe_policy, get_profiles,
};
use sea_orm::EntityTrait;
use serde::Deserialize;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::fs;
use tokio_util::io::ReaderStream;
#[cfg(debug_assertions)]
use tower_http::cors::CorsLayer;

#[derive(Deserialize)]
struct SegmentQuery {
    #[serde(rename = "startPts")]
    start_pts: Option<i64>,
}

pub fn get_hls_router() -> Router<AppState> {
    let mut router = Router::new()
        .route("/stream/{file_id}/index.m3u8", get(get_master_playlist))
        .route(
            "/stream/{file_id}/{stream_id}/{profile_id}/index.m3u8",
            get(get_stream_playlist),
        )
        .route(
            "/stream/{file_id}/{stream_id}/{profile_id}/segment/{name}",
            get(get_segment),
        )
        .route(
            "/stream/{file_id}/{stream_type}/{stream_idx}/{profile}/index.m3u8",
            get(get_stream_playlist_legacy),
        )
        .route(
            "/stream/{file_id}/{stream_type}/{stream_idx}/{profile}/{segment}",
            get(get_segment_legacy),
        );

    #[cfg(debug_assertions)]
    {
        router = router.layer(CorsLayer::permissive());
    }

    router
}

async fn get_master_playlist(
    _user: RequestAuth,
    State(state): State<AppState>,
    Path(file_id): Path<i64>,
) -> Result<Response, (StatusCode, &'static str)> {
    let packager_state = get_or_build_packager_state(&state, file_id).await?;
    let playlist = rewrite_playlist_for_file(packager_state.master_playlist(), file_id);

    let mut response = Response::new(Body::from(playlist));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.apple.mpegurl"),
    );
    Ok(response)
}

async fn get_stream_playlist(
    _user: RequestAuth,
    State(state): State<AppState>,
    Path((file_id, stream_id, profile_id)): Path<(i64, u32, String)>,
) -> Result<Response, (StatusCode, &'static str)> {
    stream_playlist_response(&state, file_id, stream_id, profile_id).await
}

async fn get_stream_playlist_legacy(
    _user: RequestAuth,
    State(state): State<AppState>,
    Path((file_id, _stream_type, stream_idx, profile)): Path<(i64, String, u32, String)>,
) -> Result<Response, (StatusCode, &'static str)> {
    let profile_id = map_legacy_profile(&profile)
        .ok_or((StatusCode::NOT_FOUND, "profile not found"))?
        .to_string();

    stream_playlist_response(&state, file_id, stream_idx, profile_id).await
}

async fn stream_playlist_response(
    state: &AppState,
    file_id: i64,
    stream_id: u32,
    profile_id: String,
) -> Result<Response, (StatusCode, &'static str)> {
    let packager_state = get_or_build_packager_state(state, file_id).await?;
    let session = packager_state
        .get_session(stream_id, &profile_id)
        .ok_or((StatusCode::NOT_FOUND, "stream profile not found"))?;

    let playlist = rewrite_playlist_for_file(session.playlist(), file_id);
    let mut response = Response::new(Body::from(playlist));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.apple.mpegurl"),
    );
    Ok(response)
}

async fn get_segment(
    _user: RequestAuth,
    State(state): State<AppState>,
    Path((file_id, stream_id, profile_id, name)): Path<(i64, u32, String, String)>,
    Query(query): Query<SegmentQuery>,
) -> Result<Response, (StatusCode, &'static str)> {
    segment_response(
        &state,
        file_id,
        stream_id,
        profile_id,
        name,
        query.start_pts,
    )
    .await
}

async fn get_segment_legacy(
    _user: RequestAuth,
    State(state): State<AppState>,
    Path((file_id, _stream_type, stream_idx, profile, segment)): Path<(
        i64,
        String,
        u32,
        String,
        String,
    )>,
    Query(query): Query<SegmentQuery>,
) -> Result<Response, (StatusCode, &'static str)> {
    let profile_id = map_legacy_profile(&profile)
        .ok_or((StatusCode::NOT_FOUND, "profile not found"))?
        .to_string();

    segment_response(
        &state,
        file_id,
        stream_idx,
        profile_id,
        segment,
        query.start_pts,
    )
    .await
}

async fn segment_response(
    state: &AppState,
    file_id: i64,
    stream_id: u32,
    profile_id: String,
    segment_name: String,
    requested_start_pts: Option<i64>,
) -> Result<Response, (StatusCode, &'static str)> {
    let packager_state = get_or_build_packager_state(state, file_id).await?;
    let session = packager_state
        .get_session(stream_id, &profile_id)
        .ok_or((StatusCode::NOT_FOUND, "stream profile not found"))?
        .clone();

    let segment_index = Session::parse_segment_name(&segment_name)
        .ok_or((StatusCode::NOT_FOUND, "segment not found"))?;

    if segment_index >= 0 {
        if !session.has_segment(segment_index) {
            return Err((StatusCode::NOT_FOUND, "segment not found"));
        }
        session
            .ensure_segment(segment_index, requested_start_pts)
            .await
            .map_err(|err| {
                tracing::warn!(error = %err, "segment request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "segment generation failed",
                )
            })?;
    } else {
        session.ensure_init().await.map_err(|err| {
            tracing::warn!(error = %err, "init request failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "segment generation failed",
            )
        })?;
    }

    let path = session
        .wait_for_segment_file(&segment_name, Duration::from_secs(10))
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "segment not found"))?;

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

async fn get_or_build_packager_state(
    state: &AppState,
    file_id: i64,
) -> Result<Arc<Package>, (StatusCode, &'static str)> {
    if let Some(existing) = state.packager_states.lock().await.get(&file_id).cloned() {
        return Ok(existing);
    }

    let file_path = resolve_file_path(state, file_id).await?;

    let profiles = get_profiles();
    let segments_root = get_config()
        .get_transcode_cache_dir()
        .join(file_id.to_string());

    let packager_state = Arc::new(
        build_package_with_keyframe_policy(
            &file_path,
            &profiles,
            Some(&segments_root),
            KeyframePolicy::CacheOnly,
        )
        .map_err(|err| {
            tracing::error!(file_id, error = %err, "failed to build packager state");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to build stream state",
            )
        })?,
    );

    let mut states = state.packager_states.lock().await;
    let entry = states
        .entry(file_id)
        .or_insert_with(|| packager_state.clone());
    Ok(entry.clone())
}

async fn resolve_file_path(
    state: &AppState,
    file_id: i64,
) -> Result<PathBuf, (StatusCode, &'static str)> {
    let (file, library) = file::Entity::find_by_id(file_id)
        .find_also_related(library::Entity)
        .one(&state.pool)
        .await
        .map_err(|err| {
            tracing::error!(error = ?err, "failed to find file");
            (StatusCode::INTERNAL_SERVER_ERROR, "error finding file")
        })?
        .ok_or((StatusCode::NOT_FOUND, "file not found"))?;

    if file.unavailable_at.is_some() {
        return Err((StatusCode::NOT_FOUND, "file is unavailable"));
    }

    let library = library.ok_or((StatusCode::INTERNAL_SERVER_ERROR, "library not found"))?;
    Ok(PathBuf::from(library.path).join(&file.relative_path))
}

fn map_legacy_profile(profile: &str) -> Option<&'static str> {
    match profile {
        "copy" => Some("video_copy"),
        "h264" => Some("video_h264"),
        "aac" => Some("audio_aac"),
        "video_copy" => Some("video_copy"),
        "video_h264" => Some("video_h264"),
        "audio_aac" => Some("audio_aac"),
        _ => None,
    }
}

fn rewrite_playlist_for_file(playlist: &str, file_id: i64) -> String {
    playlist.replace("/stream/", &format!("/api/hls/stream/{file_id}/"))
}
