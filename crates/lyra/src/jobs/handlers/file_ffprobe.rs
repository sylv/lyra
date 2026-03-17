use crate::{
    entities::{file_probe, files, jobs as jobs_entity},
    jobs::handlers::shared,
    jobs::{JobHandler, JobTarget},
    json_encoding,
};
use anyhow::Context;
use lyra_ffprobe::{
    FfprobeOutput, paths::get_ffprobe_path, probe_output, probe_streams_from_output,
};
use sea_orm::{
    ActiveValue::Set,
    DatabaseConnection, EntityTrait,
    sea_query::{Expr, OnConflict, Query, SelectStatement},
};
use std::{path::Path, path::PathBuf};

#[derive(Debug, Default)]
pub struct FileFfprobeJob;

#[async_trait::async_trait]
impl JobHandler for FileFfprobeJob {
    fn job_kind(&self) -> jobs_entity::JobKind {
        jobs_entity::JobKind::FileExtractFfprobe
    }

    fn targets(&self) -> (JobTarget, SelectStatement) {
        let mut query = shared::base_file_targets_query();
        query.and_where(
            Expr::col((files::Entity, files::Column::Id)).not_in_subquery(
                Query::select()
                    .column(file_probe::Column::FileId)
                    .from(file_probe::Entity)
                    .to_owned(),
            ),
        );
        (JobTarget::File, query)
    }

    async fn execute(
        &self,
        pool: &DatabaseConnection,
        job: &jobs_entity::Model,
    ) -> anyhow::Result<()> {
        let file_id = shared::expect_job_file_id(job)?;
        let Some(ctx) = shared::load_job_file_context(pool, &file_id, self.job_kind()).await?
        else {
            return Ok(());
        };

        extract_and_store_ffprobe(pool, &ctx.file.id, &ctx.file_path).await?;
        Ok(())
    }
}

pub(crate) async fn extract_and_store_ffprobe(
    pool: &DatabaseConnection,
    file_id: &str,
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
    .exec(pool)
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
