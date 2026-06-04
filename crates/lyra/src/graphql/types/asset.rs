use crate::assets::sign_asset_url;
use crate::entities::images;
use crate::graphql::properties::Asset;
use async_graphql::ComplexObject;

#[ComplexObject]
impl Asset {
    pub async fn signed_url(&self) -> async_graphql::Result<String> {
        Ok(sign_asset_url(&self.id))
    }

    pub async fn aspect_ratio(&self) -> Option<String> {
        match (self.width, self.height) {
            (Some(w), Some(h)) if h != 0 => Some(format!("{w} / {h}")),
            _ => None,
        }
    }
}

impl From<images::Model> for Asset {
    fn from(model: images::Model) -> Self {
        Self {
            id: model.id,
            source_url: model.source_url,
            hash_sha256: None,
            size_bytes: model.size_bytes,
            uncompressed_size_bytes: model.size_bytes,
            mime_type: model.mime_type,
            content_encoding: None,
            height: model.height,
            width: model.width,
            thumbhash: model.thumbhash.map(hex::encode),
            created_at: model.created_at,
        }
    }
}
