use crate::{
    entities::{files, jobs as jobs_entity},
    jobs::{JobHandler, JobTarget, JobTargetId, handlers::shared},
    json_encoding,
};
use anyhow::Context;
use lyra_ffprobe::{paths::get_ffprobe_path, probe_keyframes_pts};
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait, sea_query::SelectStatement};
use std::{path::Path, path::PathBuf};

#[derive(Debug, Default)]
pub struct FileKeyframesJob;

#[async_trait::async_trait]
impl JobHandler for FileKeyframesJob {
    fn job_kind(&self) -> jobs_entity::JobKind {
        jobs_entity::JobKind::FileExtractKeyframes
    }

    fn targets(&self) -> (JobTarget, SelectStatement) {
        let mut query = shared::base_file_targets_query();
        query.and_where(
            sea_orm::sea_query::Expr::col((files::Entity, files::Column::KeyframesJson))
                .eq(Vec::<u8>::new()),
        );
        (JobTarget::File, query)
    }

    async fn execute(
        &self,
        pool: &DatabaseConnection,
        target_id: &JobTargetId,
    ) -> anyhow::Result<()> {
        let file_id = shared::expect_file_target(target_id)?;
        let Some(ctx) = shared::load_job_file_context(pool, file_id, self.job_kind()).await? else {
            return Ok(());
        };

        extract_and_store_keyframes(pool, ctx.file.id, &ctx.file_path).await?;
        Ok(())
    }

    async fn cleanup(
        &self,
        pool: &DatabaseConnection,
        target_id: &JobTargetId,
    ) -> anyhow::Result<()> {
        let file_id = shared::expect_file_target(target_id)?;
        files::Entity::update(files::ActiveModel {
            id: Set(file_id),
            keyframes_json: Set(Vec::new()),
            ..Default::default()
        })
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
    let ffprobe_bin = PathBuf::from(get_ffprobe_path()?);
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

    files::Entity::update(files::ActiveModel {
        id: Set(file_id),
        keyframes_json: Set(payload),
        ..Default::default()
    })
    .exec(pool)
    .await?;

    Ok(())
}
