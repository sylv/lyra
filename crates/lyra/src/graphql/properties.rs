use crate::entities::{
    file_assets::{self, FileAssetRole},
    item_files, item_metadata, root_metadata, season_metadata,
};
use async_graphql::{ComplexObject, Context, SimpleObject};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct RootNodeProperties {
    pub description: Option<String>,
    pub rating: Option<f64>,
    pub background_url: Option<String>,
    pub runtime_minutes: Option<i64>,
    pub released_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    #[graphql(skip)]
    pub poster_asset_id: Option<i64>,
    #[graphql(skip)]
    pub thumbnail_asset_id: Option<i64>,
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct SeasonNodeProperties {
    pub description: Option<String>,
    pub rating: Option<f64>,
    pub background_url: Option<String>,
    pub season_number: Option<i64>,
    pub runtime_minutes: Option<i64>,
    pub released_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    #[graphql(skip)]
    pub poster_asset_id: Option<i64>,
    #[graphql(skip)]
    pub thumbnail_asset_id: Option<i64>,
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct ItemNodeProperties {
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
    pub item_id: String,
}

#[ComplexObject]
impl RootNodeProperties {
    pub async fn poster_url(&self) -> Option<String> {
        self.poster_asset_id
            .or(self.thumbnail_asset_id)
            .map(asset_url)
    }

    pub async fn thumbnail_url(&self) -> Option<String> {
        self.thumbnail_asset_id.map(asset_url)
    }
}

#[ComplexObject]
impl SeasonNodeProperties {
    pub async fn poster_url(&self) -> Option<String> {
        self.poster_asset_id
            .or(self.thumbnail_asset_id)
            .map(asset_url)
    }

    pub async fn thumbnail_url(&self) -> Option<String> {
        self.thumbnail_asset_id.map(asset_url)
    }
}

#[ComplexObject]
impl ItemNodeProperties {
    pub async fn poster_url(&self, ctx: &Context<'_>) -> Result<Option<String>, sea_orm::DbErr> {
        if let Some(asset_id) = self.poster_asset_id.or(self.thumbnail_asset_id) {
            return Ok(Some(asset_url(asset_id)));
        }

        let pool = ctx.data_unchecked::<DatabaseConnection>();
        Ok(self.file_thumbnail_asset_id(pool).await?.map(asset_url))
    }

    pub async fn thumbnail_url(&self, ctx: &Context<'_>) -> Result<Option<String>, sea_orm::DbErr> {
        if let Some(asset_id) = self.thumbnail_asset_id {
            return Ok(Some(asset_url(asset_id)));
        }

        let pool = ctx.data_unchecked::<DatabaseConnection>();
        Ok(self.file_thumbnail_asset_id(pool).await?.map(asset_url))
    }
}

impl RootNodeProperties {
    pub(crate) fn from_metadata(metadata: Option<root_metadata::Model>) -> Self {
        let Some(metadata) = metadata else {
            return Self {
                description: None,
                rating: None,
                background_url: None,
                runtime_minutes: None,
                released_at: None,
                ended_at: None,
                created_at: None,
                updated_at: None,
                poster_asset_id: None,
                thumbnail_asset_id: None,
            };
        };

        Self {
            description: metadata.description,
            rating: metadata.score_normalized.map(|score| score as f64 / 10.0),
            background_url: metadata.background_asset_id.map(asset_url),
            runtime_minutes: None,
            released_at: metadata.released_at,
            ended_at: metadata.ended_at,
            created_at: Some(metadata.created_at),
            updated_at: Some(metadata.updated_at),
            poster_asset_id: metadata.poster_asset_id,
            thumbnail_asset_id: metadata.thumbnail_asset_id,
        }
    }
}

impl SeasonNodeProperties {
    pub(crate) fn from_metadata(
        metadata: Option<season_metadata::Model>,
        season_number: Option<i64>,
    ) -> Self {
        let Some(metadata) = metadata else {
            return Self {
                description: None,
                rating: None,
                background_url: None,
                season_number,
                runtime_minutes: None,
                released_at: None,
                ended_at: None,
                created_at: None,
                updated_at: None,
                poster_asset_id: None,
                thumbnail_asset_id: None,
            };
        };

        Self {
            description: metadata.description,
            rating: metadata.score_normalized.map(|score| score as f64 / 10.0),
            background_url: metadata.background_asset_id.map(asset_url),
            season_number,
            runtime_minutes: None,
            released_at: metadata.released_at,
            ended_at: metadata.ended_at,
            created_at: Some(metadata.created_at),
            updated_at: Some(metadata.updated_at),
            poster_asset_id: metadata.poster_asset_id,
            thumbnail_asset_id: metadata.thumbnail_asset_id,
        }
    }
}

impl ItemNodeProperties {
    pub(crate) fn from_metadata(
        metadata: Option<item_metadata::Model>,
        item_id: String,
        season_number: Option<i64>,
        episode_number: Option<i64>,
    ) -> Self {
        let Some(metadata) = metadata else {
            return Self {
                description: None,
                rating: None,
                background_url: None,
                season_number,
                episode_number,
                runtime_minutes: None,
                released_at: None,
                ended_at: None,
                created_at: None,
                updated_at: None,
                poster_asset_id: None,
                thumbnail_asset_id: None,
                item_id,
            };
        };

        Self {
            description: metadata.description,
            rating: metadata.score_normalized.map(|score| score as f64 / 10.0),
            background_url: metadata.background_asset_id.map(asset_url),
            season_number,
            episode_number,
            runtime_minutes: None,
            released_at: metadata.released_at,
            ended_at: metadata.ended_at,
            created_at: Some(metadata.created_at),
            updated_at: Some(metadata.updated_at),
            poster_asset_id: metadata.poster_asset_id,
            thumbnail_asset_id: metadata.thumbnail_asset_id,
            item_id,
        }
    }

    async fn file_thumbnail_asset_id(
        &self,
        pool: &DatabaseConnection,
    ) -> Result<Option<i64>, sea_orm::DbErr> {
        let links = item_files::Entity::find()
            .filter(item_files::Column::ItemId.eq(self.item_id.clone()))
            .order_by_asc(item_files::Column::Order)
            .order_by_asc(item_files::Column::FileId)
            .all(pool)
            .await?;

        for link in links {
            let thumbnail = file_assets::Entity::find()
                .filter(file_assets::Column::FileId.eq(link.file_id))
                .filter(file_assets::Column::Role.eq(FileAssetRole::Thumbnail))
                .order_by_desc(file_assets::Column::AssetId)
                .one(pool)
                .await?;

            if let Some(thumbnail) = thumbnail {
                return Ok(Some(thumbnail.asset_id));
            }
        }

        Ok(None)
    }
}

fn asset_url(asset_id: i64) -> String {
    format!("/api/assets/{asset_id}")
}
