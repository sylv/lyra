use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use image::{GenericImageView, ImageFormat};
use tokio::process::Command;

pub const MAX_DIMENSION_PX: u32 = 1200;
pub const WEBP_QUALITY: f32 = 72.0;
pub const THUMBNAIL_MIME_TYPE: &str = "image/webp";
pub const SCENE_THRESHOLD: f32 = 0.35;

#[derive(Clone, Debug)]
pub struct ThumbnailOptions {
    pub ffmpeg_bin: PathBuf,
    pub max_dimension_px: u32,
    pub webp_quality: f32,
}

impl Default for ThumbnailOptions {
    fn default() -> Self {
        Self {
            ffmpeg_bin: PathBuf::from("ffmpeg"),
            max_dimension_px: MAX_DIMENSION_PX,
            webp_quality: WEBP_QUALITY,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Thumbnail {
    pub image_bytes: Vec<u8>,
    pub mime_type: &'static str,
    pub width: u32,
    pub height: u32,
}

pub async fn generate_thumbnail(
    video_path: &Path,
    options: &ThumbnailOptions,
) -> anyhow::Result<Thumbnail> {
    let scale_filter = format!("scale={}:-2:flags=lanczos", options.max_dimension_px);
    let blackframe_filter = format!(
        "blackframe=amount=1:threshold=32,metadata=mode=select:key=lavfi.blackframe.pblack:value=90:function=less,{scale_filter}"
    );
    let scene_and_blackframe_filter =
        format!("select='gt(scene,{SCENE_THRESHOLD})',{blackframe_filter}");

    let image_bytes = match encode_webp(video_path, options, &scene_and_blackframe_filter).await {
        Ok(bytes) => bytes,
        Err(first_error) => {
            tracing::warn!(
                "thumbnail scene-based selection failed for {}: {first_error:#}",
                video_path.display()
            );

            match encode_webp(video_path, options, &blackframe_filter).await {
                Ok(bytes) => bytes,
                Err(second_error) => {
                    tracing::warn!(
                        "thumbnail blackframe-only selection failed for {}: {second_error:#}",
                        video_path.display()
                    );
                    encode_webp(video_path, options, &scale_filter).await?
                }
            }
        }
    };

    let (width, height) = output_dimensions(&image_bytes)?;

    Ok(Thumbnail {
        image_bytes,
        mime_type: THUMBNAIL_MIME_TYPE,
        width,
        height,
    })
}

async fn encode_webp(
    video_path: &Path,
    options: &ThumbnailOptions,
    filter: &str,
) -> anyhow::Result<Vec<u8>> {
    let output = Command::new(&options.ffmpeg_bin)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            &video_path.to_string_lossy(),
            "-map",
            "0:v:0",
            "-an",
            "-sn",
            "-dn",
            "-vf",
            filter,
            "-frames:v",
            "1",
            "-c:v",
            "libwebp",
            "-q:v",
            &options.webp_quality.to_string(),
            "-f",
            "webp",
            "pipe:1",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!(
            "ffmpeg failed to encode thumbnail webp with filter '{filter}': {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    if output.stdout.is_empty() {
        anyhow::bail!("ffmpeg returned no thumbnail frame with filter '{filter}'");
    }

    Ok(output.stdout)
}

fn output_dimensions(image_bytes: &[u8]) -> anyhow::Result<(u32, u32)> {
    let image = image::load_from_memory_with_format(image_bytes, ImageFormat::WebP)?;
    Ok(image.dimensions())
}
