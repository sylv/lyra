use crate::entities::{
    images::{self, ImageKind},
    jobs as jobs_entity,
};
use crate::jobs::{Job, JobLease, JobOutcome};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Select};

#[derive(Debug, Default)]
pub struct AssetDownloadJob;

#[async_trait::async_trait]
impl Job for AssetDownloadJob {
    type Entity = images::Entity;
    type Model = images::Model;

    const JOB_KIND: jobs_entity::JobKind = jobs_entity::JobKind::ImageDownload;

    fn query(&self) -> Select<Self::Entity> {
        images::Entity::find()
            .filter(images::Column::Kind.is_in(vec![
                ImageKind::Thumbnail,
                ImageKind::Poster,
                ImageKind::Backdrop,
                ImageKind::Logo,
            ]))
            .filter(images::Column::SourceUrl.is_not_null())
            .filter(images::Column::MimeType.is_null())
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
        if image.mime_type.is_some() {
            return Ok(JobOutcome::Complete);
        }

        super::download_asset_to_local(db, &image).await?;
        Ok(JobOutcome::Complete)
    }
}
