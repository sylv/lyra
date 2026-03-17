use anyhow::Result;
use axum::{
    Router,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use lyra_packager::{
    BuildOptions, Package, Session, build_package_with_defaults, parse_single_input_path_arg,
};
use serde::Deserialize;
use std::{sync::Arc, time::Duration};
use tokio::fs;
use tokio_util::io::ReaderStream;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

#[derive(Deserialize)]
struct SegmentQuery {
    #[serde(rename = "startPts")]
    start_pts: Option<i64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let input = parse_single_input_path_arg()?;
    let options = BuildOptions {
        transcode_cache_dir: std::env::current_dir()?.join(".segments"),
    };
    let package = Arc::new(build_package_with_defaults(&input, &options)?);
    let app = build_router(package);

    let bind_addr = "0.0.0.0:4422";
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    tracing::info!(bind_addr = %bind_addr, "listening");
    axum::serve(listener, app).await?;
    Ok(())
}

fn build_router(package: Arc<Package>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/master.m3u8", get(master_handler))
        .route(
            "/stream/{stream_id}/{profile_id}/index.m3u8",
            get(stream_index_handler),
        )
        .route(
            "/stream/{stream_id}/{profile_id}/segment/{name}",
            get(stream_segment_handler),
        )
        .with_state(package)
        .layer(cors)
}

async fn master_handler(State(package): State<Arc<Package>>) -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")],
        package.master_playlist().to_string(),
    )
}

async fn stream_index_handler(
    State(package): State<Arc<Package>>,
    Path((stream_id, profile_id)): Path<(u32, String)>,
) -> Result<Response, StatusCode> {
    let session = package
        .get_session(stream_id, &profile_id)
        .ok_or(StatusCode::BAD_REQUEST)?;

    let mut response = Response::new(Body::from(session.playlist().to_string()));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.apple.mpegurl"),
    );
    Ok(response)
}

async fn stream_segment_handler(
    State(package): State<Arc<Package>>,
    Path((stream_id, profile_id, name)): Path<(u32, String, String)>,
    Query(query): Query<SegmentQuery>,
) -> Result<Response, StatusCode> {
    let session = package
        .get_session(stream_id, &profile_id)
        .ok_or(StatusCode::BAD_REQUEST)?
        .clone();

    let segment_index = Session::parse_segment_name(&name).ok_or(StatusCode::NOT_FOUND)?;
    if segment_index >= 0 {
        if !session.has_segment(segment_index) {
            return Err(StatusCode::NOT_FOUND);
        }
        session
            .ensure_segment(segment_index, query.start_pts)
            .await
            .map_err(|err| {
                tracing::warn!(error = %err, "segment request failed");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    } else {
        session.ensure_init().await.map_err(|err| {
            tracing::warn!(error = %err, "init request failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    let path = session
        .wait_for_segment_file(&name, Duration::from_secs(10))
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
