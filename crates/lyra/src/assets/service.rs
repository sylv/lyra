use crate::{
    assets::storage,
    entities::assets::{self, AssetKind, AssetType},
    ids,
};
use anyhow::Context;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, EntityTrait};

pub async fn create_local_asset_from_bytes<C: ConnectionTrait>(
    db: &C,
    image_bytes: &[u8],
    kind: AssetKind,
) -> anyhow::Result<assets::Model> {
    let prepared = storage::prepare_image(image_bytes)?;
    storage::persist_image_bytes(image_bytes, &prepared).await?;
    let now = chrono::Utc::now().timestamp();
    let asset_id = ids::generate_prefixed_hashid("a", [prepared.hash_sha256.as_str()]);

    if let Some(existing) = assets::Entity::find_by_id(asset_id.clone()).one(db).await? {
        let mut updated: assets::ActiveModel = existing.into();
        updated.updated_at = Set(Some(now));
        return Ok(updated.update(db).await?);
    }

    let inserted = assets::Entity::insert(assets::ActiveModel {
        id: Set(asset_id),
        kind: Set(kind),
        asset_type: Set(AssetType::Image),
        source_url: Set(None),
        hash_sha256: Set(Some(prepared.hash_sha256)),
        size_bytes: Set(Some(prepared.size_bytes)),
        uncompressed_size_bytes: Set(Some(prepared.size_bytes)),
        mime_type: Set(Some(prepared.mime_type)),
        content_encoding: Set(None),
        height: Set(Some(prepared.height)),
        width: Set(Some(prepared.width)),
        thumbhash: Set(None),
        created_at: Set(now),
        updated_at: Set(Some(now)),
        ..Default::default()
    })
    .exec_with_returning(db)
    .await?;

    Ok(inserted)
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

    let response = reqwest::get(source_url).await?.error_for_status()?;
    let bytes = response
        .bytes()
        .await
        .context("failed reading image response body")?;

    let prepared = storage::prepare_image(&bytes)?;
    storage::persist_image_bytes(&bytes, &prepared).await?;
    let now = chrono::Utc::now().timestamp();

    let mut updated: assets::ActiveModel = asset.clone().into();
    updated.hash_sha256 = Set(Some(prepared.hash_sha256));
    updated.size_bytes = Set(Some(prepared.size_bytes));
    updated.uncompressed_size_bytes = Set(Some(prepared.size_bytes));
    updated.mime_type = Set(Some(prepared.mime_type));
    updated.content_encoding = Set(None);
    updated.width = Set(Some(prepared.width));
    updated.height = Set(Some(prepared.height));
    updated.updated_at = Set(Some(now));

    Ok(updated.update(db).await?)
}

pub async fn create_local_file_asset_from_bytes<C: ConnectionTrait>(
    db: &C,
    file_bytes: &[u8],
    mime_type: &str,
    kind: AssetKind,
) -> anyhow::Result<assets::Model> {
    let compressed_bytes = zstd::encode_all(std::io::Cursor::new(file_bytes), 12)
        .context("failed to zstd-compress file asset")?;
    let prepared = storage::prepare_file_bytes(&compressed_bytes, mime_type, Some("zstd"))?;
    storage::persist_bytes_atomically(
        &storage::get_asset_output_path(&prepared.hash_sha256, &prepared.extension)?,
        &compressed_bytes,
    )
    .await?;
    let now = chrono::Utc::now().timestamp();
    let asset_id = ids::generate_prefixed_hashid("a", [prepared.hash_sha256.as_str()]);

    if let Some(existing) = assets::Entity::find_by_id(asset_id.clone()).one(db).await? {
        let mut updated: assets::ActiveModel = existing.into();
        updated.updated_at = Set(Some(now));
        return Ok(updated.update(db).await?);
    }

    let inserted = assets::Entity::insert(assets::ActiveModel {
        id: Set(asset_id),
        kind: Set(kind),
        asset_type: Set(AssetType::File),
        source_url: Set(None),
        hash_sha256: Set(Some(prepared.hash_sha256)),
        size_bytes: Set(Some(prepared.size_bytes)),
        uncompressed_size_bytes: Set(Some(
            i64::try_from(file_bytes.len()).context("file byte length exceeds i64")?,
        )),
        mime_type: Set(Some(prepared.mime_type)),
        content_encoding: Set(prepared.content_encoding),
        height: Set(None),
        width: Set(None),
        thumbhash: Set(None),
        created_at: Set(now),
        updated_at: Set(Some(now)),
        ..Default::default()
    })
    .exec_with_returning(db)
    .await?;

    Ok(inserted)
}
