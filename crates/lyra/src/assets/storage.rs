use crate::config::get_config;
use anyhow::Context;
use image::{GenericImageView, ImageFormat};
use regex::Regex;
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};

#[derive(Debug, Clone)]
pub struct PreparedImage {
    pub size_bytes: i64,
    pub mime_type: String,
    pub width: i64,
    pub height: i64,
}

pub fn prepare_image(bytes: &[u8]) -> anyhow::Result<PreparedImage> {
    if let Some(image) = prepare_svg_image(bytes)? {
        return Ok(image);
    }

    let format = image::guess_format(bytes).context("failed to guess image format")?;
    let mime_type = match format {
        ImageFormat::Jpeg => "image/jpeg",
        ImageFormat::Png => "image/png",
        ImageFormat::WebP => "image/webp",
        other => return Err(anyhow::anyhow!("unsupported image format: {other:?}")),
    };

    let decoded =
        image::load_from_memory_with_format(bytes, format).context("failed to decode image")?;
    let (width, height) = decoded.dimensions();

    Ok(PreparedImage {
        size_bytes: i64::try_from(bytes.len()).context("image byte length exceeds i64")?,
        mime_type: mime_type.to_string(),
        width: i64::from(width),
        height: i64::from(height),
    })
}

fn prepare_svg_image(bytes: &[u8]) -> anyhow::Result<Option<PreparedImage>> {
    let raw = match std::str::from_utf8(bytes) {
        Ok(raw) => raw,
        Err(_) => return Ok(None),
    };
    if !raw.contains("<svg") {
        return Ok(None);
    }

    let (width, height) = parse_svg_dimensions(raw)?;
    Ok(Some(PreparedImage {
        size_bytes: i64::try_from(bytes.len()).context("image byte length exceeds i64")?,
        mime_type: "image/svg+xml".to_string(),
        width,
        height,
    }))
}

fn parse_svg_dimensions(svg: &str) -> anyhow::Result<(i64, i64)> {
    let width_attr = capture_svg_attr(svg, "width")
        .and_then(|value| parse_svg_length(&value))
        .map(|value| value.round() as i64);
    let height_attr = capture_svg_attr(svg, "height")
        .and_then(|value| parse_svg_length(&value))
        .map(|value| value.round() as i64);

    if let (Some(width), Some(height)) = (width_attr, height_attr)
        && width > 0
        && height > 0
    {
        return Ok((width, height));
    }

    if let Some(view_box) = capture_svg_attr(svg, "viewBox") {
        let parts = view_box
            .split(|ch: char| ch.is_ascii_whitespace() || ch == ',')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        if parts.len() == 4 {
            let width = parts[2]
                .parse::<f64>()
                .ok()
                .map(|value| value.round() as i64);
            let height = parts[3]
                .parse::<f64>()
                .ok()
                .map(|value| value.round() as i64);
            if let (Some(width), Some(height)) = (width, height)
                && width > 0
                && height > 0
            {
                return Ok((width, height));
            }
        }
    }

    Err(anyhow::anyhow!(
        "unsupported svg without parseable width/height or viewBox"
    ))
}

fn capture_svg_attr(svg: &str, attr: &str) -> Option<String> {
    let pattern = format!(r#"{attr}\s*=\s*"([^"]+)""#);
    let regex = Regex::new(&pattern).ok()?;
    regex
        .captures(svg)
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str().to_string())
}

fn parse_svg_length(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    let numeric = trimmed.trim_end_matches(|ch: char| ch.is_ascii_alphabetic() || ch == '%');
    numeric.parse::<f64>().ok()
}

pub fn get_asset_output_path(image_id: &str) -> anyhow::Result<PathBuf> {
    Ok(get_config().get_asset_store_dir().join(image_id))
}

pub fn get_asset_output_path_from_mime(
    image_id: &str,
    _mime_type: &str,
) -> anyhow::Result<PathBuf> {
    get_asset_output_path(image_id)
}

pub fn get_asset_output_path_from_mime_and_encoding(
    image_id: &str,
    _mime_type: &str,
    _content_encoding: Option<&str>,
) -> anyhow::Result<PathBuf> {
    get_asset_output_path(image_id)
}

pub fn get_transformed_cache_path(
    image_id: &str,
    width: Option<u32>,
    height: Option<u32>,
    extension: &str,
) -> PathBuf {
    let width = width.unwrap_or(0);
    let height = height.unwrap_or(0);
    get_config()
        .get_image_dir()
        .join(format!("{image_id}_{width}x{height}.{extension}"))
}

pub async fn persist_bytes_atomically(output_path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    let parent = output_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("output path has no parent: {}", output_path.display()))?;
    tokio::fs::create_dir_all(parent).await?;

    if tokio::fs::try_exists(output_path).await? {
        return Ok(());
    }

    let tmp_path = parent.join(format!(
        ".{}.{}.tmp",
        output_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow::anyhow!("output path has invalid file name"))?,
        rand::random::<u64>()
    ));

    let mut file = BufWriter::new(File::create(&tmp_path).await?);
    file.write_all(bytes).await?;
    file.flush().await?;
    file.get_ref().sync_all().await?;

    match tokio::fs::rename(&tmp_path, output_path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            tokio::fs::remove_file(&tmp_path).await?;
            Ok(())
        }
        Err(error) => {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            Err(error.into())
        }
    }
}

pub async fn persist_image_bytes(
    image_id: &str,
    bytes: &[u8],
    _image: &PreparedImage,
) -> anyhow::Result<PathBuf> {
    let output_path = get_asset_output_path(image_id)?;
    persist_bytes_atomically(&output_path, bytes).await?;
    Ok(output_path)
}
