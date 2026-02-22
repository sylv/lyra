use crate::{
    AppState, RequestAuth,
    assets::{download_asset_to_local, storage},
    entities::assets as assets_entity,
    error::AppError,
};
use axum::{
    Router,
    body::Body,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::get,
};
use image::{GenericImageView, ImageOutputFormat, imageops::FilterType};
use reqwest::header::{CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE};
use sea_orm::EntityTrait;
use serde::Deserialize;
use std::{io::Cursor, path::Path as FsPath};
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_util::io::ReaderStream;

#[derive(Debug, Deserialize, Clone)]
pub struct TranscodeParams {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

pub fn get_assets_router() -> Router<AppState> {
    Router::new().route("/{asset_id}", get(get_asset))
}

async fn get_asset(
    _auth: RequestAuth,
    State(state): State<AppState>,
    Path(asset_id): Path<i64>,
    Query(params): Query<TranscodeParams>,
) -> Result<Response, AppError> {
    let mut asset = assets_entity::Entity::find_by_id(asset_id)
        .one(&state.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("asset not found"))?;

    if asset.deleted_at.is_some() {
        return Err(anyhow::anyhow!("asset has been deleted").into());
    }

    if asset.hash_sha256.is_none() {
        asset = download_asset_to_local(&state.pool, &asset).await?;
    }

    serve_asset(&asset, &params).await
}

async fn serve_asset(
    asset: &assets_entity::Model,
    params: &TranscodeParams,
) -> Result<Response, AppError> {
    let hash_sha256 = asset
        .hash_sha256
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("asset has no local hash"))?;
    let mime_type = asset
        .mime_type
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("asset has no mime_type"))?;

    let original_path = storage::get_asset_output_path_from_mime(hash_sha256, mime_type)?;
    if params.width.is_none() && params.height.is_none() {
        return stream_file(&original_path, mime_type).await;
    }

    let transformed_path =
        storage::get_transformed_cache_path(hash_sha256, params.width, params.height);
    if !tokio::fs::try_exists(&transformed_path).await? {
        let original_bytes = tokio::fs::read(&original_path)
            .await
            .map_err(|err| anyhow::anyhow!("failed reading source asset file: {err}"))?;
        let image = image::load_from_memory(&original_bytes)?;

        let (original_width, original_height) = image.dimensions();
        let target_width = params.width.unwrap_or(original_width);
        let target_height = params.height.unwrap_or(original_height);

        let resized = image.resize(target_width, target_height, FilterType::Lanczos3);

        let mut cursor = Cursor::new(Vec::new());
        resized.write_to(&mut cursor, ImageOutputFormat::Jpeg(80))?;
        let bytes = cursor.into_inner();

        let mut file = File::create(&transformed_path).await?;
        file.write_all(&bytes).await?;
        file.sync_all().await?;
    }

    stream_file(&transformed_path, "image/jpeg").await
}

async fn stream_file(path: &FsPath, content_type: &str) -> Result<Response, AppError> {
    let file = File::open(path).await?;
    let metadata = file.metadata().await?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = [
        (CONTENT_LENGTH, metadata.len().to_string()),
        (CONTENT_TYPE, content_type.to_string()),
        (
            CACHE_CONTROL,
            "public, max-age=31536000, immutable".to_string(),
        ),
    ];

    Ok((headers, body).into_response())
}
