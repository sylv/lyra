use crate::jobs::handlers::shared::get_job_file_path;
use crate::jobs::{Job, JobLease, JobOutcome};
use crate::{
    entities::{file_probe, files, jobs as jobs_entity},
    json_encoding,
};
use anyhow::Context;
use lyra_ffprobe::{
    FfprobeOutput, paths::get_ffprobe_path, probe_output, probe_streams_from_output,
};
use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Select,
    sea_query::{Expr, OnConflict, Query},
};
use std::path::Path;

#[derive(Debug, Default)]
pub struct FileFfprobeJob;

#[async_trait::async_trait]
impl Job for FileFfprobeJob {
    type Entity = files::Entity;
    type Model = files::Model;

    const JOB_KIND: jobs_entity::JobKind = jobs_entity::JobKind::FileExtractFfprobe;

    fn query(&self) -> Select<Self::Entity> {
        files::Entity::find()
            .filter(files::Column::UnavailableAt.is_null())
            .filter(
                Expr::col((files::Entity, files::Column::Id)).not_in_subquery(
                    Query::select()
                        .column(file_probe::Column::FileId)
                        .from(file_probe::Entity)
                        .to_owned(),
                ),
            )
            .order_by_asc(files::Column::Id)
    }

    fn target_id(&self, target: &Self::Model) -> String {
        target.id.clone()
    }

    async fn run(
        &self,
        db: &DatabaseConnection,
        file: Self::Model,
        ctx: &JobLease,
    ) -> anyhow::Result<JobOutcome> {
        let Some(file_path) = get_job_file_path(db, &file, Self::JOB_KIND).await? else {
            return Ok(JobOutcome::Complete);
        };

        let Some(_) = extract_and_store_ffprobe(db, &file.id, &file_path, ctx).await? else {
            return Ok(JobOutcome::Cancelled);
        };

        Ok(JobOutcome::Complete)
    }
}

pub(crate) async fn extract_and_store_ffprobe(
    db: &impl ConnectionTrait,
    file_id: &str,
    file_path: &Path,
    ctx: &JobLease,
) -> anyhow::Result<Option<FfprobeOutput>> {
    let ffprobe_output =
        probe_output(get_ffprobe_path()?, file_path, ctx.get_cancellation_token()).await?;
    let Some(ffprobe_output) = ffprobe_output else {
        return Ok(None);
    };

    upsert_ffprobe_output(db, file_id, &ffprobe_output).await?;
    Ok(Some(ffprobe_output))
}

async fn upsert_ffprobe_output(
    db: &impl ConnectionTrait,
    file_id: &str,
    ffprobe_output: &FfprobeOutput,
) -> anyhow::Result<()> {
    let probe = probe_streams_from_output(ffprobe_output)
        .with_context(|| format!("failed to normalize ffprobe output for file {file_id}"))?;
    let primary_video = probe
        .streams
        .iter()
        .find(|stream| matches!(stream.stream_type, lyra_ffprobe::StreamType::Video));
    let primary_audio = probe
        .streams
        .iter()
        .find(|stream| matches!(stream.stream_type, lyra_ffprobe::StreamType::Audio));
    let has_subtitles = probe
        .streams
        .iter()
        .any(|stream| matches!(stream.stream_type, lyra_ffprobe::StreamType::Subtitle));

    let duration_s = probe
        .duration_seconds
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
    let height = primary_video.and_then(|stream| stream.height.map(i64::from));
    let width = primary_video.and_then(|stream| stream.width.map(i64::from));
    let video_bitrate = primary_video
        .and_then(|stream| stream.bit_rate)
        .and_then(|value| i64::try_from(value).ok());
    let audio_bitrate = primary_audio
        .and_then(|stream| stream.bit_rate)
        .and_then(|value| i64::try_from(value).ok());
    let audio_channels = primary_audio.and_then(|stream| stream.channels.map(i64::from));

    file_probe::Entity::insert(file_probe::ActiveModel {
        file_id: Set(file_id.to_string()),
        duration_s: Set(duration_s),
        height: Set(height),
        width: Set(width),
        fps: Set(fps),
        video_codec: Set(primary_video.and_then(|stream| stream.codec_name.clone())),
        video_bitrate: Set(video_bitrate),
        audio_codec: Set(primary_audio.and_then(|stream| stream.codec_name.clone())),
        audio_bitrate: Set(audio_bitrate),
        audio_channels: Set(audio_channels),
        has_subtitles: Set(i64::from(has_subtitles)),
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
    .exec(db)
    .await
    .with_context(|| {
        format!(
            "failed storing ffprobe for file {file_id} (duration_s={duration_s:?}, width={width:?}, height={height:?}, fps={fps:?}, video_bitrate={video_bitrate:?}, audio_bitrate={audio_bitrate:?}, audio_channels={audio_channels:?}, has_subtitles={has_subtitles})"
        )
    })?;

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
