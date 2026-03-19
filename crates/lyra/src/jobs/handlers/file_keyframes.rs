use crate::{
    entities::{files, jobs as jobs_entity},
    jobs::handlers::shared,
    jobs::{JobHandler, JobRunContext, JobRunResult, JobTarget},
    json_encoding,
};
use anyhow::Context;
use lyra_ffprobe::{paths::get_ffprobe_path, probe_keyframes_pts};
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait, sea_query::SelectStatement};
use std::path::Path;

#[derive(Debug, Default)]
pub struct FileKeyframesJob;

#[async_trait::async_trait]
impl JobHandler for FileKeyframesJob {
    fn job_kind(&self) -> jobs_entity::JobKind {
        jobs_entity::JobKind::FileExtractKeyframes
    }

    fn is_heavy(&self) -> bool {
        true
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
        job: &jobs_entity::Model,
        ctx: &JobRunContext,
    ) -> anyhow::Result<JobRunResult> {
        let file_id = shared::expect_job_file_id(job)?;
        let Some(file_ctx) = shared::load_job_file_context(pool, &file_id, self.job_kind()).await?
        else {
            return Ok(JobRunResult::Complete);
        };

        let Some(_) =
            extract_and_store_keyframes(pool, &file_ctx.file.id, &file_ctx.file_path, ctx).await?
        else {
            return Ok(JobRunResult::Cancelled);
        };

        Ok(JobRunResult::Complete)
    }
}

pub(crate) async fn extract_and_store_keyframes(
    pool: &DatabaseConnection,
    file_id: &str,
    file_path: &Path,
    ctx: &JobRunContext,
) -> anyhow::Result<Option<Vec<i64>>> {
    let keyframes = probe_keyframes_pts(
        get_ffprobe_path()?,
        file_path,
        Some(ctx.cancellation_token()),
    )
    .await?;
    let Some(keyframes) = keyframes else {
        return Ok(None);
    };

    upsert_keyframes(pool, file_id, &keyframes).await?;
    Ok(Some(keyframes))
}

async fn upsert_keyframes(
    pool: &DatabaseConnection,
    file_id: &str,
    keyframes: &[i64],
) -> anyhow::Result<()> {
    let payload =
        json_encoding::encode_json_zstd(&keyframes).context("failed to encode keyframe payload")?;

    files::Entity::update(files::ActiveModel {
        id: Set(file_id.to_string()),
        keyframes_json: Set(payload),
        ..Default::default()
    })
    .exec(pool)
    .await?;

    Ok(())
}
