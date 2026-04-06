use crate::{
    AppState,
    auth::{RequestAuth, ensure_library_access},
    config::get_config,
    entities::{files, libraries},
    file_analysis,
    jobs::{self, FileProbeJob},
};
use axum::{
    Router,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode, header},
    response::Response,
    routing::get,
};
use lyra_packager::{BuildOptions, Package, Session, build_package, get_profiles};
use sea_orm::EntityTrait;
use serde::Deserialize;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::fs;
use tokio_util::io::ReaderStream;
#[cfg(debug_assertions)]
use tower_http::cors::CorsLayer;

const ON_DEMAND_JOB_TIMEOUT: Duration = Duration::from_secs(120);
const JOB_LOCK_GRACE: Duration = Duration::from_mins(5);

#[derive(Deserialize)]
struct SegmentQuery {
    #[serde(rename = "startPts")]
    start_pts: Option<i64>,
}

pub fn get_hls_router() -> Router<AppState> {
    let mut router = Router::new()
        .route("/stream/{file_id}/master.m3u8", get(get_master_playlist))
        .route(
            "/stream/{file_id}/{stream_id}/{profile_id}/index.m3u8",
            get(get_stream_playlist),
        )
        .route(
            "/stream/{file_id}/{stream_id}/{profile_id}/segment/{name}",
            get(get_segment),
        );

    #[cfg(debug_assertions)]
    {
        router = router.layer(CorsLayer::permissive());
    }

    router
}

async fn get_master_playlist(
    auth: RequestAuth,
    State(state): State<AppState>,
    Path(file_id): Path<String>,
) -> Result<Response, (StatusCode, &'static str)> {
    let _guard = state.job_semaphore.push_lock(JOB_LOCK_GRACE).await;
    let packager_state = get_or_build_packager_state(&state, &auth, file_id.clone()).await?;
    let playlist = rewrite_playlist_for_file(packager_state.master_playlist(), &file_id);

    let mut response = Response::new(Body::from(playlist));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.apple.mpegurl"),
    );
    Ok(response)
}

async fn get_stream_playlist(
    auth: RequestAuth,
    State(state): State<AppState>,
    Path((file_id, stream_id, profile_id)): Path<(String, u32, String)>,
) -> Result<Response, (StatusCode, &'static str)> {
    let _guard = state.job_semaphore.push_lock(JOB_LOCK_GRACE).await;
    let packager_state = get_or_build_packager_state(&state, &auth, file_id.clone()).await?;
    let session = packager_state
        .get_session(stream_id, &profile_id)
        .ok_or((StatusCode::NOT_FOUND, "stream profile not found"))?;

    let playlist = rewrite_playlist_for_file(session.playlist(), &file_id);
    let mut response = Response::new(Body::from(playlist));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.apple.mpegurl"),
    );
    Ok(response)
}

async fn get_segment(
    auth: RequestAuth,
    State(state): State<AppState>,
    Path((file_id, stream_id, profile_id, name)): Path<(String, u32, String, String)>,
    Query(query): Query<SegmentQuery>,
) -> Result<Response, (StatusCode, &'static str)> {
    let _guard = state.job_semaphore.push_lock(JOB_LOCK_GRACE).await;
    let packager_state = get_or_build_packager_state(&state, &auth, file_id).await?;
    let session = packager_state
        .get_session(stream_id, &profile_id)
        .ok_or((StatusCode::NOT_FOUND, "stream profile not found"))?
        .clone();

    let segment_index =
        Session::parse_segment_name(&name).ok_or((StatusCode::NOT_FOUND, "segment not found"))?;

    if segment_index >= 0 {
        if !session.has_segment(segment_index) {
            return Err((StatusCode::NOT_FOUND, "segment not found"));
        }
        session
            .ensure_segment(segment_index, query.start_pts)
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
        .wait_for_segment_file(&name, Duration::from_secs(10))
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "segment not found"))?;

    let file = fs::File::open(&path)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "segment not found"))?;
    let body = Body::from_stream(ReaderStream::new(file));
    let mut response = Response::new(body);
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(session.segment_content_type()),
    );
    Ok(response)
}

async fn get_or_build_packager_state(
    state: &AppState,
    auth: &RequestAuth,
    file_id: String,
) -> Result<Arc<Package>, (StatusCode, &'static str)> {
    let file_path = resolve_file_path(state, auth, &file_id).await?;

    if let Some(existing) = state
        .packager_states
        .lock()
        .await
        .get(file_id.as_str())
        .cloned()
    {
        return Ok(existing);
    }
    let mut generated_probe = false;
    let mut generated_keyframes = false;
    let mut probe = file_analysis::load_cached_probe(&state.pool, &file_id)
        .await
        .map_err(|err| {
            tracing::error!(
                file_id,
                error = %err,
                "failed to load cached probe data"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to prepare stream metadata",
            )
        })?;
    let mut keyframes_pts = file_analysis::load_cached_keyframes(&state.pool, &file_id)
        .await
        .map_err(|err| {
            tracing::error!(
                file_id,
                error = %err,
                "failed to load cached keyframe data"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to prepare stream metadata",
            )
        })?;

    if probe.is_none() || keyframes_pts.is_none() {
        generated_probe = probe.is_none();
        generated_keyframes = keyframes_pts.is_none();

        let file = files::Entity::find_by_id(file_id.clone())
            .one(&state.pool)
            .await
            .map_err(|err| {
                tracing::error!(
                    file_id,
                    error = %err,
                    "failed to load file before playback analysis job"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to prepare stream metadata",
                )
            })?
            .ok_or_else(|| {
                tracing::error!(file_id, "file disappeared before playback analysis job");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to prepare stream metadata",
                )
            })?;

        jobs::try_run_job(&state.pool, &FileProbeJob, file, ON_DEMAND_JOB_TIMEOUT)
            .await
            .map_err(|err| {
                tracing::error!(
                    file_id,
                    error = %err,
                    "failed to generate playback analysis on-demand"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to prepare stream metadata",
                )
            })?;

        probe = file_analysis::load_cached_probe(&state.pool, &file_id)
            .await
            .map_err(|err| {
                tracing::error!(
                    file_id,
                    error = %err,
                    "failed to load cached probe data after playback analysis job"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to prepare stream metadata",
                )
            })?;
        keyframes_pts = file_analysis::load_cached_keyframes(&state.pool, &file_id)
            .await
            .map_err(|err| {
                tracing::error!(
                    file_id,
                    error = %err,
                    "failed to load cached keyframe data after playback analysis job"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to prepare stream metadata",
                )
            })?;
    }

    let probe = probe.ok_or_else(|| {
        tracing::error!(
            file_id,
            "playback analysis finished without storing probe data"
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to prepare stream metadata",
        )
    })?;

    let keyframes_pts = keyframes_pts.ok_or_else(|| {
        tracing::error!(
            file_id,
            "playback analysis finished without storing keyframes"
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to prepare stream metadata",
        )
    })?;

    if generated_probe || generated_keyframes {
        tracing::warn!(
            file_id,
            generated_probe,
            generated_keyframes,
            "playback requested before background media analysis completed; generating missing playback analysis on-demand"
        );
    }

    let profiles = get_profiles();
    let build_options = BuildOptions {
        transcode_cache_dir: get_config().get_transcode_cache_dir().join(file_id.clone()),
    };

    let packager_state = Arc::new(
        build_package(
            &file_path,
            &profiles,
            &build_options,
            &probe,
            &keyframes_pts,
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
    auth: &RequestAuth,
    file_id: &str,
) -> Result<PathBuf, (StatusCode, &'static str)> {
    let (file, library) = files::Entity::find_by_id(file_id)
        .find_also_related(libraries::Entity)
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
    ensure_library_access(&state.pool, auth, &library.id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "file not found"))?;
    Ok(PathBuf::from(library.path).join(&file.relative_path))
}

fn rewrite_playlist_for_file(playlist: &str, file_id: &str) -> String {
    playlist.replace("/stream/", &format!("/api/hls/stream/{file_id}/"))
}
