use crate::{
    assets as assets_api,
    config::get_config,
    entities::{
        assets as assets_entity,
        file_assets::{self, FileAssetRole},
        files, libraries,
    },
    ffmpeg,
    jobs::{JobExecutionPolicy, JobHandler},
};
use anyhow::Context;
use lyra_timeline_preview::{PreviewOptions, generate_previews};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait,
};
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Default)]
pub struct FileTimelinePreviewJob;

#[async_trait::async_trait]
impl JobHandler for FileTimelinePreviewJob {
    fn job_type(&self) -> &'static str {
        "file.generate_timeline_preview"
    }

    fn execution_policy(&self) -> JobExecutionPolicy {
        JobExecutionPolicy::default()
    }

    async fn execute(&self, pool: &DatabaseConnection, file_id: i64) -> anyhow::Result<()> {
        let maybe_file = files::Entity::find_by_id(file_id)
            .find_also_related(libraries::Entity)
            .one(pool)
            .await
            .with_context(|| format!("failed to fetch file {file_id}"))?;

        let Some((file, library)) = maybe_file else {
            return Ok(());
        };

        let Some(library) = library else {
            return Ok(());
        };

        if file.unavailable_at.is_some() || file.corrupted_at.is_some() {
            return Ok(());
        }

        let file_path = PathBuf::from(&library.path).join(&file.relative_path);
        if !file_path.exists() {
            tracing::warn!(
                file_id,
                path = %file_path.display(),
                "file path missing while generating timeline preview"
            );

            files::Entity::update(files::ActiveModel {
                id: Set(file.id),
                unavailable_at: Set(Some(chrono::Utc::now().timestamp())),
                ..Default::default()
            })
            .exec(pool)
            .await?;

            anyhow::bail!("file path missing while generating timeline preview");
        }

        let preview_options = PreviewOptions {
            ffmpeg_bin: PathBuf::from(ffmpeg::get_ffmpeg_path()),
            working_dir: get_config()
                .data_dir
                .join("tmp")
                .join("timeline-preview")
                .join(file_id.to_string()),
            ..PreviewOptions::default()
        };

        let timeline_previews = generate_previews(&file_path, &preview_options).await?;

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
                file_id: Set(file.id),
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
        let tx = pool.begin().await?;
        let stale_asset_ids: Vec<i64> = file_assets::Entity::find()
            .filter(file_assets::Column::FileId.eq(file_id))
            .filter(file_assets::Column::Role.eq(FileAssetRole::TimelinePreviewSheet))
            .all(&tx)
            .await?
            .into_iter()
            .map(|row| row.asset_id)
            .collect();

        file_assets::Entity::delete_many()
            .filter(file_assets::Column::FileId.eq(file_id))
            .filter(file_assets::Column::Role.eq(FileAssetRole::TimelinePreviewSheet))
            .exec(&tx)
            .await?;

        let now = chrono::Utc::now().timestamp();
        assets_entity::Entity::update_many()
            .filter(assets_entity::Column::Id.is_in(stale_asset_ids))
            .set(assets_entity::ActiveModel {
                deleted_at: Set(Some(now)),
                ..Default::default()
            })
            .exec(&tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }
}

fn duration_to_millis(duration: Duration) -> anyhow::Result<i64> {
    i64::try_from(duration.as_millis()).context("duration is too large to fit into i64 millis")
}
