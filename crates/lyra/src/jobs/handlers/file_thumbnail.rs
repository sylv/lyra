use crate::{
    assets as assets_api,
    entities::{
        assets as assets_entity,
        file_assets::{self, FileAssetRole},
        files, jobs as jobs_entity,
        metadata_source::MetadataSource,
        node_files, node_metadata,
    },
    jobs::handlers::shared,
    jobs::{JobHandler, JobTarget},
};
use lyra_ffprobe::paths::get_ffmpeg_path;
use lyra_thumbnail::{ThumbnailOptions, generate_thumbnail};
use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait,
    sea_query::{Expr, Query, SelectStatement},
};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct FileThumbnailJob;

#[async_trait::async_trait]
impl JobHandler for FileThumbnailJob {
    fn job_kind(&self) -> jobs_entity::JobKind {
        jobs_entity::JobKind::FileGenerateThumbnail
    }

    fn targets(&self) -> (JobTarget, SelectStatement) {
        let mut query = shared::base_file_targets_query();
        query.and_where(
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
        );
        query.and_where(
            Expr::col((files::Entity, files::Column::Id)).not_in_subquery(
                Query::select()
                    .column(node_files::Column::FileId)
                    .from(node_files::Entity)
                    .and_where(
                        Expr::col((node_files::Entity, node_files::Column::NodeId)).in_subquery(
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
        );
        (JobTarget::File, query)
    }

    async fn execute(
        &self,
        pool: &DatabaseConnection,
        job: &jobs_entity::Model,
    ) -> anyhow::Result<()> {
        let file_id = shared::expect_job_file_id(job)?;
        let Some(ctx) = shared::load_job_file_context(pool, &file_id, self.job_kind()).await?
        else {
            return Ok(());
        };

        let thumbnail_options = ThumbnailOptions {
            ffmpeg_bin: PathBuf::from(get_ffmpeg_path()?),
            ..ThumbnailOptions::default()
        };
        let thumbnail = generate_thumbnail(&ctx.file_path, &thumbnail_options).await?;
        let file_id = ctx.file.id.clone();

        // todo: we could skip this with a smarter query
        let mut tx = pool.begin().await?;
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
        Ok(())
    }
}
