use crate::AppState;
use crate::entities::file::File;
use crate::entities::media::{Media, MediaType};
use axum::Router;
use axum::extract::State;
use juno::errors::{RpcError, RpcStatus};
use juno::router::RpcRouter;
use juno::rpc;
use serde::Serialize;
use sqlx::QueryBuilder;

#[derive(specta::Type, Serialize)]
pub struct MediaWithFirstConnection {
    pub media: Media,
    pub default_connection: Option<File>,
}

#[derive(specta::Type, Serialize)]
pub struct MediaDetails {
    pub media: Media,
    pub default_connection: Option<File>,
    pub connections: Vec<File>,
}

#[derive(Debug, specta::Type, serde::Deserialize)]
pub struct GetAllMediaFilter {
    pub parent_id: Option<i64>,
    pub search: Option<String>,
    pub media_types: Option<Vec<MediaType>>,
}

#[rpc(query)]
async fn get_all_media(
    State(state): State<AppState>,
    filter: GetAllMediaFilter,
) -> Result<Vec<MediaWithFirstConnection>, RpcError> {
    let mut query_builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
        "SELECT id, name, description, poster_url, background_url, thumbnail_url, parent_id, media_type, tmdb_parent_id, tmdb_item_id, rating, release_date, runtime_minutes, season_number, episode_number FROM media WHERE ",
    );

    // handle parent_id condition
    if let Some(parent_id) = filter.parent_id {
        query_builder.push("parent_id = ");
        query_builder.push_bind(parent_id);
    } else {
        query_builder.push("parent_id IS NULL");
    }

    // handle search condition
    if let Some(search) = filter.search {
        if !search.trim().is_empty() {
            query_builder.push(" AND (name LIKE ");
            query_builder.push_bind(format!("%{}%", search));
            query_builder.push(" OR description LIKE ");
            query_builder.push_bind(format!("%{}%", search));
            query_builder.push(")");
        }
    }

    // handle media_types filter
    if let Some(media_types) = filter.media_types {
        if !media_types.is_empty() {
            query_builder.push(" AND media_type IN (");
            let mut separated = query_builder.separated(", ");
            for media_type in media_types.iter() {
                separated.push_bind(media_type.as_int());
            }
            separated.push_unseparated(")");
        }
    }

    let media = query_builder
        .build_query_as::<Media>()
        .fetch_all(&state.pool)
        .await
        .map_err(|e| RpcError::new(RpcStatus::InternalServerError, e.to_string()))?;

    // Get default connections for all media items
    let media_ids: Vec<i64> = media.iter().map(|m| m.id).collect();
    let default_connections = Media::find_default_connections(&state.pool, &media_ids)
        .await
        .map_err(|e| RpcError::new(RpcStatus::InternalServerError, e.to_string()))?;

    // Combine media with their default connections
    let media_with_connections = media
        .into_iter()
        .map(|m| MediaWithFirstConnection {
            default_connection: default_connections.get(&m.id).cloned(),
            media: m,
        })
        .collect();

    Ok(media_with_connections)
}

#[rpc(query)]
async fn get_media_by_id(
    State(state): State<AppState>,
    media_id: i64,
) -> Result<MediaDetails, RpcError> {
    let media = Media::find_by_id(&state.pool, media_id)
        .await
        .map_err(|e| RpcError::new(RpcStatus::InternalServerError, e.to_string()))?
        .ok_or_else(|| RpcError::new(RpcStatus::NotFound, "Media not found".to_string()))?;

    let connections = File::find_by_media_id(&state.pool, media.id)
        .await
        .map_err(|e| RpcError::new(RpcStatus::InternalServerError, e.to_string()))?;

    let default_connection = Media::find_default_connections(&state.pool, &[media.id])
        .await
        .map_err(|e| RpcError::new(RpcStatus::InternalServerError, e.to_string()))?
        .into_iter()
        .next()
        .map(|(_, file)| file);

    Ok(MediaDetails {
        media,
        connections,
        default_connection,
    })
}

#[rpc(query)]
async fn get_seasons(State(state): State<AppState>, show_id: i64) -> Result<Vec<i32>, RpcError> {
    let media_type = MediaType::Season.as_int();
    let seasons = sqlx::query_scalar!(
        "SELECT DISTINCT season_number FROM media WHERE parent_id = ? AND media_type = ?",
        show_id,
        media_type
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| RpcError::new(RpcStatus::InternalServerError, e.to_string()))?;

    Ok(seasons
        .into_iter()
        .filter_map(|s| s.map(|v| v as i32))
        .collect())
}

#[rpc(query)]
async fn get_season_episodes(
    State(state): State<AppState>,
    show_id: i64,
    season_numbers: Vec<i64>,
) -> Result<Vec<MediaWithFirstConnection>, RpcError> {
    let episode_media_type = MediaType::Episode.as_int();

    // build the query using QueryBuilder for the IN clause
    let mut query_builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
        r#"
        SELECT e.id, e.name, e.description, e.poster_url, e.background_url, e.thumbnail_url, 
               e.parent_id, e.media_type, e.tmdb_parent_id, e.tmdb_item_id, e.rating, 
               e.release_date, e.runtime_minutes, e.season_number, e.episode_number
        FROM media e
        JOIN media s ON e.parent_id = s.id
        WHERE s.parent_id = "#,
    );

    query_builder.push_bind(show_id);
    query_builder.push(" AND e.media_type = ");
    query_builder.push_bind(episode_media_type);
    query_builder.push(" AND s.season_number IN (");

    let mut separated = query_builder.separated(", ");
    for season_num in season_numbers.iter() {
        separated.push_bind(season_num);
    }
    separated.push_unseparated(") ORDER BY e.season_number, e.episode_number");

    let episodes = query_builder
        .build_query_as::<Media>()
        .fetch_all(&state.pool)
        .await
        .map_err(|e| RpcError::new(RpcStatus::InternalServerError, e.to_string()))?;

    // get default connections for all episodes
    let episode_ids: Vec<i64> = episodes.iter().map(|e| e.id).collect();
    let default_connections = Media::find_default_connections(&state.pool, &episode_ids)
        .await
        .map_err(|e| RpcError::new(RpcStatus::InternalServerError, e.to_string()))?;

    // combine episodes with their default connections
    let episodes_with_connections = episodes
        .into_iter()
        .map(|e| MediaWithFirstConnection {
            default_connection: default_connections.get(&e.id).cloned(),
            media: e,
        })
        .collect();

    Ok(episodes_with_connections)
}

pub fn get_api_router() -> Router<AppState> {
    RpcRouter::new()
        .for_state::<AppState>()
        .add(get_all_media)
        .add(get_media_by_id)
        .add(get_season_episodes)
        .add(get_seasons)
        .write_client("client/src/@generated/server.ts")
        .unwrap()
        .to_router()
}
