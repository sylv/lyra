use crate::{
    entities::{file_keyframes, files, jobs as jobs_entity},
    ffmpeg,
    jobs::{JobHandler, handlers::shared},
    json_encoding,
};
use anyhow::Context;
use lyra_ffprobe::probe_keyframes_pts;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    sea_query::OnConflict, sea_query::Query,
};
use std::{path::Path, path::PathBuf};

#[derive(Debug, Default)]
pub struct FileKeyframesJob;

#[async_trait::async_trait]
impl JobHandler for FileKeyframesJob {
    fn job_type(&self) -> jobs_entity::JobType {
        jobs_entity::JobType::FileExtractKeyframes
    }

    fn filter_condition(&self) -> Option<Condition> {
        Some(
            Condition::all().add(
                files::Column::Id.not_in_subquery(
                    Query::select()
                        .column(file_keyframes::Column::FileId)
                        .from(file_keyframes::Entity)
                        .to_owned(),
                ),
            ),
        )
    }

    async fn execute(&self, pool: &DatabaseConnection, file_id: i64) -> anyhow::Result<()> {
        let Some(ctx) = shared::load_job_file_context(pool, file_id, self.job_type()).await? else {
            return Ok(());
        };

        extract_and_store_keyframes(pool, ctx.file.id, &ctx.file_path).await?;
        Ok(())
    }

    async fn cleanup(&self, pool: &DatabaseConnection, file_id: i64) -> anyhow::Result<()> {
        file_keyframes::Entity::delete_by_id(file_id)
            .exec(pool)
            .await?;
        Ok(())
    }
}

pub(crate) async fn extract_and_store_keyframes(
    pool: &DatabaseConnection,
    file_id: i64,
    file_path: &Path,
) -> anyhow::Result<Vec<i64>> {
    let ffprobe_bin = PathBuf::from(ffmpeg::get_ffprobe_path());
    let input = file_path.to_path_buf();
    let keyframes = tokio::task::spawn_blocking(move || probe_keyframes_pts(&ffprobe_bin, &input))
        .await
        .context("ffprobe keyframe task panicked")??;

    upsert_keyframes(pool, file_id, &keyframes).await?;
    Ok(keyframes)
}

async fn upsert_keyframes(
    pool: &DatabaseConnection,
    file_id: i64,
    keyframes: &[i64],
) -> anyhow::Result<()> {
    let payload =
        json_encoding::encode_json_zstd(&keyframes).context("failed to encode keyframe payload")?;
    let now = chrono::Utc::now().timestamp();

    file_keyframes::Entity::insert(file_keyframes::ActiveModel {
        file_id: Set(file_id),
        keyframe_list: Set(payload),
        generated_at: Set(now),
    })
    .on_conflict(
        OnConflict::column(file_keyframes::Column::FileId)
            .update_columns([
                file_keyframes::Column::KeyframeList,
                file_keyframes::Column::GeneratedAt,
            ])
            .to_owned(),
    )
    .exec(pool)
    .await?;

    Ok(())
}
