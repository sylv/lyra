use crate::{
    config::get_config,
    entities::file_subtitles,
    ids,
    subtitles::{subtitle_disposition_bits, subtitle_path},
};
use anyhow::{Context, Result, bail};
use lyra_bitsubconvert::{
    BitmapSubtitleKind, BitmapToWebVttOptions, ExtractedSubtitleInput,
    convert_extracted_bitmap_subtitles_to_webvtt,
};
use lyra_probe::{Codec, Stream, StreamKind, get_ffmpeg_path};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::{Output, Stdio},
};
use tokio::{
    fs,
    io::{AsyncRead, AsyncReadExt},
    process::Command,
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct SubtitleDescriptor {
    pub source_stream_index: i64,
    pub language_bcp47: Option<String>,
    pub display_name: String,
    pub disposition_bits: i64,
}

pub fn subtitle_descriptor_from_stream(stream: &Stream) -> Option<SubtitleDescriptor> {
    if stream.kind() != StreamKind::Subtitle {
        return None;
    }

    let fallback = format!("Subtitle {}", stream.index + 1);
    Some(SubtitleDescriptor {
        source_stream_index: i64::from(stream.index),
        language_bcp47: stream.language_bcp47.clone(),
        display_name: stream.display_name.clone().unwrap_or(fallback),
        disposition_bits: subtitle_disposition_bits(stream.disposition),
    })
}

pub fn can_extract_webvtt_subtitle(stream: &Stream) -> bool {
    matches!(
        &stream.codec,
        Codec::SubtitleAss
            | Codec::SubtitleSubRip
            | Codec::SubtitleMovText
            | Codec::SubtitleText
            | Codec::SubtitleTtml
            | Codec::SubtitleWebVtt
            | Codec::SubtitlePgs
    )
}

pub async fn extract_webvtt_subtitle_batch<'a, I>(
    input_video_path: &Path,
    subtitles: I,
    cancellation_token: Option<&CancellationToken>,
) -> Result<Option<HashMap<i64, Vec<u8>>>>
where
    I: IntoIterator<Item = (&'a Stream, &'a SubtitleDescriptor)>,
{
    let subtitles = subtitles.into_iter().collect::<Vec<_>>();
    if subtitles.is_empty() {
        return Ok(Some(HashMap::new()));
    }

    let owned_cancellation_token;
    let cancellation_token = match cancellation_token {
        Some(token) => token,
        None => {
            owned_cancellation_token = CancellationToken::new();
            &owned_cancellation_token
        }
    };

    let temp_dir = tempfile::tempdir().context("failed to create temporary subtitle directory")?;
    let mut extracted = HashMap::with_capacity(subtitles.len());

    for (stream, descriptor) in subtitles {
        if cancellation_token.is_cancelled() {
            return Ok(None);
        }

        let output_path = temp_dir
            .path()
            .join(format!("stream_{}.vtt", descriptor.source_stream_index));
        match stream.codec {
            Codec::SubtitlePgs => {
                let sup_path = temp_dir
                    .path()
                    .join(format!("stream_{}.sup", descriptor.source_stream_index));
                let Some(output) = extract_pgs_stream_to_sup(
                    input_video_path,
                    stream.index,
                    &sup_path,
                    cancellation_token,
                )
                .await?
                else {
                    return Ok(None);
                };

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    bail!("ffmpeg PGS subtitle extraction failed: {stderr}");
                }

                let converted = convert_extracted_bitmap_subtitles_to_webvtt(
                    BitmapSubtitleKind::Pgs,
                    ExtractedSubtitleInput::Pgs { sup_path },
                    (0, 0),
                    descriptor.language_bcp47.clone(),
                    &get_config().data_dir.join("subtitle_models"),
                    BitmapToWebVttOptions::default(),
                )
                .await
                .context("failed to convert PGS subtitles to WebVTT")?;
                extracted.insert(
                    descriptor.source_stream_index,
                    converted.webvtt.into_bytes(),
                );
            }
            _ => {
                let Some(output) = convert_stream_to_webvtt(
                    input_video_path,
                    stream.index,
                    &output_path,
                    cancellation_token,
                )
                .await?
                else {
                    return Ok(None);
                };

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    bail!("ffmpeg subtitle conversion failed: {stderr}");
                }

                let bytes = fs::read(&output_path).await.with_context(|| {
                    format!(
                        "failed to read converted subtitle {}",
                        output_path.display()
                    )
                })?;
                extracted.insert(descriptor.source_stream_index, bytes);
            }
        }
    }

    Ok(Some(extracted))
}

pub async fn upsert_extracted_subtitle<C: ConnectionTrait>(
    db: &C,
    file_id: &str,
    existing: Option<&file_subtitles::Model>,
    descriptor: &SubtitleDescriptor,
    webvtt_bytes: &[u8],
    extracted_at: i64,
) -> Result<file_subtitles::Model> {
    let subtitle_id = existing
        .map(|row| row.id.clone())
        .unwrap_or_else(ids::generate_ulid);
    persist_webvtt(&subtitle_id, webvtt_bytes).await?;

    let model = if let Some(row) = existing {
        let mut active: file_subtitles::ActiveModel = row.clone().into();
        active.language_bcp47 = Set(descriptor.language_bcp47.clone());
        active.display_name = Set(Some(descriptor.display_name.clone()));
        active.disposition_bits = Set(descriptor.disposition_bits);
        active.extracted_at = Set(extracted_at);
        active.update(db).await?
    } else {
        file_subtitles::ActiveModel {
            id: Set(subtitle_id),
            file_id: Set(file_id.to_string()),
            source_stream_index: Set(descriptor.source_stream_index),
            language_bcp47: Set(descriptor.language_bcp47.clone()),
            display_name: Set(Some(descriptor.display_name.clone())),
            disposition_bits: Set(descriptor.disposition_bits),
            extracted_at: Set(extracted_at),
        }
        .insert(db)
        .await?
    };

    Ok(model)
}

async fn persist_webvtt(subtitle_id: &str, webvtt_bytes: &[u8]) -> Result<()> {
    let output_path = subtitle_path(subtitle_id);
    let output_dir = output_path
        .parent()
        .map(PathBuf::from)
        .context("subtitle path has no parent directory")?;
    fs::create_dir_all(&output_dir).await?;
    let compressed = zstd::encode_all(std::io::Cursor::new(webvtt_bytes), 10)
        .context("failed to zstd-compress subtitle")?;
    fs::write(output_path, compressed)
        .await
        .context("failed to write subtitle file")
}

async fn convert_stream_to_webvtt(
    input_video_path: &Path,
    stream_index: u32,
    output_path: &Path,
    cancellation_token: &CancellationToken,
) -> Result<Option<Output>> {
    let mut command = Command::new(get_ffmpeg_path());
    command
        .args(["-nostdin", "-hide_banner", "-loglevel", "error", "-y", "-i"])
        .arg(input_video_path)
        .args(["-map", &format!("0:{stream_index}"), "-f", "webvtt"])
        .arg(output_path);
    run_command_output(&mut command, cancellation_token)
        .await
        .context("failed to run ffmpeg for subtitle conversion")
}

async fn extract_pgs_stream_to_sup(
    input_video_path: &Path,
    stream_index: u32,
    output_path: &Path,
    cancellation_token: &CancellationToken,
) -> Result<Option<Output>> {
    let mut command = Command::new(get_ffmpeg_path());
    command
        .args(["-nostdin", "-hide_banner", "-loglevel", "error", "-y", "-i"])
        .arg(input_video_path)
        .args([
            "-map",
            &format!("0:{stream_index}"),
            "-c:s",
            "copy",
            "-f",
            "sup",
        ])
        .arg(output_path);
    run_command_output(&mut command, cancellation_token)
        .await
        .context("failed to run ffmpeg for PGS subtitle extraction")
}

async fn run_command_output(
    command: &mut Command,
    cancellation_token: &CancellationToken,
) -> Result<Option<Output>> {
    command.kill_on_drop(true);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
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
