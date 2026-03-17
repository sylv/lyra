use crate::entities::{metadata_source::MetadataSource, node_metadata};
use crate::ids;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait};

pub async fn insert_local_node_metadata<C: ConnectionTrait>(
    pool: &C,
    node_id: &str,
    name: &str,
    imdb_id: Option<String>,
    tmdb_id: Option<i64>,
    now: i64,
) -> Result<(), sea_orm::DbErr> {
    node_metadata::ActiveModel {
        id: Set(ids::generate_ulid()),
        node_id: Set(node_id.to_string()),
        source: Set(MetadataSource::Local),
        provider_id: Set("local".to_string()),
        imdb_id: Set(imdb_id),
        tmdb_id: Set(tmdb_id),
        name: Set(name.to_string()),
        description: Set(None),
        score_display: Set(None),
        score_normalized: Set(None),
        released_at: Set(None),
        ended_at: Set(None),
        poster_asset_id: Set(None),
        thumbnail_asset_id: Set(None),
        background_asset_id: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(pool)
    .await?;

    Ok(())
}
