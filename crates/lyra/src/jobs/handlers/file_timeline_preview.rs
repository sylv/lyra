use crate::{
    assets as assets_api,
    config::get_config,
    entities::{
        file_assets::{self, FileAssetRole},
        files, jobs as jobs_entity,
    },
    jobs::{JobHandler, JobTarget, JobTargetId, handlers::shared},
};
use anyhow::Context;
use lyra_ffprobe::paths::get_ffmpeg_path;
use lyra_timeline_preview::{PreviewOptions, generate_previews};
use sea_orm::{
    ActiveValue::Set,
    DatabaseConnection, EntityTrait, TransactionTrait,
    sea_query::{Expr, Query, SelectStatement},
};
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Default)]
pub struct FileTimelinePreviewJob;

#[async_trait::async_trait]
impl JobHandler for FileTimelinePreviewJob {
    fn job_kind(&self) -> jobs_entity::JobKind {
        jobs_entity::JobKind::FileGenerateTimelinePreview
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
                            .eq(FileAssetRole::TimelinePreviewSheet),
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

        let preview_options = PreviewOptions {
            ffmpeg_bin: PathBuf::from(get_ffmpeg_path()?),
            working_dir: get_config()
                .data_dir
                .join("tmp")
                .join("timeline-preview")
                .join(file_id.to_string()),
            ..PreviewOptions::default()
        };

        let timeline_previews = generate_previews(&ctx.file_path, &preview_options).await?;

        let mut tx = pool.begin().await?;
        for preview in timeline_previews {
            let asset =
                assets_api::create_local_asset_from_bytes(&tx, &preview.preview_bytes).await?;
            let sheet_width_px = asset
                .width
                .context("timeline preview asset missing width")?;
            let sheet_height_px = asset
                .height
                .context("timeline preview asset missing height")?;

            file_assets::Entity::insert(file_assets::ActiveModel {
                file_id: Set(ctx.file.id),
                asset_id: Set(asset.id),
                role: Set(FileAssetRole::TimelinePreviewSheet),
                chapter_number: Set(None),
                position_ms: Set(Some(duration_to_millis(preview.start_time)?)),
                end_ms: Set(Some(duration_to_millis(preview.end_time)?)),
                sheet_frame_height: Set(Some(sheet_height_px)),
                sheet_frame_width: Set(Some(sheet_width_px)),
                sheet_gap_size: Set(Some(lyra_timeline_preview::GAP_PX as i64)),
                sheet_interval: Set(Some(duration_to_millis(preview.frame_interval)?)),
            })
            .exec(&mut tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn cleanup(
        &self,
        pool: &DatabaseConnection,
        target_id: &JobTargetId,
    ) -> anyhow::Result<()> {
        let file_id = shared::expect_file_target(target_id)?;
        shared::cleanup_file_assets_for_role(pool, file_id, FileAssetRole::TimelinePreviewSheet)
            .await
    }
}

fn duration_to_millis(duration: Duration) -> anyhow::Result<i64> {
    i64::try_from(duration.as_millis()).context("duration is too large to fit into i64 millis")
}
