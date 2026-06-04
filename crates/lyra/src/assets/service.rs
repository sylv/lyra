use crate::entities::images::{self, ImageKind, ImageSource};
use anyhow::Context;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, EntityTrait};
use sha2::{Digest, Sha256};

pub async fn create_local_asset_from_bytes<C: ConnectionTrait>(
    db: &C,
    image_bytes: &[u8],
    kind: ImageKind,
) -> anyhow::Result<images::Model> {
    let prepared = super::storage::prepare_image(image_bytes)?;
    let now = chrono::Utc::now().timestamp();
    let image_id = ulid::Ulid::new().to_string();
    super::storage::persist_image_bytes(&image_id, image_bytes, &prepared).await?;

    let inserted = images::Entity::insert(images::ActiveModel {
        id: Set(image_id),
        kind: Set(kind),
        source: Set(ImageSource::Local),
        source_url: Set(None),
        size_bytes: Set(Some(prepared.size_bytes)),
        mime_type: Set(Some(prepared.mime_type)),
        height: Set(Some(prepared.height)),
        width: Set(Some(prepared.width)),
        thumbhash: Set(None),
        created_at: Set(now),
        updated_at: Set(Some(now)),
    })
    .exec_with_returning(db)
    .await?;

    Ok(inserted)
}

pub async fn download_asset_to_local<C: ConnectionTrait>(
    db: &C,
    image: &images::Model,
) -> anyhow::Result<images::Model> {
    if image.mime_type.is_some() {
        return Ok(image.clone());
    }

    let source_url = image
        .source_url
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("image {} has no source_url", image.id))?;

    let output_path = super::storage::get_asset_output_path(&image.id)?;
    let bytes = if tokio::fs::try_exists(&output_path).await? {
        tokio::fs::read(&output_path).await?
    } else {
        let response = reqwest::get(source_url).await?.error_for_status()?;
        let bytes = response
            .bytes()
            .await
            .context("failed reading image response body")?;
        bytes.to_vec()
    };

    let prepared = super::storage::prepare_image(&bytes)?;
    super::storage::persist_image_bytes(&image.id, &bytes, &prepared).await?;
    let now = chrono::Utc::now().timestamp();

    let mut updated: images::ActiveModel = image.clone().into();
    updated.size_bytes = Set(Some(prepared.size_bytes));
    updated.mime_type = Set(Some(prepared.mime_type));
    updated.width = Set(Some(prepared.width));
    updated.height = Set(Some(prepared.height));
    updated.updated_at = Set(Some(now));

    Ok(updated.update(db).await?)
}

pub fn remote_image_id(source_url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source_url.as_bytes());
    hex::encode(hasher.finalize())
}
