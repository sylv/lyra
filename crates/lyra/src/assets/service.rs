use crate::{
    assets::storage,
    entities::assets::{self, AssetSource},
};
use anyhow::Context;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, EntityTrait};

const ALLOWED_REMOTE_URL_PREFIXES: &[&str] = &["https://image.tmdb.org/t/p"];

pub async fn create_local_asset_from_bytes<C: ConnectionTrait>(
    db: &C,
    image_bytes: &[u8],
) -> anyhow::Result<assets::Model> {
    let prepared = storage::prepare_image(image_bytes)?;
    storage::persist_image_bytes(image_bytes, &prepared).await?;

    let inserted = assets::Entity::insert(assets::ActiveModel {
        source: Set(AssetSource::Local),
        source_url: Set(None),
        hash_sha256: Set(Some(prepared.hash_sha256)),
        size_bytes: Set(Some(prepared.size_bytes)),
        mime_type: Set(Some(prepared.mime_type)),
        height: Set(Some(prepared.height)),
        width: Set(Some(prepared.width)),
        thumbhash: Set(None),
        deleted_at: Set(None),
        ..Default::default()
    })
    .exec_with_returning(db)
    .await?;

    Ok(inserted)
}

pub async fn create_local_asset_from_path<C: ConnectionTrait>(
    db: &C,
    image_path: &std::path::Path,
) -> anyhow::Result<assets::Model> {
    let bytes = tokio::fs::read(image_path)
        .await
        .with_context(|| format!("failed to read image at {}", image_path.display()))?;

    create_local_asset_from_bytes(db, &bytes).await
}

pub async fn download_asset_to_local<C: ConnectionTrait>(
    db: &C,
    asset: &assets::Model,
) -> anyhow::Result<assets::Model> {
    if asset.hash_sha256.is_some() {
        return Ok(asset.clone());
    }

    let source_url = asset
        .source_url
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("asset {} has no source_url", asset.id))?;

    if !is_url_allowed(source_url) {
        return Err(anyhow::anyhow!(
            "asset {} source_url is not allowed: {}",
            asset.id,
            source_url
        ));
    }

    let response = reqwest::get(source_url).await?.error_for_status()?;
    let bytes = response
        .bytes()
        .await
        .context("failed reading image response body")?;

    let prepared = storage::prepare_image(&bytes)?;
    storage::persist_image_bytes(&bytes, &prepared).await?;

    let mut updated: assets::ActiveModel = asset.clone().into();
    updated.hash_sha256 = Set(Some(prepared.hash_sha256));
    updated.size_bytes = Set(Some(prepared.size_bytes));
    updated.mime_type = Set(Some(prepared.mime_type));
    updated.width = Set(Some(prepared.width));
    updated.height = Set(Some(prepared.height));

    Ok(updated.update(db).await?)
}

fn is_url_allowed(url: &str) -> bool {
    ALLOWED_REMOTE_URL_PREFIXES
        .iter()
        .any(|allowed| url.starts_with(allowed))
}
