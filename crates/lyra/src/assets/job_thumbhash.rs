use crate::entities::{
    images::{self, ImageKind},
    jobs as jobs_entity,
};
use crate::jobs::{Job, JobLease, JobOutcome};
use anyhow::{Context, Result};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Select,
};
use tokio::task::spawn_blocking;

#[derive(Debug, Default)]
pub struct AssetThumbhashJob;

#[async_trait::async_trait]
impl Job for AssetThumbhashJob {
    type Entity = images::Entity;
    type Model = images::Model;

    const JOB_KIND: jobs_entity::JobKind = jobs_entity::JobKind::ImageGenerateThumbhash;

    fn query(&self) -> Select<Self::Entity> {
        images::Entity::find()
            .filter(images::Column::Kind.is_in([
                ImageKind::Poster,
                ImageKind::Thumbnail,
                ImageKind::Backdrop,
            ]))
            .filter(images::Column::MimeType.is_not_null())
            .filter(
                Condition::any()
                    .add(images::Column::Thumbhash.is_null())
                    .add(images::Column::Thumbhash.eq(Vec::<u8>::new())),
            )
            .order_by_asc(images::Column::Id)
    }

    fn target_id(&self, target: &Self::Model) -> String {
        target.id.clone()
    }

    async fn run(
        &self,
        db: &DatabaseConnection,
        image: Self::Model,
        _ctx: &JobLease,
    ) -> anyhow::Result<JobOutcome> {
        let image_id = image.id.clone();
        if image
            .thumbhash
            .as_ref()
            .is_some_and(|thumbhash| !thumbhash.is_empty())
        {
            return Ok(JobOutcome::Complete);
        }

        let mime_type = image
            .mime_type
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("image {image_id} is missing mime_type"))?;
        let image_path = super::storage::get_asset_output_path_from_mime(&image.id, mime_type)?;
        let bytes = tokio::fs::read(&image_path)
            .await
            .with_context(|| format!("failed to read image at {}", image_path.display()))?;

        let format = image::guess_format(&bytes).context("failed to guess image format")?;
        let thumbhash = spawn_blocking::<_, Result<Vec<u8>>>(move || {
            let decoded = image::load_from_memory_with_format(&bytes, format)
                .context("failed to decode image")?;
            let resized = decoded.thumbnail(100, 100).to_rgba8();

            let width = usize::try_from(resized.width()).context("image width exceeds usize")?;
            let height = usize::try_from(resized.height()).context("image height exceeds usize")?;
            Ok(thumbhash::rgba_to_thumb_hash(
                width,
                height,
                resized.as_raw(),
            ))
        })
        .await??;

        let mut updated: images::ActiveModel = image.into();
        updated.thumbhash = Set(Some(thumbhash));
        updated.update(db).await?;

        Ok(JobOutcome::Complete)
    }
}
