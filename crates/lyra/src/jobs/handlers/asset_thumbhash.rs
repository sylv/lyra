use crate::{
    assets::storage,
    entities::{assets, jobs as jobs_entity},
    jobs::handlers::shared,
    jobs::{ASSET_ID_COLUMN, JobHandler, JobTarget},
};
use anyhow::Context;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, sea_query::SelectStatement,
};

#[derive(Debug, Default)]
pub struct AssetThumbhashJob;

#[async_trait::async_trait]
impl JobHandler for AssetThumbhashJob {
    fn job_kind(&self) -> jobs_entity::JobKind {
        jobs_entity::JobKind::AssetGenerateThumbhash
    }

    fn targets(&self) -> (JobTarget, SelectStatement) {
        let mut query = assets::Entity::find()
            .select_only()
            .column_as(assets::Column::Id, ASSET_ID_COLUMN)
            .filter(assets::Column::HashSha256.is_not_null())
            .filter(
                Condition::any()
                    .add(assets::Column::Thumbhash.is_null())
                    .add(assets::Column::Thumbhash.eq(Vec::<u8>::new())),
            )
            .order_by_asc(assets::Column::Id);

        (JobTarget::Asset, QuerySelect::query(&mut query).to_owned())
    }

    async fn execute(
        &self,
        pool: &DatabaseConnection,
        job: &jobs_entity::Model,
    ) -> anyhow::Result<()> {
        let asset_id = shared::expect_job_asset_id(job)?;
        let Some(asset) = assets::Entity::find_by_id(&asset_id).one(pool).await? else {
            return Ok(());
        };

        if asset
            .thumbhash
            .as_ref()
            .is_some_and(|thumbhash| !thumbhash.is_empty())
        {
            return Ok(());
        }

        let hash_sha256 = asset
            .hash_sha256
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("asset {asset_id} is missing hash_sha256"))?;
        let mime_type = asset
            .mime_type
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("asset {asset_id} is missing mime_type"))?;
        let image_path = storage::get_asset_output_path_from_mime(hash_sha256, mime_type)?;
        let bytes = tokio::fs::read(&image_path)
            .await
            .with_context(|| format!("failed to read image at {}", image_path.display()))?;

        let format = image::guess_format(&bytes).context("failed to guess image format")?;
        let decoded = image::load_from_memory_with_format(&bytes, format)
            .context("failed to decode image")?;
        let resized = decoded.thumbnail(100, 100).to_rgba8();

        let width = usize::try_from(resized.width()).context("image width exceeds usize")?;
        let height = usize::try_from(resized.height()).context("image height exceeds usize")?;
        let thumbhash = thumbhash::rgba_to_thumb_hash(width, height, resized.as_raw());

        let mut updated: assets::ActiveModel = asset.into();
        updated.thumbhash = Set(Some(thumbhash));
        updated.update(pool).await?;

        Ok(())
    }
}
