use crate::entities::media::{self, MediaType};
use async_graphql::{Context, InputObject, Object};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

#[derive(Debug, InputObject, serde::Deserialize)]
pub struct MediaFilter {
    pub parent_id: Option<i64>,
    pub season_numbers: Option<Vec<i64>>,
    pub search: Option<String>,
    pub media_types: Option<Vec<MediaType>>,
}

pub struct Query;

#[Object]
impl Query {
    async fn media_list(
        &self,
        ctx: &Context<'_>,
        filter: MediaFilter,
    ) -> Result<Vec<media::Model>, async_graphql::Error> {
        let mut query = media::Entity::find().order_by_desc(media::Column::StartDate);

        if let Some(parent_id) = filter.parent_id {
            query = query.filter(media::Column::ParentId.eq(parent_id));
        }

        if let Some(season_numbers) = filter.season_numbers {
            query = query.filter(media::Column::SeasonNumber.is_in(season_numbers));
        }

        if let Some(search) = filter.search {
            query = query.filter(media::Column::Name.contains(search));
        }

        let media_types = filter
            .media_types
            .unwrap_or_else(|| vec![MediaType::Movie, MediaType::Show]);
        query = query.filter(media::Column::MediaType.is_in(media_types));

        let pool = ctx.data::<DatabaseConnection>()?;
        let media = query
            .all(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(media)
    }

    async fn media(
        &self,
        ctx: &Context<'_>,
        media_id: i64,
    ) -> Result<media::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let media = media::Entity::find_by_id(media_id)
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("Media not found".to_string()))?;

        Ok(media)
    }
}
