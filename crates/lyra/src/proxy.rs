use crate::{AppState, config::get_config, error::AppError};
use axum::{
    Router,
    body::Body,
    extract::{Path, Query},
    response::IntoResponse,
    routing::get,
};
use futures_util::TryStreamExt;
use image::{GenericImageView, ImageOutputFormat, imageops::FilterType};
use reqwest::header::{CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::{BufWriter, Cursor};
use std::path::PathBuf;
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_util::io::ReaderStream;

const ALLOWED_URLS: &[&str] = &["https://image.tmdb.org/t/p"];
const JPEG_HEADER: &[u8] = &[0xFF, 0xD8, 0xFF];
const MAX_SIZE: u32 = 1000;

// todo: it would be nice to support webp (.. and png), but the image crates webp support
// seems mediocre at best (or i'm missing something), but their jpeg support is excellent
#[derive(Debug, Deserialize)]
pub struct TranscodeParams {
    width: Option<u32>,
    height: Option<u32>,
}

async fn proxy_image(
    Path(url): Path<String>,
    Query(params): Query<TranscodeParams>,
) -> Result<impl IntoResponse, AppError> {
    if !is_url_allowed(&url) {
        return Err(anyhow::anyhow!("URL not allowed: {}", url).into());
    }

    let cache_path = get_cache_path(&url, None);
    let mut file_path = if cache_path.exists() {
        cache_path
    } else {
        download_and_cache_image(&url, &cache_path).await?;
        cache_path
    };

    if params.height.is_some() || params.width.is_some() {
        let transformed_path = get_cache_path(&url, Some((params.width, params.height)));
        file_path = if transformed_path.exists() {
            transformed_path
        } else {
            let image = image::open(file_path)?;

            let (original_width, original_height) = image.dimensions();
            let width = params.width.unwrap_or(original_width);
            let height = params.height.unwrap_or(original_height);

            let image = image.resize(width, height, FilterType::Lanczos3);

            let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
            image.write_to(&mut buffer, ImageOutputFormat::Jpeg(80))?;

            let bytes: Vec<u8> = buffer.into_inner()?.into_inner();

            let mut file = File::create(&transformed_path).await?;
            file.write_all(&bytes).await?;
            file.sync_all().await?;

            transformed_path
        };
    }

    let file = File::open(file_path).await?;
    let metadata = file.metadata().await?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let content_length = metadata.len().to_string();

    let headers = [
        (CONTENT_LENGTH, content_length),
        (CONTENT_TYPE, "image/jpeg".to_string()),
        (
            CACHE_CONTROL,
            "public, max-age=31536000, immutable".to_string(),
        ),
    ];

    Ok((headers, body).into_response())
}

async fn download_and_cache_image(url: &str, cache_path: &PathBuf) -> Result<(), AppError> {
    let response = reqwest::get(url).await?.error_for_status()?;

    let mut image_bytes = Vec::new();
    let mut stream = response.bytes_stream();
    let mut is_first_chunk = true;

    while let Some(bytes) = stream.try_next().await? {
        if is_first_chunk {
            if bytes.starts_with(JPEG_HEADER) {
                is_first_chunk = false;
            } else {
                return Err(anyhow::anyhow!("response is not a jpeg image").into());
            }
        }
        image_bytes.extend_from_slice(&bytes);
    }

    let mut image = image::load_from_memory(&image_bytes)?;
    if image.width() > MAX_SIZE || image.height() > MAX_SIZE {
        image = image.resize(MAX_SIZE, MAX_SIZE, FilterType::Lanczos3);
    }

    let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
    image.write_to(&mut buffer, ImageOutputFormat::Jpeg(80))?;
    let processed_bytes = buffer.into_inner()?.into_inner();

    let mut file = File::create(cache_path).await?;
    file.write_all(&processed_bytes).await?;
    file.sync_all().await?;

    Ok(())
}

fn is_url_allowed(url: &str) -> bool {
    for pattern in ALLOWED_URLS {
        if url.starts_with(pattern) {
            return true;
        }
    }

    false
}

fn get_cache_path(url: &str, size: Option<(Option<u32>, Option<u32>)>) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let hash = hasher.finalize();
    let suffix = match size {
        Some((width, height)) => format!("{}x{}", width.unwrap_or(0), height.unwrap_or(0)),
        None => "original".to_string(),
    };

    let file_name = format!("{:x}_{}.jpeg", hash, suffix);
    get_config().get_image_dir().join(file_name)
}

pub fn get_proxy_router() -> Router<AppState> {
    Router::new().route("/{url}", get(proxy_image))
}
