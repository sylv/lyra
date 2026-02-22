use crate::config::get_config;
use anyhow::Context;
use image::{GenericImageView, ImageFormat};
use sha2::{Digest, Sha256};
use std::{io::ErrorKind, path::PathBuf};
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};

#[derive(Debug, Clone)]
pub struct PreparedImage {
    pub hash_sha256: String,
    pub size_bytes: i64,
    pub mime_type: String,
    pub width: i64,
    pub height: i64,
    pub extension: &'static str,
}

pub fn hash_bytes_sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn prepare_image(bytes: &[u8]) -> anyhow::Result<PreparedImage> {
    let format = image::guess_format(bytes).context("failed to guess image format")?;
    let (mime_type, extension) = match format {
        ImageFormat::Jpeg => ("image/jpeg", "jpg"),
        ImageFormat::WebP => ("image/webp", "webp"),
        other => return Err(anyhow::anyhow!("unsupported image format: {other:?}")),
    };

    let decoded =
        image::load_from_memory_with_format(bytes, format).context("failed to decode image")?;
    let (width, height) = decoded.dimensions();

    Ok(PreparedImage {
        hash_sha256: hash_bytes_sha256_hex(bytes),
        size_bytes: i64::try_from(bytes.len()).context("image byte length exceeds i64")?,
        mime_type: mime_type.to_string(),
        width: i64::from(width),
        height: i64::from(height),
        extension,
    })
}

pub fn extension_for_mime(mime_type: &str) -> anyhow::Result<&'static str> {
    match mime_type {
        "image/jpeg" => Ok("jpg"),
        "image/webp" => Ok("webp"),
        _ => Err(anyhow::anyhow!("unsupported mime type: {mime_type}")),
    }
}

pub fn get_asset_output_path(hash_sha256: &str, extension: &str) -> anyhow::Result<PathBuf> {
    let mut chars = hash_sha256.chars();
    let first = chars
        .next()
        .ok_or_else(|| anyhow::anyhow!("invalid hash, missing first character"))?
        .to_string();
    let second = chars
        .next()
        .ok_or_else(|| anyhow::anyhow!("invalid hash, missing second character"))?
        .to_string();

    Ok(get_config()
        .get_asset_store_dir()
        .join(first)
        .join(second)
        .join(format!("{hash_sha256}.{extension}")))
}

pub fn get_asset_output_path_from_mime(
    hash_sha256: &str,
    mime_type: &str,
) -> anyhow::Result<PathBuf> {
    let extension = extension_for_mime(mime_type)?;
    get_asset_output_path(hash_sha256, extension)
}

pub fn get_transformed_cache_path(
    hash_sha256: &str,
    width: Option<u32>,
    height: Option<u32>,
) -> PathBuf {
    let width = width.unwrap_or(0);
    let height = height.unwrap_or(0);
    get_config()
        .get_image_dir()
        .join(format!("{hash_sha256}_{width}x{height}.jpg"))
}

pub async fn persist_image_bytes(bytes: &[u8], image: &PreparedImage) -> anyhow::Result<PathBuf> {
    let output_path = get_asset_output_path(&image.hash_sha256, image.extension)?;

    if let Some(parent) = output_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    if tokio::fs::try_exists(&output_path).await? {
        return Ok(output_path);
    }

    let tmp_dir = get_config().get_tmp_dir().join("assets");
    tokio::fs::create_dir_all(&tmp_dir).await?;
    let tmp_path = tmp_dir.join(format!(
        "{}.{}.tmp",
        image.hash_sha256,
        rand::random::<u64>()
    ));

    let mut file = BufWriter::new(File::create(&tmp_path).await?);
    file.write_all(bytes).await?;
    file.flush().await?;
    file.get_ref().sync_all().await?;

    match tokio::fs::rename(&tmp_path, &output_path).await {
        Ok(()) => Ok(output_path),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            tokio::fs::remove_file(&tmp_path).await?;
            Ok(output_path)
        }
        Err(error) => {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            Err(error.into())
        }
    }
}
