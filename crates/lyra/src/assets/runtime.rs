use crate::{
    assets::{download_asset_to_local, storage},
    entities::assets,
};
use anyhow::Context;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    QueryFilter,
};
use std::{io::ErrorKind, time::Duration};
use tokio::time::sleep;

const POLL_INTERVAL: Duration = Duration::from_secs(10);

pub async fn start_asset_background_worker(pool: DatabaseConnection) -> anyhow::Result<()> {
    loop {
        let to_process = assets::Entity::find()
            .filter(
                Condition::any()
                    .add(
                        // for items that we haven't downloaded yet, we want to prioritize downloading them
                        Condition::all()
                            .add(assets::Column::SourceUrl.is_not_null())
                            .add(assets::Column::HashSha256.is_null()),
                    )
                    .add(
                        // for local items that need a thumbhash generated
                        Condition::all()
                            .add(assets::Column::HashSha256.is_not_null())
                            .add(
                                Condition::any()
                                    .add(assets::Column::Thumbhash.is_null())
                                    .add(assets::Column::Thumbhash.eq(Vec::<u8>::new())),
                            ),
                    )
                    .add(
                        // for items that have been deleted, we want to prioritize cleaning them up
                        Condition::all().add(assets::Column::DeletedAt.is_not_null()),
                    ),
            )
            .one(&pool)
            .await?;

        if let Some(to_process) = to_process {
            if to_process.deleted_at.is_some() {
                tracing::info!("deleting asset {}", to_process.id);
                cleanup_deleted_assets(&pool, &to_process).await?;
            } else {
                if to_process.hash_sha256.is_none() {
                    tracing::info!("downloading asset {}", to_process.id);
                    // todo: if eg tmdb is down or our network is down, this will cause the process to crash
                    // we should instead detcct errors and disable downloads for say, an hour.
                    download_asset_to_local(&pool, &to_process).await?;
                } else {
                    tracing::info!("generating thumbhash for asset {}", to_process.id);
                    generate_thumbhash_for(&pool, &to_process).await?;
                }
            }
        } else {
            sleep(POLL_INTERVAL).await;
        }
    }
}

async fn cleanup_deleted_assets(
    pool: &DatabaseConnection,
    asset: &assets::Model,
) -> anyhow::Result<()> {
    if let (Some(hash_sha256), Some(mime_type)) =
        (asset.hash_sha256.as_deref(), asset.mime_type.as_deref())
    {
        let output_path = storage::get_asset_output_path_from_mime(hash_sha256, mime_type)?;
        match tokio::fs::remove_file(output_path).await {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }

    assets::Entity::delete_by_id(asset.id).exec(pool).await?;

    Ok(())
}

async fn generate_thumbhash_for(
    pool: &DatabaseConnection,
    asset: &assets::Model,
) -> anyhow::Result<()> {
    let hash_sha256 = asset
        .hash_sha256
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("asset {} is missing hash_sha256", asset.id))?;
    let mime_type = asset
        .mime_type
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("asset {} is missing mime_type", asset.id))?;
    let image_path = storage::get_asset_output_path_from_mime(hash_sha256, mime_type)?;
    let bytes = tokio::fs::read(&image_path)
        .await
        .with_context(|| format!("failed to read image at {}", image_path.display()))?;

    let format = image::guess_format(&bytes).context("failed to guess image format")?;
    let decoded =
        image::load_from_memory_with_format(&bytes, format).context("failed to decode image")?;
    let resized = decoded.thumbnail(100, 100).to_rgba8();

    let width = usize::try_from(resized.width()).context("image width exceeds usize")?;
    let height = usize::try_from(resized.height()).context("image height exceeds usize")?;
    let thumbhash = thumbhash::rgba_to_thumb_hash(width, height, resized.as_raw());

    let mut updated: assets::ActiveModel = asset.clone().into();
    updated.thumbhash = Set(Some(thumbhash));
    updated.update(pool).await?;

    Ok(())
}
