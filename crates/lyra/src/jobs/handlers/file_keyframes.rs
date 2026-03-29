use crate::jobs::{Job, JobLease, JobOutcome};
use crate::{
    entities::{files, jobs as jobs_entity},
    jobs::handlers::shared,
    json_encoding,
};
use anyhow::Context;
use lyra_keyframe_extractor::extract_keyframes;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Select,
};
use std::path::Path;

#[derive(Debug, Default)]
pub struct FileKeyframesJob;

#[async_trait::async_trait]
impl Job for FileKeyframesJob {
    type Entity = files::Entity;
    type Model = files::Model;

    const JOB_KIND: jobs_entity::JobKind = jobs_entity::JobKind::FileExtractKeyframes;
    const IS_HEAVY: bool = true;

    fn query(&self) -> Select<Self::Entity> {
        files::Entity::find()
            .filter(files::Column::UnavailableAt.is_null())
            .filter(files::Column::KeyframesJson.eq(Vec::<u8>::new()))
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
        let Some(file_path) = shared::get_job_file_path(db, &file, Self::JOB_KIND).await? else {
            return Ok(JobOutcome::Complete);
        };

        println!("extracting");
        let Some(_) = extract_and_store_keyframes(db, &file.id, &file_path, ctx).await? else {
            return Ok(JobOutcome::Cancelled);
        };

        Ok(JobOutcome::Complete)
    }
}

pub(crate) async fn extract_and_store_keyframes(
    db: &impl ConnectionTrait,
    file_id: &str,
    file_path: &Path,
    ctx: &JobLease,
) -> anyhow::Result<Option<Vec<i64>>> {
    let keyframes = extract_keyframes(file_path, ctx.get_cancellation_token()).await?;
    let Some(keyframes) = keyframes else {
        return Ok(None);
    };

    upsert_keyframes(db, file_id, &keyframes).await?;
    Ok(Some(keyframes))
}

async fn upsert_keyframes(
    db: &impl ConnectionTrait,
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
    .exec(db)
    .await?;

    Ok(())
}
