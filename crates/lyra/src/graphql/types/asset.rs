use crate::assets::sign_asset_url;
use crate::entities::assets;
use crate::graphql::properties::Asset;
use async_graphql::ComplexObject;

#[ComplexObject]
impl Asset {
    pub async fn signed_url(&self) -> async_graphql::Result<String> {
        Ok(sign_asset_url(&self.id))
    }
}

impl From<assets::Model> for Asset {
    fn from(model: assets::Model) -> Self {
        Self {
            id: model.id,
            source_url: model.source_url,
            hash_sha256: model.hash_sha256,
            size_bytes: model.size_bytes,
            uncompressed_size_bytes: model.uncompressed_size_bytes,
            mime_type: model.mime_type,
            content_encoding: model.content_encoding,
            height: model.height,
            width: model.width,
            thumbhash: model.thumbhash.map(hex::encode),
            created_at: model.created_at,
        }
    }
}
