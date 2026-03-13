use crate::{
    assets::download_asset_to_local,
    entities::{assets, jobs as jobs_entity},
    jobs::handlers::shared,
    jobs::{ASSET_ID_COLUMN, JobHandler, JobTarget},
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    sea_query::SelectStatement,
};

#[derive(Debug, Default)]
pub struct AssetDownloadJob;

#[async_trait::async_trait]
impl JobHandler for AssetDownloadJob {
    fn job_kind(&self) -> jobs_entity::JobKind {
        jobs_entity::JobKind::AssetDownload
    }

    fn targets(&self) -> (JobTarget, SelectStatement) {
        let mut query = assets::Entity::find()
            .select_only()
            .column_as(assets::Column::Id, ASSET_ID_COLUMN)
            .filter(assets::Column::SourceUrl.is_not_null())
            .filter(assets::Column::HashSha256.is_null())
            .order_by_asc(assets::Column::Id);

        (JobTarget::Asset, QuerySelect::query(&mut query).to_owned())
    }

    async fn execute(
        &self,
        pool: &DatabaseConnection,
        job: &jobs_entity::Model,
    ) -> anyhow::Result<()> {
        let asset_id = shared::expect_job_asset_id(job)?;
        let Some(asset) = assets::Entity::find_by_id(asset_id).one(pool).await? else {
            return Ok(());
        };

        if asset.hash_sha256.is_some() {
            return Ok(());
        }

        download_asset_to_local(pool, &asset).await?;
        Ok(())
    }
}
