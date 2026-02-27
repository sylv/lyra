use crate::{
    entities::{file_probe, files, jobs as jobs_entity},
    jobs::{JobHandler, handlers::shared},
    json_encoding,
};
use anyhow::Context;
use lyra_ffprobe::{FfprobeOutput, paths::get_ffprobe_path, probe_output};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    sea_query::OnConflict, sea_query::Query,
};
use std::{path::Path, path::PathBuf};

#[derive(Debug, Default)]
pub struct FileFfprobeJob;

#[async_trait::async_trait]
impl JobHandler for FileFfprobeJob {
    fn job_type(&self) -> jobs_entity::JobType {
        jobs_entity::JobType::FileExtractFfprobe
    }

    fn filter_condition(&self) -> Option<Condition> {
        Some(
            Condition::all().add(
                files::Column::Id.not_in_subquery(
                    Query::select()
                        .column(file_probe::Column::FileId)
                        .from(file_probe::Entity)
                        .to_owned(),
                ),
            ),
        )
    }

    async fn execute(&self, pool: &DatabaseConnection, file_id: i64) -> anyhow::Result<()> {
        let Some(ctx) = shared::load_job_file_context(pool, file_id, self.job_type()).await? else {
            return Ok(());
        };

        extract_and_store_ffprobe(pool, ctx.file.id, &ctx.file_path).await?;
        Ok(())
    }

    async fn cleanup(&self, pool: &DatabaseConnection, file_id: i64) -> anyhow::Result<()> {
        file_probe::Entity::delete_by_id(file_id).exec(pool).await?;
        Ok(())
    }
}

pub(crate) async fn extract_and_store_ffprobe(
    pool: &DatabaseConnection,
    file_id: i64,
    file_path: &Path,
) -> anyhow::Result<FfprobeOutput> {
    let ffprobe_bin = PathBuf::from(get_ffprobe_path()?);
    let input = file_path.to_path_buf();
    let ffprobe_output = tokio::task::spawn_blocking(move || probe_output(&ffprobe_bin, &input))
        .await
        .context("ffprobe probe task panicked")??;

    upsert_ffprobe_output(pool, file_id, &ffprobe_output).await?;
    Ok(ffprobe_output)
}

async fn upsert_ffprobe_output(
    pool: &DatabaseConnection,
    file_id: i64,
    ffprobe_output: &FfprobeOutput,
) -> anyhow::Result<()> {
    let primary_video = ffprobe_output
        .streams
        .iter()
        .find(|stream| stream.codec_type.as_deref() == Some("video"));
    let primary_audio = ffprobe_output
        .streams
        .iter()
        .find(|stream| stream.codec_type.as_deref() == Some("audio"));
    let has_subtitles = ffprobe_output
        .streams
        .iter()
        .any(|stream| stream.codec_type.as_deref() == Some("subtitle"));

    let duration_s = ffprobe_output
        .format
        .as_ref()
        .and_then(|format| format.duration.as_deref())
        .and_then(|value| value.parse::<f64>().ok())
        .map(|value| value.max(0.0).floor() as i64);

    let fps = primary_video
        .and_then(|stream| stream.avg_frame_rate.as_deref())
        .and_then(parse_frame_rate)
        .or_else(|| {
            primary_video
                .and_then(|stream| stream.r_frame_rate.as_deref())
                .and_then(parse_frame_rate)
        });

    let streams = json_encoding::encode_json_zstd(ffprobe_output)
        .context("failed to encode ffprobe payload")?;
    let now = chrono::Utc::now().timestamp();

    file_probe::Entity::insert(file_probe::ActiveModel {
        file_id: Set(file_id),
        duration_s: Set(duration_s),
        height: Set(primary_video.and_then(|stream| stream.height)),
        width: Set(primary_video.and_then(|stream| stream.width)),
        fps: Set(fps),
        video_codec: Set(primary_video.and_then(|stream| stream.codec_name.clone())),
        video_bitrate: Set(primary_video
            .and_then(|stream| stream.bit_rate.as_deref())
            .and_then(|value| value.parse::<i64>().ok())),
        audio_codec: Set(primary_audio.and_then(|stream| stream.codec_name.clone())),
        audio_bitrate: Set(primary_audio
            .and_then(|stream| stream.bit_rate.as_deref())
            .and_then(|value| value.parse::<i64>().ok())),
        audio_channels: Set(primary_audio.and_then(|stream| stream.channels)),
        has_subtitles: Set(has_subtitles),
        streams: Set(Some(streams)),
        generated_at: Set(now),
    })
    .on_conflict(
        OnConflict::column(file_probe::Column::FileId)
            .update_columns([
                file_probe::Column::DurationS,
                file_probe::Column::Height,
                file_probe::Column::Width,
                file_probe::Column::Fps,
                file_probe::Column::VideoCodec,
                file_probe::Column::VideoBitrate,
                file_probe::Column::AudioCodec,
                file_probe::Column::AudioBitrate,
                file_probe::Column::AudioChannels,
                file_probe::Column::HasSubtitles,
                file_probe::Column::Streams,
                file_probe::Column::GeneratedAt,
            ])
            .to_owned(),
    )
    .exec(pool)
    .await?;

    Ok(())
}

fn parse_frame_rate(value: &str) -> Option<f64> {
    if value.contains('/') {
        let mut parts = value.split('/');
        let num = parts.next().and_then(|part| part.parse::<f64>().ok())?;
        let den = parts.next().and_then(|part| part.parse::<f64>().ok())?;
        if parts.next().is_some() || den <= 0.0 {
            return None;
        }
        let rate = num / den;
        return if rate.is_finite() && rate > 0.0 {
            Some(rate)
        } else {
            None
        };
    }

    value.parse::<f64>().ok().and_then(|rate| {
        if rate.is_finite() && rate > 0.0 {
            Some(rate)
        } else {
            None
        }
    })
}
