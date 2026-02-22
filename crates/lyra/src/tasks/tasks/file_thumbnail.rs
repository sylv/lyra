use crate::{
    assets as assets_api,
    entities::{
        assets as assets_entity,
        file_assets::{self, FileAssetRole},
        files, libraries, tasks as tasks_entity,
    },
    ffmpeg,
    tasks::{TaskHandler, TaskLike, TaskScopeKind},
};
use anyhow::Context;
use lyra_thumbnail::{ThumbnailOptions, generate_thumbnail};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FileThumbnailTaskArgs;

#[derive(Debug, Default)]
pub struct FileThumbnailTask;

#[async_trait::async_trait]
impl TaskHandler for FileThumbnailTask {
    type InputArgs = FileThumbnailTaskArgs;

    fn task_type(&self) -> &'static str {
        "file.generate_thumbnail"
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
                "file path missing while generating thumbnail"
            );

            return Ok(());
        }

        let thumbnail_options = ThumbnailOptions {
            ffmpeg_bin: PathBuf::from(ffmpeg::get_ffmpeg_path()),
            ..ThumbnailOptions::default()
        };
        let thumbnail = generate_thumbnail(&file_path, &thumbnail_options).await?;

        let mut tx = pool.begin().await?;
        let asset = assets_api::create_local_asset_from_bytes(&tx, &thumbnail.image_bytes).await?;

        file_assets::Entity::insert(file_assets::ActiveModel {
            file_id: Set(file.id),
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
            .filter(file_assets::Column::Role.eq(FileAssetRole::Thumbnail))
            .all(&tx)
            .await?
            .into_iter()
            .map(|row| row.asset_id)
            .collect();

        file_assets::Entity::delete_many()
            .filter(file_assets::Column::FileId.eq(file_id))
            .filter(file_assets::Column::Role.eq(FileAssetRole::Thumbnail))
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
