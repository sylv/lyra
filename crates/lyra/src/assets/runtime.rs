use crate::{
    assets::{download_asset_to_local, storage},
    entities::assets,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};
use std::{io::ErrorKind, time::Duration};

const POLL_INTERVAL: Duration = Duration::from_secs(10);
const BATCH_SIZE: u64 = 8;

pub async fn start_asset_background_worker(pool: DatabaseConnection) -> anyhow::Result<()> {
    loop {
        if let Err(error) = cleanup_deleted_assets(&pool).await {
            tracing::error!(error = ?error, "asset cleanup pass failed");
        }

        if let Err(error) = materialize_remote_assets(&pool).await {
            tracing::error!(error = ?error, "asset download pass failed");
        }

        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

async fn cleanup_deleted_assets(pool: &DatabaseConnection) -> anyhow::Result<()> {
    let stale_assets = assets::Entity::find()
        .filter(assets::Column::DeletedAt.is_not_null())
        .limit(BATCH_SIZE)
        .all(pool)
        .await?;

    for asset in stale_assets {
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
    }

    Ok(())
}

async fn materialize_remote_assets(pool: &DatabaseConnection) -> anyhow::Result<()> {
    let pending_assets = assets::Entity::find()
        .filter(assets::Column::DeletedAt.is_null())
        .filter(assets::Column::SourceUrl.is_not_null())
        .filter(assets::Column::HashSha256.is_null())
        .limit(BATCH_SIZE)
        .all(pool)
        .await?;

    for asset in pending_assets {
        if let Err(error) = download_asset_to_local(pool, &asset).await {
            tracing::warn!(asset_id = asset.id, error = ?error, "failed to download remote asset");
        }
    }

    Ok(())
}
