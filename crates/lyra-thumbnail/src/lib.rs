use std::{
    path::{Path, PathBuf},
    process::{Output, Stdio},
};

use image::{GenericImageView, ImageFormat};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

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
    cancellation_token: Option<&CancellationToken>,
) -> anyhow::Result<Option<Thumbnail>> {
    let owned_cancellation_token;
    let cancellation_token = match cancellation_token {
        Some(cancellation_token) => cancellation_token,
        None => {
            owned_cancellation_token = CancellationToken::new();
            &owned_cancellation_token
        }
    };
    let scale_filter = format!("scale={}:-2:flags=lanczos", options.max_dimension_px);
    let blackframe_filter = format!(
        "blackframe=amount=1:threshold=32,metadata=mode=select:key=lavfi.blackframe.pblack:value=90:function=less,{scale_filter}"
    );
    let scene_and_blackframe_filter =
        format!("select='gt(scene,{SCENE_THRESHOLD})',{blackframe_filter}");

    let image_bytes = match encode_webp(
        video_path,
        options,
        &scene_and_blackframe_filter,
        cancellation_token,
    )
    .await
    {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(None),
        Err(first_error) => {
            tracing::warn!(
                "thumbnail scene-based selection failed for {}: {first_error:#}",
                video_path.display()
            );

            match encode_webp(video_path, options, &blackframe_filter, cancellation_token).await {
                Ok(Some(bytes)) => bytes,
                Ok(None) => return Ok(None),
                Err(second_error) => {
                    tracing::warn!(
                        "thumbnail blackframe-only selection failed for {}: {second_error:#}",
                        video_path.display()
                    );
                    let Some(bytes) =
                        encode_webp(video_path, options, &scale_filter, cancellation_token).await?
                    else {
                        return Ok(None);
                    };
                    bytes
                }
            }
        }
    };

    let (width, height) = output_dimensions(&image_bytes)?;

    Ok(Some(Thumbnail {
        image_bytes,
        mime_type: THUMBNAIL_MIME_TYPE,
        width,
        height,
    }))
}

async fn encode_webp(
    video_path: &Path,
    options: &ThumbnailOptions,
    filter: &str,
    cancellation_token: &CancellationToken,
) -> anyhow::Result<Option<Vec<u8>>> {
    let output = run_ffmpeg_output(
        Command::new(&options.ffmpeg_bin)
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
            .stderr(Stdio::piped()),
        cancellation_token,
    )
    .await?;
    let Some(output) = output else {
        return Ok(None);
    };

    if !output.status.success() {
        anyhow::bail!(
            "ffmpeg failed to encode thumbnail webp with filter '{filter}': {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    if output.stdout.is_empty() {
        anyhow::bail!("ffmpeg returned no thumbnail frame with filter '{filter}'");
    }

    Ok(Some(output.stdout))
}

async fn run_ffmpeg_output(
    command: &mut Command,
    cancellation_token: &CancellationToken,
) -> anyhow::Result<Option<Output>> {
    command.kill_on_drop(true);
    let mut child = command.spawn()?;
    let stdout_task = spawn_pipe_reader(child.stdout.take());
    let stderr_task = spawn_pipe_reader(child.stderr.take());

    let status = tokio::select! {
        status = child.wait() => status?,
        _ = cancellation_token.cancelled() => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            let _ = stdout_task.await;
            let _ = stderr_task.await;
            return Ok(None);
        }
    };

    let stdout = stdout_task.await??;
    let stderr = stderr_task.await??;
    Ok(Some(Output {
        status,
        stdout,
        stderr,
    }))
}

fn spawn_pipe_reader<R>(pipe: Option<R>) -> JoinHandle<anyhow::Result<Vec<u8>>>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let Some(mut pipe) = pipe else {
            return Ok(Vec::new());
        };

        let mut output = Vec::new();
        pipe.read_to_end(&mut output).await?;
        Ok(output)
    })
}

fn output_dimensions(image_bytes: &[u8]) -> anyhow::Result<(u32, u32)> {
    let image = image::load_from_memory_with_format(image_bytes, ImageFormat::WebP)?;
    Ok(image.dimensions())
}
