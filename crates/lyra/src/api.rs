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

    // async fn get_seasons(
    //     State(state): State<AppState>,
    //     show_id: i64,
    // ) -> Result<Vec<i32>, RpcError> {
    //     let media_type = MediaType::Season.as_int();
    //     let seasons = sqlx::query_scalar!(
    //         "SELECT DISTINCT season_number FROM media WHERE parent_id = ? AND media_type = ?",
    //         show_id,
    //         media_type
    //     )
    //     .fetch_all(&state.pool)
    //     .await
    //     .map_err(|e| RpcError::new(RpcStatus::InternalServerError, e.to_string()))?;

    //     Ok(seasons
    //         .into_iter()
    //         .filter_map(|s| s.map(|v| v as i32))
    //         .collect())
    // }

    // async fn get_season_episodes(
    //     State(state): State<AppState>,
    //     show_id: i64,
    //     season_numbers: Vec<i64>,
    // ) -> Result<Vec<MediaWithFirstConnection>, RpcError> {
    //     let episode_media_type = MediaType::Episode.as_int();

    //     // build the query using QueryBuilder for the IN clause
    //     let mut query_builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
    //         r#"
    //     SELECT e.id, e.name, e.description, e.poster_url, e.background_url, e.thumbnail_url,
    //            e.parent_id, e.media_type, e.tmdb_parent_id, e.tmdb_item_id, e.rating,
    //            e.release_date, e.runtime_minutes, e.season_number, e.episode_number
    //     FROM media e
    //     JOIN media s ON e.parent_id = s.id
    //     WHERE s.parent_id = "#,
    //     );

    //     query_builder.push_bind(show_id);
    //     query_builder.push(" AND e.media_type = ");
    //     query_builder.push_bind(episode_media_type);
    //     query_builder.push(" AND s.season_number IN (");

    //     let mut separated = query_builder.separated(", ");
    //     for season_num in season_numbers.iter() {
    //         separated.push_bind(season_num);
    //     }
    //     separated.push_unseparated(") ORDER BY e.season_number, e.episode_number");

    //     let episodes = query_builder
    //         .build_query_as::<Media>()
    //         .fetch_all(&state.pool)
    //         .await
    //         .map_err(|e| RpcError::new(RpcStatus::InternalServerError, e.to_string()))?;

    //     // get default connections for all episodes
    //     let episode_ids: Vec<i64> = episodes.iter().map(|e| e.id).collect();
    //     let default_connections = Media::find_default_connections(&state.pool, &episode_ids)
    //         .await
    //         .map_err(|e| RpcError::new(RpcStatus::InternalServerError, e.to_string()))?;

    //     // combine episodes with their default connections
    //     let episodes_with_connections = episodes
    //         .into_iter()
    //         .map(|e| MediaWithFirstConnection {
    //             default_connection: default_connections.get(&e.id).cloned(),
    //             media: e,
    //         })
    //         .collect();

    //     Ok(episodes_with_connections)
    // }

    // pub fn get_api_router() -> Router<AppState> {
    //     RpcRouter::new()
    //         .for_state::<AppState>()
    //         .add(get_all_media)
    //         .add(get_media_by_id)
    //         .add(get_season_episodes)
    //         .add(get_seasons)
    //         .write_client("client/src/@generated/server.ts")
    //         .unwrap()
    //         .to_router()
    // }
}
