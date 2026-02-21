use crate::{
    config::get_config,
    entities::{
        assets::{self, AssetSource},
        file_assets::{self, FileAssetRole},
        files, libraries, tasks as tasks_entity,
    },
    ffmpeg,
    tasks::{TaskHandler, TaskLike, TaskScopeKind},
};
use anyhow::Context;
use lyra_timeline_preview::{PreviewOptions, TimelinePreview, generate_previews};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FileTimelinePreviewTaskArgs;

#[derive(Debug, Default)]
pub struct FileTimelinePreviewTask;

#[derive(Debug)]
struct PreparedPreviewSheet {
    hash_sha256: String,
    size_bytes: i64,
    start_ms: i64,
    end_ms: i64,
    frame_interval_ms: i64,
    sheet_width_px: i64,
}

#[async_trait::async_trait]
impl TaskHandler for FileTimelinePreviewTask {
    type InputArgs = FileTimelinePreviewTaskArgs;

    fn task_type(&self) -> &'static str {
        "file.generate_timeline_preview"
    }

    fn version_number(&self) -> i64 {
        1
    }

    async fn reconcile(
        &self,
        pool: &DatabaseConnection,
    ) -> anyhow::Result<Vec<TaskLike<Self::InputArgs>>> {
        let all_files = files::Entity::find().all(pool).await?;
        Ok(all_files
            .into_iter()
            .map(|file| TaskLike {
                scope_kind: TaskScopeKind::File,
                scope_id: file.id.to_string(),
                input_args: None,
                version_hash: None,
            })
            .collect())
    }

    async fn execute(
        &self,
        pool: &DatabaseConnection,
        task: &tasks_entity::Model,
        _args: &Self::InputArgs,
    ) -> anyhow::Result<()> {
        let file_id = task
            .scope_id
            .parse::<i64>()
            .with_context(|| format!("invalid file id in scope_id '{}'", task.scope_id))?;

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

            return Ok(());
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
        let mut prepared_sheets = Vec::with_capacity(timeline_previews.len());
        for preview in timeline_previews {
            prepared_sheets.push(write_preview_sheet_to_disk(preview).await?);
        }

        let mut tx = pool.begin().await?;
        for sheet in prepared_sheets {
            // todo: get width/height
            let asset = assets::Entity::insert(assets::ActiveModel {
                source: Set(AssetSource::Local),
                source_url: Set(None),
                hash_sha256: Set(Some(sheet.hash_sha256)),
                size_bytes: Set(Some(sheet.size_bytes)),
                mime_type: Set(Some("image/webp".to_string())),
                height: Set(None),
                width: Set(Some(sheet.sheet_width_px)),
                thumbhash: Set(None),
                deleted_at: Set(None),
                ..Default::default()
            })
            .exec_with_returning(&mut tx)
            .await?;

            file_assets::Entity::insert(file_assets::ActiveModel {
                file_id: Set(file.id),
                asset_id: Set(asset.id),
                role: Set(FileAssetRole::TimelinePreviewSheet),
                chapter_number: Set(None),
                position_ms: Set(Some(sheet.start_ms)),
                end_ms: Set(Some(sheet.end_ms)),
                sheet_frame_height: Set(None),
                sheet_frame_width: Set(Some(sheet.sheet_width_px)),
                sheet_gap_size: Set(Some(lyra_timeline_preview::GAP_PX as i64)),
                sheet_interval: Set(Some(sheet.frame_interval_ms)),
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
        task: &tasks_entity::Model,
        _args: &Self::InputArgs,
    ) -> anyhow::Result<()> {
        let file_id = task
            .scope_id
            .parse::<i64>()
            .with_context(|| format!("invalid file id in scope_id '{}'", task.scope_id))?;

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
        assets::Entity::update_many()
            .filter(assets::Column::Id.is_in(stale_asset_ids))
            .set(assets::ActiveModel {
                deleted_at: Set(Some(now)),
                ..Default::default()
            })
            .exec(&tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }
}

async fn write_preview_sheet_to_disk(
    preview: TimelinePreview,
) -> anyhow::Result<PreparedPreviewSheet> {
    let hash_sha256 = hash_bytes_hex(&preview.preview_bytes);
    let output_path = get_asset_output_path(&hash_sha256);

    if let Some(parent) = output_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&output_path, &preview.preview_bytes).await?;

    Ok(PreparedPreviewSheet {
        hash_sha256,
        size_bytes: i64::try_from(preview.preview_bytes.len())
            .context("preview byte length exceeds i64")?,
        start_ms: duration_to_millis(preview.start_time)?,
        end_ms: duration_to_millis(preview.end_time)?,
        frame_interval_ms: duration_to_millis(preview.frame_interval)?,
        sheet_width_px: i64::from(preview.width_px),
    })
}

fn get_asset_output_path(hash_sha256: &str) -> PathBuf {
    let mut chars = hash_sha256.chars();
    let first = chars.next().unwrap().to_string();
    let second = chars.next().unwrap().to_string();

    get_config()
        .get_asset_store_dir()
        .join(first)
        .join(second)
        .join(format!("{hash_sha256}.webp"))
}

fn hash_bytes_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn duration_to_millis(duration: Duration) -> anyhow::Result<i64> {
    i64::try_from(duration.as_millis()).context("duration is too large to fit into i64 millis")
}
