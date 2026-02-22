use crate::entities::{
    assets,
    file_assets::{self, FileAssetRole},
    item_files, item_metadata, root_metadata, season_metadata,
};
use async_graphql::{ComplexObject, Context, SimpleObject};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

#[derive(Clone, Debug, SimpleObject)]
pub struct Asset {
    pub id: i64,
    pub source: assets::AssetSource,
    pub source_url: Option<String>,
    pub hash_sha256: Option<String>,
    pub size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub height: Option<i64>,
    pub width: Option<i64>,
    pub thumbhash: Option<String>,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct RootNodeProperties {
    pub description: Option<String>,
    pub rating: Option<f64>,
    pub runtime_minutes: Option<i64>,
    pub released_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    #[graphql(skip)]
    pub background_asset_id: Option<i64>,
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
    pub season_number: Option<i64>,
    pub runtime_minutes: Option<i64>,
    pub released_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    #[graphql(skip)]
    pub background_asset_id: Option<i64>,
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
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
    pub runtime_minutes: Option<i64>,
    pub released_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    #[graphql(skip)]
    pub background_asset_id: Option<i64>,
    #[graphql(skip)]
    pub poster_asset_id: Option<i64>,
    #[graphql(skip)]
    pub thumbnail_asset_id: Option<i64>,
    #[graphql(skip)]
    pub item_id: String,
}

#[ComplexObject]
impl RootNodeProperties {
    pub async fn background_image(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_asset(pool, self.background_asset_id).await
    }

    pub async fn poster_image(&self, ctx: &Context<'_>) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_asset(pool, self.poster_asset_id.or(self.thumbnail_asset_id)).await
    }

    pub async fn thumbnail_image(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_asset(pool, self.thumbnail_asset_id).await
    }
}

#[ComplexObject]
impl SeasonNodeProperties {
    pub async fn background_image(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_asset(pool, self.background_asset_id).await
    }

    pub async fn poster_image(&self, ctx: &Context<'_>) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_asset(pool, self.poster_asset_id.or(self.thumbnail_asset_id)).await
    }

    pub async fn thumbnail_image(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_asset(pool, self.thumbnail_asset_id).await
    }
}

#[ComplexObject]
impl ItemNodeProperties {
    pub async fn background_image(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_asset(pool, self.background_asset_id).await
    }

    pub async fn poster_image(&self, ctx: &Context<'_>) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        if let Some(asset_id) = self.poster_asset_id.or(self.thumbnail_asset_id) {
            return find_asset(pool, Some(asset_id)).await;
        }

        let asset_id = self.file_thumbnail_asset_id(pool).await?;
        find_asset(pool, asset_id).await
    }

    pub async fn thumbnail_image(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        if let Some(asset_id) = self.thumbnail_asset_id {
            return find_asset(pool, Some(asset_id)).await;
        }

        let asset_id = self.file_thumbnail_asset_id(pool).await?;
        find_asset(pool, asset_id).await
    }
}

impl RootNodeProperties {
    pub(crate) fn from_metadata(metadata: Option<root_metadata::Model>) -> Self {
        let Some(metadata) = metadata else {
            return Self {
                description: None,
                rating: None,
                runtime_minutes: None,
                released_at: None,
                ended_at: None,
                created_at: None,
                updated_at: None,
                background_asset_id: None,
                poster_asset_id: None,
                thumbnail_asset_id: None,
            };
        };

        Self {
            description: metadata.description,
            rating: metadata.score_normalized.map(|score| score as f64 / 10.0),
            runtime_minutes: None,
            released_at: metadata.released_at,
            ended_at: metadata.ended_at,
            created_at: Some(metadata.created_at),
            updated_at: Some(metadata.updated_at),
            background_asset_id: metadata.background_asset_id,
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
                season_number,
                runtime_minutes: None,
                released_at: None,
                ended_at: None,
                created_at: None,
                updated_at: None,
                background_asset_id: None,
                poster_asset_id: None,
                thumbnail_asset_id: None,
            };
        };

        Self {
            description: metadata.description,
            rating: metadata.score_normalized.map(|score| score as f64 / 10.0),
            season_number,
            runtime_minutes: None,
            released_at: metadata.released_at,
            ended_at: metadata.ended_at,
            created_at: Some(metadata.created_at),
            updated_at: Some(metadata.updated_at),
            background_asset_id: metadata.background_asset_id,
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
                season_number,
                episode_number,
                runtime_minutes: None,
                released_at: None,
                ended_at: None,
                created_at: None,
                updated_at: None,
                background_asset_id: None,
                poster_asset_id: None,
                thumbnail_asset_id: None,
                item_id,
            };
        };

        Self {
            description: metadata.description,
            rating: metadata.score_normalized.map(|score| score as f64 / 10.0),
            season_number,
            episode_number,
            runtime_minutes: None,
            released_at: metadata.released_at,
            ended_at: metadata.ended_at,
            created_at: Some(metadata.created_at),
            updated_at: Some(metadata.updated_at),
            background_asset_id: metadata.background_asset_id,
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

impl Asset {
    fn from_model(model: assets::Model) -> Self {
        Self {
            id: model.id,
            source: model.source,
            source_url: model.source_url,
            hash_sha256: model.hash_sha256,
            size_bytes: model.size_bytes,
            mime_type: model.mime_type,
            height: model.height,
            width: model.width,
            thumbhash: model.thumbhash.map(hex::encode),
            created_at: model.created_at,
            deleted_at: model.deleted_at,
        }
    }
}

async fn find_asset(
    pool: &DatabaseConnection,
    asset_id: Option<i64>,
) -> Result<Option<Asset>, sea_orm::DbErr> {
    let Some(asset_id) = asset_id else {
        return Ok(None);
    };

    let model = assets::Entity::find_by_id(asset_id).one(pool).await?;
    Ok(model.map(Asset::from_model))
}
