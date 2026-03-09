use crate::{
    assets as assets_api,
    entities::{
        file_assets::{self, FileAssetRole},
        files, jobs as jobs_entity,
    },
    jobs::{JobHandler, JobTarget, JobTargetId, handlers::shared},
};
use lyra_ffprobe::paths::get_ffmpeg_path;
use lyra_thumbnail::{ThumbnailOptions, generate_thumbnail};
use sea_orm::{
    ActiveValue::Set,
    DatabaseConnection, EntityTrait, TransactionTrait,
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
        (JobTarget::File, query)
    }

    async fn execute(
        &self,
        pool: &DatabaseConnection,
        target_id: &JobTargetId,
    ) -> anyhow::Result<()> {
        let file_id = shared::expect_file_target(target_id)?;
        let Some(ctx) = shared::load_job_file_context(pool, file_id, self.job_kind()).await? else {
            return Ok(());
        };

        let thumbnail_options = ThumbnailOptions {
            ffmpeg_bin: PathBuf::from(get_ffmpeg_path()?),
            ..ThumbnailOptions::default()
        };
        let thumbnail = generate_thumbnail(&ctx.file_path, &thumbnail_options).await?;

        let mut tx = pool.begin().await?;
        let asset = assets_api::create_local_asset_from_bytes(&tx, &thumbnail.image_bytes).await?;

        file_assets::Entity::insert(file_assets::ActiveModel {
            file_id: Set(ctx.file.id),
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

    async fn cleanup(
        &self,
        pool: &DatabaseConnection,
        target_id: &JobTargetId,
    ) -> anyhow::Result<()> {
        let file_id = shared::expect_file_target(target_id)?;
        shared::cleanup_file_assets_for_role(pool, file_id, FileAssetRole::Thumbnail).await
    }
}
