use crate::{
    assets,
    entities::{
        assets::AssetKind,
        file_subtitles::{self, SubtitleKind, SubtitleSource},
    },
    ids,
    subtitles::{
        extension_for_subtitle_kind, mime_type_for_subtitle_kind, subtitle_disposition_bits,
        subtitle_kind_from_stream,
    },
};
use anyhow::{Context, Result, bail};
use lyra_bitsubconvert::{
    BitmapSubtitleKind, BitmapToWebVttOptions, ExtractedSubtitleInput,
    convert_extracted_bitmap_subtitles_to_webvtt,
};
use lyra_packager::state::build_track_display_name;
use lyra_probe::{ProbeData, Stream, get_ffmpeg_path};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
};
use std::{
    collections::HashMap,
    io::Cursor,
    path::{Path, PathBuf},
};
use tar::Builder;
use tokio::{fs, process::Command};

#[derive(Debug, Clone)]
pub struct SubtitleDescriptor {
    pub kind: SubtitleKind,
    pub stream_index: i64,
    pub language_bcp47: Option<String>,
    pub display_name: String,
    pub disposition_bits: i64,
}

pub fn subtitle_descriptor_from_stream(stream: &Stream) -> Option<SubtitleDescriptor> {
    let kind = subtitle_kind_from_stream(stream)?;
    let fallback = format!("Subtitle {}", stream.index + 1);

    Some(SubtitleDescriptor {
        kind,
        stream_index: i64::from(stream.index),
        language_bcp47: stream.language_bcp47.clone(),
        display_name: build_track_display_name(
            stream.language_bcp47.as_deref(),
            stream.original_title.as_deref(),
            &fallback,
            stream.is_forced(),
            stream.is_hearing_impaired(),
            stream.is_commentary(),
        ),
        disposition_bits: subtitle_disposition_bits(stream.disposition),
    })
}

pub async fn extract_subtitle_bytes_batch<'a, I>(
    input_video_path: &Path,
    subtitles: I,
) -> Result<HashMap<i64, Vec<u8>>>
where
    I: IntoIterator<Item = (&'a Stream, &'a SubtitleDescriptor)>,
{
    let subtitles: Vec<_> = subtitles.into_iter().collect();
    if subtitles.is_empty() {
        return Ok(HashMap::new());
    }

    let temp_dir = tempfile::tempdir().context("failed to create temporary subtitle directory")?;
    let planned_outputs = subtitles
        .iter()
        .map(|(stream, descriptor)| {
            (
                i64::from(stream.index),
                descriptor.kind,
                temp_dir.path().join(format!(
                    "stream_{}.{}",
                    descriptor.stream_index,
                    extension_for_subtitle_kind(descriptor.kind)
                )),
            )
        })
        .collect::<Vec<_>>();

    // Feed ffmpeg one output per subtitle stream so demuxing only reads the input once.
    let mut command = Command::new(get_ffmpeg_path());
    command
        .args(["-nostdin", "-hide_banner", "-loglevel", "error", "-y", "-i"])
        .arg(input_video_path);

    for ((stream, _descriptor), (_stream_index, _kind, output_path)) in
        subtitles.iter().zip(planned_outputs.iter())
    {
        command
            .args(["-map", &format!("0:{}", stream.index), "-c", "copy"])
            .arg(output_path);
    }

    let ffmpeg_output = command
        .output()
        .await
        .context("failed to run ffmpeg for subtitle extraction")?;

    if !ffmpeg_output.status.success() {
        let stderr = String::from_utf8_lossy(&ffmpeg_output.stderr);
        bail!("ffmpeg subtitle extraction failed: {stderr}");
    }

    let mut extracted = HashMap::with_capacity(planned_outputs.len());
    for (stream_index, kind, output_path) in planned_outputs {
        let bytes = match kind {
            SubtitleKind::VobSub => archive_vobsub_pair(&output_path)?,
            _ => fs::read(&output_path).await.with_context(|| {
                format!(
                    "failed to read extracted subtitle {}",
                    output_path.display()
                )
            })?,
        };
        extracted.insert(stream_index, bytes);
    }

    Ok(extracted)
}

fn archive_vobsub_pair(idx_path: &Path) -> Result<Vec<u8>> {
    let sub_path = idx_path.with_extension("sub");
    let mut builder = Builder::new(Vec::new());
    builder
        .append_path_with_name(
            idx_path,
            idx_path.file_name().context("missing idx filename")?,
        )
        .context("failed to add idx file to vobsub archive")?;
    builder
        .append_path_with_name(
            &sub_path,
            sub_path.file_name().context("missing sub filename")?,
        )
        .context("failed to add sub file to vobsub archive")?;
    builder
        .finish()
        .context("failed to finalize vobsub archive")?;
    builder
        .into_inner()
        .context("failed to read vobsub archive")
}

pub async fn upsert_extracted_subtitle<C: ConnectionTrait>(
    db: &C,
    file_id: &str,
    descriptor: &SubtitleDescriptor,
    bytes: &[u8],
    last_seen_at: i64,
) -> Result<file_subtitles::Model> {
    let asset = assets::create_local_file_asset_from_bytes(
        db,
        bytes,
        mime_type_for_subtitle_kind(descriptor.kind),
        AssetKind::Subtitle,
    )
    .await?;

    let model = file_subtitles::ActiveModel {
        id: Set(ids::generate_ulid()),
        file_id: Set(file_id.to_string()),
        asset_id: Set(asset.id),
        derived_from_subtitle_id: Set(None),
        kind: Set(descriptor.kind),
        stream_index: Set(descriptor.stream_index),
        source: Set(SubtitleSource::Extracted),
        language_bcp47: Set(descriptor.language_bcp47.clone()),
        display_name: Set(Some(descriptor.display_name.clone())),
        disposition_bits: Set(descriptor.disposition_bits),
        last_seen_at: Set(last_seen_at),
        processed_at: Set((descriptor.kind == SubtitleKind::Vtt).then_some(last_seen_at)),
        created_at: Set(last_seen_at),
        updated_at: Set(last_seen_at),
    }
    .insert(db)
    .await?;

    Ok(model)
}

pub async fn refresh_extracted_subtitle_metadata<C: ConnectionTrait>(
    db: &C,
    row: &file_subtitles::Model,
    descriptor: &SubtitleDescriptor,
    last_seen_at: i64,
) -> Result<file_subtitles::Model> {
    let mut active: file_subtitles::ActiveModel = row.clone().into();
    active.kind = Set(descriptor.kind);
    active.language_bcp47 = Set(descriptor.language_bcp47.clone());
    active.display_name = Set(Some(descriptor.display_name.clone()));
    active.disposition_bits = Set(descriptor.disposition_bits);
    active.last_seen_at = Set(last_seen_at);
    active.updated_at = Set(last_seen_at);
    if descriptor.kind == SubtitleKind::Vtt && row.processed_at.is_none() {
        active.processed_at = Set(Some(last_seen_at));
    }
    Ok(active.update(db).await?)
}

pub async fn refresh_derived_subtitles_last_seen<C: ConnectionTrait>(
    db: &C,
    source_subtitle_id: &str,
    descriptor: &SubtitleDescriptor,
    last_seen_at: i64,
) -> Result<()> {
    let derived_rows = file_subtitles::Entity::find()
        .filter(file_subtitles::Column::DerivedFromSubtitleId.eq(source_subtitle_id))
        .all(db)
        .await?;

    for row in derived_rows {
        let mut active: file_subtitles::ActiveModel = row.into();
        active.language_bcp47 = Set(descriptor.language_bcp47.clone());
        active.display_name = Set(Some(descriptor.display_name.clone()));
        active.disposition_bits = Set(descriptor.disposition_bits);
        active.last_seen_at = Set(last_seen_at);
        active.updated_at = Set(last_seen_at);
        active.update(db).await?;
    }

    Ok(())
}

pub async fn convert_text_subtitle_to_vtt(
    input_path: &Path,
    output_path: &Path,
) -> Result<Vec<u8>> {
    let ffmpeg_output = Command::new(get_ffmpeg_path())
        .args(["-nostdin", "-hide_banner", "-loglevel", "error", "-y", "-i"])
        .arg(input_path)
        .args(["-f", "webvtt"])
        .arg(output_path)
        .output()
        .await
        .context("failed to run ffmpeg for subtitle conversion")?;

    if !ffmpeg_output.status.success() {
        let stderr = String::from_utf8_lossy(&ffmpeg_output.stderr);
        bail!("ffmpeg subtitle conversion failed: {stderr}");
    }

    fs::read(output_path).await.with_context(|| {
        format!(
            "failed to read converted subtitle {}",
            output_path.display()
        )
    })
}

pub async fn convert_bitmap_subtitle_to_vtt(
    row: &file_subtitles::Model,
    asset_bytes: &[u8],
    probe_data: &ProbeData,
    model_dir: &Path,
) -> Result<Vec<u8>> {
    let video_stream = probe_data
        .get_video_stream()
        .context("probe data missing video stream for subtitle OCR")?;
    let width = video_stream
        .width()
        .context("probe data missing video width")?;
    let height = video_stream
        .height()
        .context("probe data missing video height")?;
    let temp_dir = tempfile::tempdir().context("failed to create OCR temp dir")?;
    let input = write_bitmap_input(temp_dir.path(), row.kind, asset_bytes)?;
    let converted = convert_extracted_bitmap_subtitles_to_webvtt(
        bitmap_kind_for_subtitle_kind(row.kind)?,
        input,
        (width, height),
        row.language_bcp47.clone(),
        model_dir,
        BitmapToWebVttOptions::default(),
    )
    .await?;
    Ok(converted.webvtt.into_bytes())
}

fn write_bitmap_input(
    temp_dir: &Path,
    kind: SubtitleKind,
    bytes: &[u8],
) -> Result<ExtractedSubtitleInput> {
    match kind {
        SubtitleKind::Pgs => {
            let sup_path = temp_dir.join("subtitle.sup");
            std::fs::write(&sup_path, bytes)
                .with_context(|| format!("failed to write {}", sup_path.display()))?;
            Ok(ExtractedSubtitleInput::Pgs { sup_path })
        }
        SubtitleKind::VobSub => {
            let mut archive = tar::Archive::new(Cursor::new(bytes));
            archive
                .unpack(temp_dir)
                .with_context(|| format!("failed to unpack {}", temp_dir.display()))?;
            let idx_path = temp_dir.join("stream_0.idx");
            let sub_path = temp_dir.join("stream_0.sub");
            let resolved_idx = if idx_path.exists() {
                idx_path
            } else {
                find_file_with_extension(temp_dir, "idx")?
            };
            let resolved_sub = if sub_path.exists() {
                sub_path
            } else {
                find_file_with_extension(temp_dir, "sub")?
            };
            Ok(ExtractedSubtitleInput::VobSub {
                idx_path: resolved_idx,
                sub_path: resolved_sub,
            })
        }
        other => bail!("subtitle kind {other:?} is not bitmap"),
    }
}

fn find_file_with_extension(dir: &Path, extension: &str) -> Result<PathBuf> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some(extension) {
            return Ok(path);
        }
    }
    bail!("failed to find .{extension} file in {}", dir.display())
}

pub fn bitmap_kind_for_subtitle_kind(kind: SubtitleKind) -> Result<BitmapSubtitleKind> {
    match kind {
        SubtitleKind::Pgs => Ok(BitmapSubtitleKind::Pgs),
        SubtitleKind::VobSub => Ok(BitmapSubtitleKind::VobSub),
        other => bail!("subtitle kind {other:?} is not bitmap"),
    }
}
