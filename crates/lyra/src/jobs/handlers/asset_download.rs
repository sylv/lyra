use crate::jobs::{Job, JobLease, JobOutcome};
use crate::{
    assets::download_asset_to_local,
    entities::{assets, jobs as jobs_entity},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Select};

#[derive(Debug, Default)]
pub struct AssetDownloadJob;

#[async_trait::async_trait]
impl Job for AssetDownloadJob {
    type Entity = assets::Entity;
    type Model = assets::Model;

    const JOB_KIND: jobs_entity::JobKind = jobs_entity::JobKind::AssetDownload;

    fn query(&self) -> Select<Self::Entity> {
        assets::Entity::find()
            .filter(assets::Column::SourceUrl.is_not_null())
            .filter(assets::Column::HashSha256.is_null())
            .order_by_asc(assets::Column::Id)
    }

    fn target_id(&self, target: &Self::Model) -> String {
        target.id.clone()
    }

    async fn run(
        &self,
        db: &DatabaseConnection,
        asset: Self::Model,
        _ctx: &JobLease,
    ) -> anyhow::Result<JobOutcome> {
        if asset.hash_sha256.is_some() {
            return Ok(JobOutcome::Complete);
        }

        download_asset_to_local(db, &asset).await?;
        Ok(JobOutcome::Complete)
    }
}
