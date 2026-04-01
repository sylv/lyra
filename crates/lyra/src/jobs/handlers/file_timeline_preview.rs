use crate::jobs::handlers::shared::get_job_file_path;
use crate::jobs::{Job, JobLease, JobOutcome};
use crate::{
    assets as assets_api,
    config::get_config,
    entities::{
        assets::{self as assets_entity, AssetKind},
        file_assets::{self, FileAssetRole},
        files, jobs as jobs_entity,
    },
};
use anyhow::Context;
use lyra_ffprobe::paths::get_ffmpeg_path;
use lyra_timeline_preview::{PreviewOptions, generate_previews};
use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Select,
    TransactionTrait,
    sea_query::{Expr, Query},
};
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Default)]
pub struct FileTimelinePreviewJob;

#[async_trait::async_trait]
impl Job for FileTimelinePreviewJob {
    type Entity = files::Entity;
    type Model = files::Model;

    const JOB_KIND: jobs_entity::JobKind = jobs_entity::JobKind::FileGenerateTimelinePreview;
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
                                .eq(FileAssetRole::TimelinePreviewSheet),
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
        lease: &JobLease,
    ) -> anyhow::Result<JobOutcome> {
        let Some(file_path) = get_job_file_path(db, &file, Self::JOB_KIND).await? else {
            return Ok(JobOutcome::Complete);
        };
        let file_id = file.id.clone();
        let preview_options = PreviewOptions {
            ffmpeg_bin: PathBuf::from(get_ffmpeg_path()?),
            working_dir: get_config()
                .data_dir
                .join("tmp")
                .join("timeline-preview")
                .join(file_id.clone()),
            ..PreviewOptions::default()
        };

        let Some(timeline_previews) =
            generate_previews(&file_path, &preview_options, lease.get_cancellation_token()).await?
        else {
            return Ok(JobOutcome::Cancelled);
        };

        let mut tx = db.begin().await?;

        // Existing assets are replaced atomically so retrying the job never leaves duplicate sheets.
        let stale_asset_ids = file_assets::Entity::find()
            .filter(file_assets::Column::FileId.eq(file_id.clone()))
            .filter(file_assets::Column::Role.eq(FileAssetRole::TimelinePreviewSheet))
            .all(&tx)
            .await?
            .into_iter()
            .map(|row| row.asset_id)
            .collect::<Vec<_>>();

        file_assets::Entity::delete_many()
            .filter(file_assets::Column::FileId.eq(file_id.clone()))
            .filter(file_assets::Column::Role.eq(FileAssetRole::TimelinePreviewSheet))
            .exec(&tx)
            .await?;

        if !stale_asset_ids.is_empty() {
            assets_entity::Entity::delete_many()
                .filter(assets_entity::Column::Id.is_in(stale_asset_ids))
                .exec(&tx)
                .await?;
        }

        for preview in timeline_previews {
            let asset = assets_api::create_local_asset_from_bytes(
                &tx,
                &preview.preview_bytes,
                AssetKind::TimelinePreviewSheet,
            )
            .await?;
            let sheet_width_px = asset
                .width
                .context("timeline preview asset missing width")?;
            let sheet_height_px = asset
                .height
                .context("timeline preview asset missing height")?;

            file_assets::Entity::insert(file_assets::ActiveModel {
                file_id: Set(file_id.clone()),
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
        Ok(JobOutcome::Complete)
    }
}

fn duration_to_millis(duration: Duration) -> anyhow::Result<i64> {
    i64::try_from(duration.as_millis()).context("duration is too large to fit into i64 millis")
}
