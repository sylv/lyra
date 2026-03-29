use crate::jobs::handlers::shared::get_job_file_path;
use crate::jobs::{Job, JobLease, JobOutcome};
use crate::{
    assets as assets_api,
    entities::{
        assets as assets_entity,
        file_assets::{self, FileAssetRole},
        files, jobs as jobs_entity,
        metadata_source::MetadataSource,
        node_files, node_metadata,
    },
};
use lyra_ffprobe::paths::get_ffmpeg_path;
use lyra_thumbnail::{ThumbnailOptions, generate_thumbnail};
use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Select,
    TransactionTrait,
    sea_query::{Expr, Query},
};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct FileThumbnailJob;

#[async_trait::async_trait]
impl Job for FileThumbnailJob {
    type Entity = files::Entity;
    type Model = files::Model;

    const JOB_KIND: jobs_entity::JobKind = jobs_entity::JobKind::FileGenerateThumbnail;
    const IS_HEAVY: bool = true;

    fn query(&self) -> Select<Self::Entity> {
        files::Entity::find()
            .filter(files::Column::UnavailableAt.is_null())
            .filter(
                Expr::col((files::Entity, files::Column::Id)).not_in_subquery(
                    Query::select()
                        .column(file_assets::Column::FileId)
                        .from(file_assets::Entity)
                        .and_where(
                            Expr::col((file_assets::Entity, file_assets::Column::Role))
                                .eq(FileAssetRole::Thumbnail),
                        )
                        .to_owned(),
                ),
            )
            .filter(
                Expr::col((files::Entity, files::Column::Id)).not_in_subquery(
                    Query::select()
                        .column(node_files::Column::FileId)
                        .from(node_files::Entity)
                        .and_where(
                            Expr::col((node_files::Entity, node_files::Column::NodeId))
                                .in_subquery(
                                    Query::select()
                                        .column(node_metadata::Column::NodeId)
                                        .from(node_metadata::Entity)
                                        .and_where(
                                            Expr::col((
                                                node_metadata::Entity,
                                                node_metadata::Column::Source,
                                            ))
                                            .eq(MetadataSource::Remote),
                                        )
                                        .and_where(
                                            Expr::col((
                                                node_metadata::Entity,
                                                node_metadata::Column::ThumbnailAssetId,
                                            ))
                                            .is_not_null(),
                                        )
                                        .to_owned(),
                                ),
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

        let thumbnail_options = ThumbnailOptions {
            ffmpeg_bin: PathBuf::from(get_ffmpeg_path()?),
            ..ThumbnailOptions::default()
        };

        let Some(thumbnail) =
            generate_thumbnail(&file_path, &thumbnail_options, ctx.get_cancellation_token())
                .await?
        else {
            return Ok(JobOutcome::Cancelled);
        };
        let file_id = file.id.clone();

        let mut tx = db.begin().await?;
        // Existing thumbnails are replaced atomically so retries do not accumulate stale assets.
        let stale_asset_ids = file_assets::Entity::find()
            .filter(file_assets::Column::FileId.eq(file_id.clone()))
            .filter(file_assets::Column::Role.eq(FileAssetRole::Thumbnail))
            .all(&tx)
            .await?
            .into_iter()
            .map(|row| row.asset_id)
            .collect::<Vec<_>>();

        file_assets::Entity::delete_many()
            .filter(file_assets::Column::FileId.eq(file_id.clone()))
            .filter(file_assets::Column::Role.eq(FileAssetRole::Thumbnail))
            .exec(&tx)
            .await?;

        if !stale_asset_ids.is_empty() {
            assets_entity::Entity::delete_many()
                .filter(assets_entity::Column::Id.is_in(stale_asset_ids))
                .exec(&tx)
                .await?;
        }

        let asset = assets_api::create_local_asset_from_bytes(&tx, &thumbnail.image_bytes).await?;

        file_assets::Entity::insert(file_assets::ActiveModel {
            file_id: Set(file_id),
            asset_id: Set(asset.id),
            role: Set(FileAssetRole::Thumbnail),
            chapter_number: Set(None),
            position_ms: Set(None),
            end_ms: Set(None),
            sheet_frame_height: Set(None),
            sheet_frame_width: Set(None),
            sheet_gap_size: Set(None),
            sheet_interval: Set(None),
        })
        .exec(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(JobOutcome::Complete)
    }
}
