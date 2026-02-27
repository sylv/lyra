use crate::{
    assets as assets_api,
    entities::{
        file_assets::{self, FileAssetRole},
        jobs as jobs_entity,
    },
    jobs::{JobHandler, handlers::shared},
};
use lyra_ffprobe::paths::get_ffmpeg_path;
use lyra_thumbnail::{ThumbnailOptions, generate_thumbnail};
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait, TransactionTrait};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct FileThumbnailJob;

#[async_trait::async_trait]
impl JobHandler for FileThumbnailJob {
    fn job_type(&self) -> jobs_entity::JobType {
        jobs_entity::JobType::FileGenerateThumbnail
    }

    async fn execute(&self, pool: &DatabaseConnection, file_id: i64) -> anyhow::Result<()> {
        let Some(ctx) = shared::load_job_file_context(pool, file_id, self.job_type()).await? else {
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

    async fn cleanup(&self, pool: &DatabaseConnection, file_id: i64) -> anyhow::Result<()> {
        shared::cleanup_file_assets_for_role(pool, file_id, FileAssetRole::Thumbnail).await
    }
}
