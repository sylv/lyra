use crate::entities::{
    file_assets::{self, FileAssetRole},
    metadata,
};
use async_graphql::{ComplexObject, Context, SimpleObject};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct NodeProperties {
    pub description: Option<String>,
    pub rating: Option<f64>,
    pub background_url: Option<String>,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
    pub runtime_minutes: Option<i64>,
    pub released_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    #[graphql(skip)]
    pub poster_asset_id: Option<i64>,
    #[graphql(skip)]
    pub thumbnail_asset_id: Option<i64>,
    #[graphql(skip)]
    pub file_id: Option<i64>,
}

#[ComplexObject]
impl NodeProperties {
    pub async fn poster_url(&self, ctx: &Context<'_>) -> Result<Option<String>, sea_orm::DbErr> {
        if let Some(poster_asset_id) = self.poster_asset_id {
            return Ok(Some(asset_url(poster_asset_id)));
        }

        if let Some(thumbnail_asset_id) = self.thumbnail_asset_id {
            return Ok(Some(asset_url(thumbnail_asset_id)));
        }

        let pool = ctx.data_unchecked::<DatabaseConnection>();
        Ok(self.file_thumbnail_asset_id(pool).await?.map(asset_url))
    }

    pub async fn thumbnail_url(&self, ctx: &Context<'_>) -> Result<Option<String>, sea_orm::DbErr> {
        if let Some(thumbnail_asset_id) = self.thumbnail_asset_id {
            return Ok(Some(asset_url(thumbnail_asset_id)));
        }

        let pool = ctx.data_unchecked::<DatabaseConnection>();
        Ok(self.file_thumbnail_asset_id(pool).await?.map(asset_url))
    }
}

impl NodeProperties {
    pub(crate) fn from_metadata(metadata: Option<metadata::Model>, file_id: Option<i64>) -> Self {
        let Some(metadata) = metadata else {
            return Self {
                description: None,
                rating: None,
                background_url: None,
                season_number: None,
                episode_number: None,
                runtime_minutes: None,
                released_at: None,
                ended_at: None,
                created_at: None,
                updated_at: None,
                poster_asset_id: None,
                thumbnail_asset_id: None,
                file_id,
            };
        };

        Self {
            description: metadata.description,
            rating: metadata.score_normalized.map(|score| score as f64 / 10.0),
            background_url: metadata
                .background_asset_id
                .map(|asset_id| format!("/api/assets/{asset_id}")),
            season_number: metadata.season_number,
            episode_number: metadata.episode_number,
            runtime_minutes: None,
            released_at: metadata.released_at,
            ended_at: metadata.ended_at,
            created_at: Some(metadata.created_at),
            updated_at: Some(metadata.updated_at),
            poster_asset_id: metadata.poster_asset_id,
            thumbnail_asset_id: metadata.thumbnail_asset_id,
            file_id,
        }
    }

    async fn file_thumbnail_asset_id(
        &self,
        pool: &DatabaseConnection,
    ) -> Result<Option<i64>, sea_orm::DbErr> {
        let Some(file_id) = self.file_id else {
            return Ok(None);
        };

        let file_thumbnail = file_assets::Entity::find()
            .filter(file_assets::Column::FileId.eq(file_id))
            .filter(file_assets::Column::Role.eq(FileAssetRole::Thumbnail))
            .order_by_desc(file_assets::Column::AssetId)
            .one(pool)
            .await?;

        Ok(file_thumbnail.map(|asset| asset.asset_id))
    }
}

fn asset_url(asset_id: i64) -> String {
    format!("/api/assets/{asset_id}")
}
