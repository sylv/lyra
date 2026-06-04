use crate::jobs::{Job, JobLease, JobOutcome, JobScheduling};
use crate::media::get_job_file_path;
use crate::{
    assets as assets_api,
    entities::{
        file_images::{self, FileImageRole},
        files,
        images::{self as images_entity, ImageKind},
        jobs as jobs_entity,
    },
};
use lyra_thumbnail::{ThumbnailOptions, generate_thumbnail};
use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, DatabaseConnection, EntityTrait, ExprTrait, QueryFilter, QueryOrder, Select,
    TransactionTrait,
    sea_query::{Expr, Query},
};

#[derive(Debug, Default)]
pub struct FileThumbnailJob;

#[async_trait::async_trait]
impl Job for FileThumbnailJob {
    type Entity = files::Entity;
    type Model = files::Model;

    const JOB_KIND: jobs_entity::JobKind = jobs_entity::JobKind::FileGenerateThumbnail;
    const SCHEDULING: JobScheduling = JobScheduling::Heavy(3);

    fn query(&self) -> Select<Self::Entity> {
        files::Entity::find()
            .filter(files::Column::UnavailableAt.is_null())
            .filter(
                Expr::col((files::Entity, files::Column::Id)).not_in_subquery(
                    Query::select()
                        .column(file_images::Column::FileId)
                        .from(file_images::Entity)
                        .and_where(
                            Expr::col((file_images::Entity, file_images::Column::Role))
                                .eq(FileImageRole::Thumbnail),
                        )
                        .to_owned(),
                ),
            )
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
        let Some(file_path) = get_job_file_path(db, &file, Self::JOB_KIND).await? else {
            return Ok(JobOutcome::Complete);
        };

        let thumbnail_options = ThumbnailOptions::default();
        let Some(thumbnail) =
            generate_thumbnail(&file_path, &thumbnail_options, ctx.get_cancellation_token())
                .await?
        else {
            return Ok(JobOutcome::Cancelled);
        };
        let file_id = file.id.clone();

        let tx = db.begin().await?;
        // Existing thumbnails are replaced atomically so retries do not accumulate stale assets.
        let stale_image_ids = file_images::Entity::find()
            .filter(file_images::Column::FileId.eq(file_id.clone()))
            .filter(file_images::Column::Role.eq(FileImageRole::Thumbnail))
            .all(&tx)
            .await?
            .into_iter()
            .map(|row| row.image_id)
            .collect::<Vec<_>>();

        file_images::Entity::delete_many()
            .filter(file_images::Column::FileId.eq(file_id.clone()))
            .filter(file_images::Column::Role.eq(FileImageRole::Thumbnail))
            .exec(&tx)
            .await?;

        if !stale_image_ids.is_empty() {
            images_entity::Entity::delete_many()
                .filter(images_entity::Column::Id.is_in(stale_image_ids))
                .exec(&tx)
                .await?;
        }

        let asset = assets_api::create_local_asset_from_bytes(
            &tx,
            &thumbnail.image_bytes,
            ImageKind::Thumbnail,
        )
        .await?;

        file_images::Entity::insert(file_images::ActiveModel {
            file_id: Set(file_id),
            image_id: Set(asset.id),
            role: Set(FileImageRole::Thumbnail),
            start_ms: Set(None),
            end_ms: Set(None),
            sheet_frame_height: Set(None),
            sheet_frame_width: Set(None),
            sheet_gap_size: Set(None),
            sheet_interval: Set(None),
        })
        .exec(&tx)
        .await?;

        tx.commit().await?;
        Ok(JobOutcome::Complete)
    }
}
