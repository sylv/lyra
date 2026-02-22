use crate::entities::{item_metadata, root_metadata, season_metadata};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait};

pub async fn insert_local_root_metadata<C: ConnectionTrait>(
    pool: &C,
    root_id: &str,
    name: &str,
    now: i64,
) -> Result<(), sea_orm::DbErr> {
    root_metadata::ActiveModel {
        root_id: Set(root_id.to_string()),
        source: Set("local".to_string()),
        source_key: Set(None),
        is_primary: Set(true),
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

pub async fn insert_local_season_metadata<C: ConnectionTrait>(
    pool: &C,
    season_id: &str,
    name: &str,
    now: i64,
) -> Result<(), sea_orm::DbErr> {
    season_metadata::ActiveModel {
        season_id: Set(season_id.to_string()),
        source: Set("local".to_string()),
        source_key: Set(None),
        is_primary: Set(true),
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

pub async fn insert_local_item_metadata<C: ConnectionTrait>(
    pool: &C,
    item_id: &str,
    name: &str,
    now: i64,
) -> Result<(), sea_orm::DbErr> {
    item_metadata::ActiveModel {
        item_id: Set(item_id.to_string()),
        source: Set("local".to_string()),
        source_key: Set(None),
        is_primary: Set(true),
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
