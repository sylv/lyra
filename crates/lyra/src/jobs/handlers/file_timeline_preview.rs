use crate::{
    assets as assets_api,
    config::get_config,
    entities::{
        file_assets::{self, FileAssetRole},
        jobs as jobs_entity,
    },
    ffmpeg,
    jobs::{JobHandler, handlers::shared},
};
use anyhow::Context;
use lyra_timeline_preview::{PreviewOptions, generate_previews};
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait, TransactionTrait};
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Default)]
pub struct FileTimelinePreviewJob;

#[async_trait::async_trait]
impl JobHandler for FileTimelinePreviewJob {
    fn job_type(&self) -> jobs_entity::JobType {
        jobs_entity::JobType::FileGenerateTimelinePreview
    }

    async fn execute(&self, pool: &DatabaseConnection, file_id: i64) -> anyhow::Result<()> {
        let Some(ctx) = shared::load_job_file_context(pool, file_id, self.job_type()).await? else {
            return Ok(());
        };

        let preview_options = PreviewOptions {
            ffmpeg_bin: PathBuf::from(ffmpeg::get_ffmpeg_path()),
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

    async fn cleanup(&self, pool: &DatabaseConnection, file_id: i64) -> anyhow::Result<()> {
        shared::cleanup_file_assets_for_role(pool, file_id, FileAssetRole::TimelinePreviewSheet)
            .await
    }
}

fn duration_to_millis(duration: Duration) -> anyhow::Result<i64> {
    i64::try_from(duration.as_millis()).context("duration is too large to fit into i64 millis")
}
