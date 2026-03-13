use crate::entities::{
    assets::{self},
    item_metadata, items,
    metadata_source::MetadataSource,
    root_metadata, season_metadata, seasons,
};
use lyra_metadata::{EpisodeMetadata, ImageSet, MovieMetadata, SeasonMetadata, SeriesMetadata};
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use std::collections::HashMap;

pub async fn upsert_remote_root_metadata_from_series(
    pool: &DatabaseConnection,
    root_id: &str,
    provider_id: &str,
    metadata: &SeriesMetadata,
    now: i64,
) -> anyhow::Result<()> {
    upsert_remote_root_metadata(
        pool,
        root_id,
        provider_id,
        metadata_fields_from_series(metadata),
        now,
    )
    .await
}

pub async fn upsert_remote_root_metadata_from_movie(
    pool: &DatabaseConnection,
    root_id: &str,
    provider_id: &str,
    metadata: &MovieMetadata,
    now: i64,
) -> anyhow::Result<()> {
    upsert_remote_root_metadata(
        pool,
        root_id,
        provider_id,
        metadata_fields_from_movie(metadata),
        now,
    )
    .await
}

pub async fn overwrite_remote_item_metadata_for_batch(
    pool: &DatabaseConnection,
    provider_id: &str,
    batch: &[items::Model],
    episodes: &[EpisodeMetadata],
    now: i64,
) -> anyhow::Result<()> {
    let item_ids = batch.iter().map(|item| item.id.clone()).collect::<Vec<_>>();
    clear_remote_item_metadata_for_batch(pool, &item_ids).await?;

    for episode in episodes {
        let Some(item) = batch.iter().find(|item| item.id == episode.item_id) else {
            continue;
        };

        upsert_remote_item_metadata(
            pool,
            &item.root_id,
            &item.id,
            provider_id,
            MetadataFields {
                name: episode.name.clone(),
                description: episode.description.clone(),
                score_display: episode.score_display.clone(),
                score_normalized: episode.score_normalized,
                released_at: episode.released_at,
                ended_at: None,
                images: episode.images.clone(),
            },
            now,
        )
        .await?;
    }

    Ok(())
}

pub async fn overwrite_remote_movie_metadata_for_batch(
    pool: &DatabaseConnection,
    provider_id: &str,
    batch: &[items::Model],
    metadata: &MovieMetadata,
    now: i64,
) -> anyhow::Result<()> {
    let item_ids = batch.iter().map(|item| item.id.clone()).collect::<Vec<_>>();
    clear_remote_item_metadata_for_batch(pool, &item_ids).await?;

    for item in batch {
        upsert_remote_item_metadata(
            pool,
            &item.root_id,
            &item.id,
            provider_id,
            metadata_fields_from_movie(metadata),
            now,
        )
        .await?;
    }

    Ok(())
}

pub async fn overwrite_remote_season_metadata_for_batch(
    pool: &DatabaseConnection,
    root_id: &str,
    provider_id: &str,
    batch: &[items::Model],
    seasons_result: &[SeasonMetadata],
    now: i64,
) -> anyhow::Result<()> {
    let batch_season_ids = batch
        .iter()
        .filter_map(|item| item.season_id.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if batch_season_ids.is_empty() {
        return Ok(());
    }

    season_metadata::Entity::delete_many()
        .filter(season_metadata::Column::SeasonId.is_in(batch_season_ids.clone()))
        .filter(season_metadata::Column::Source.eq(MetadataSource::Remote))
        .exec(pool)
        .await?;

    let season_number_map = seasons::Entity::find()
        .filter(seasons::Column::Id.is_in(batch_season_ids))
        .all(pool)
        .await?
        .into_iter()
        .map(|season| (season.season_number, season.id))
        .collect::<HashMap<_, _>>();

    for season in seasons_result {
        let Some(season_id) = season_number_map.get(&(season.season_number as i64)) else {
            continue;
        };
        upsert_remote_season_metadata(
            pool,
            root_id,
            season_id,
            provider_id,
            MetadataFields {
                name: season.name.clone(),
                description: season.description.clone(),
                score_display: season.score_display.clone(),
                score_normalized: season.score_normalized,
                released_at: season.released_at,
                ended_at: season.ended_at,
                images: season.images.clone(),
            },
            now,
        )
        .await?;
    }
    Ok(())
}

pub async fn clear_remote_item_metadata_for_batch(
    pool: &DatabaseConnection,
    item_ids: &[String],
) -> anyhow::Result<()> {
    if item_ids.is_empty() {
        return Ok(());
    }
    item_metadata::Entity::delete_many()
        .filter(item_metadata::Column::ItemId.is_in(item_ids.to_vec()))
        .filter(item_metadata::Column::Source.eq(MetadataSource::Remote))
        .exec(pool)
        .await?;

    Ok(())
}

#[derive(Clone)]
struct MetadataFields {
    name: String,
    description: Option<String>,
    score_display: Option<String>,
    score_normalized: Option<i64>,
    released_at: Option<i64>,
    ended_at: Option<i64>,
    images: ImageSet,
}

fn metadata_fields_from_series(metadata: &SeriesMetadata) -> MetadataFields {
    MetadataFields {
        name: metadata.name.clone(),
        description: metadata.description.clone(),
        score_display: metadata.score_display.clone(),
        score_normalized: metadata.score_normalized,
        released_at: metadata.released_at,
        ended_at: metadata.ended_at,
        images: metadata.images.clone(),
    }
}

fn metadata_fields_from_movie(metadata: &MovieMetadata) -> MetadataFields {
    MetadataFields {
        name: metadata.name.clone(),
        description: metadata.description.clone(),
        score_display: metadata.score_display.clone(),
        score_normalized: metadata.score_normalized,
        released_at: metadata.released_at,
        ended_at: metadata.ended_at,
        images: metadata.images.clone(),
    }
}

async fn upsert_remote_root_metadata(
    pool: &DatabaseConnection,
    root_id: &str,
    provider_id: &str,
    metadata: MetadataFields,
    now: i64,
) -> anyhow::Result<()> {
    let poster_asset_id =
        ensure_remote_asset(pool, metadata.images.poster_url.as_deref(), now).await?;
    let thumbnail_asset_id =
        ensure_remote_asset(pool, metadata.images.thumbnail_url.as_deref(), now).await?;
    let background_asset_id =
        ensure_remote_asset(pool, metadata.images.background_url.as_deref(), now).await?;

    root_metadata::Entity::insert(root_metadata::ActiveModel {
        root_id: Set(root_id.to_string()),
        source: Set(MetadataSource::Remote),
        provider_id: Set(provider_id.to_string()),
        name: Set(metadata.name),
        description: Set(metadata.description),
        score_display: Set(metadata.score_display),
        score_normalized: Set(metadata.score_normalized),
        released_at: Set(metadata.released_at),
        ended_at: Set(metadata.ended_at),
        poster_asset_id: Set(poster_asset_id),
        thumbnail_asset_id: Set(thumbnail_asset_id),
        background_asset_id: Set(background_asset_id),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    })
    .on_conflict(
        OnConflict::columns([root_metadata::Column::RootId, root_metadata::Column::Source])
            .update_columns([
                root_metadata::Column::ProviderId,
                root_metadata::Column::Name,
                root_metadata::Column::Description,
                root_metadata::Column::ScoreDisplay,
                root_metadata::Column::ScoreNormalized,
                root_metadata::Column::ReleasedAt,
                root_metadata::Column::EndedAt,
                root_metadata::Column::PosterAssetId,
                root_metadata::Column::ThumbnailAssetId,
                root_metadata::Column::BackgroundAssetId,
                root_metadata::Column::UpdatedAt,
            ])
            .to_owned(),
    )
    .exec(pool)
    .await?;
    Ok(())
}

async fn upsert_remote_season_metadata(
    pool: &DatabaseConnection,
    root_id: &str,
    season_id: &str,
    provider_id: &str,
    metadata: MetadataFields,
    now: i64,
) -> anyhow::Result<()> {
    let poster_asset_id =
        ensure_remote_asset(pool, metadata.images.poster_url.as_deref(), now).await?;
    let thumbnail_asset_id =
        ensure_remote_asset(pool, metadata.images.thumbnail_url.as_deref(), now).await?;
    let background_asset_id =
        ensure_remote_asset(pool, metadata.images.background_url.as_deref(), now).await?;

    season_metadata::Entity::insert(season_metadata::ActiveModel {
        root_id: Set(root_id.to_string()),
        season_id: Set(season_id.to_string()),
        source: Set(MetadataSource::Remote),
        provider_id: Set(provider_id.to_string()),
        name: Set(metadata.name),
        description: Set(metadata.description),
        score_display: Set(metadata.score_display),
        score_normalized: Set(metadata.score_normalized),
        released_at: Set(metadata.released_at),
        ended_at: Set(metadata.ended_at),
        poster_asset_id: Set(poster_asset_id),
        thumbnail_asset_id: Set(thumbnail_asset_id),
        background_asset_id: Set(background_asset_id),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    })
    .on_conflict(
        OnConflict::columns([
            season_metadata::Column::RootId,
            season_metadata::Column::SeasonId,
            season_metadata::Column::Source,
        ])
        .update_columns([
            season_metadata::Column::ProviderId,
            season_metadata::Column::Name,
            season_metadata::Column::Description,
            season_metadata::Column::ScoreDisplay,
            season_metadata::Column::ScoreNormalized,
            season_metadata::Column::ReleasedAt,
            season_metadata::Column::EndedAt,
            season_metadata::Column::PosterAssetId,
            season_metadata::Column::ThumbnailAssetId,
            season_metadata::Column::BackgroundAssetId,
            season_metadata::Column::UpdatedAt,
        ])
        .to_owned(),
    )
    .exec(pool)
    .await?;
    Ok(())
}

async fn upsert_remote_item_metadata(
    pool: &DatabaseConnection,
    root_id: &str,
    item_id: &str,
    provider_id: &str,
    metadata: MetadataFields,
    now: i64,
) -> anyhow::Result<()> {
    let poster_asset_id =
        ensure_remote_asset(pool, metadata.images.poster_url.as_deref(), now).await?;
    let thumbnail_asset_id =
        ensure_remote_asset(pool, metadata.images.thumbnail_url.as_deref(), now).await?;
    let background_asset_id =
        ensure_remote_asset(pool, metadata.images.background_url.as_deref(), now).await?;

    item_metadata::Entity::insert(item_metadata::ActiveModel {
        root_id: Set(root_id.to_string()),
        item_id: Set(item_id.to_string()),
        source: Set(MetadataSource::Remote),
        provider_id: Set(provider_id.to_string()),
        name: Set(metadata.name),
        description: Set(metadata.description),
        score_display: Set(metadata.score_display),
        score_normalized: Set(metadata.score_normalized),
        released_at: Set(metadata.released_at),
        ended_at: Set(metadata.ended_at),
        poster_asset_id: Set(poster_asset_id),
        thumbnail_asset_id: Set(thumbnail_asset_id),
        background_asset_id: Set(background_asset_id),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    })
    .on_conflict(
        OnConflict::columns([
            item_metadata::Column::RootId,
            item_metadata::Column::ItemId,
            item_metadata::Column::Source,
        ])
        .update_columns([
            item_metadata::Column::ProviderId,
            item_metadata::Column::Name,
            item_metadata::Column::Description,
            item_metadata::Column::ScoreDisplay,
            item_metadata::Column::ScoreNormalized,
            item_metadata::Column::ReleasedAt,
            item_metadata::Column::EndedAt,
            item_metadata::Column::PosterAssetId,
            item_metadata::Column::ThumbnailAssetId,
            item_metadata::Column::BackgroundAssetId,
            item_metadata::Column::UpdatedAt,
        ])
        .to_owned(),
    )
    .exec(pool)
    .await?;
    Ok(())
}

async fn ensure_remote_asset(
    pool: &DatabaseConnection,
    source_url: Option<&str>,
    now: i64,
) -> anyhow::Result<Option<i64>> {
    let Some(source_url) = source_url else {
        return Ok(None);
    };
    if source_url.trim().is_empty() {
        return Ok(None);
    }

    if let Some(existing) = assets::Entity::find()
        .filter(assets::Column::SourceUrl.eq(source_url.to_string()))
        .order_by_desc(assets::Column::Id)
        .one(pool)
        .await?
    {
        return Ok(Some(existing.id));
    }

    let inserted = assets::ActiveModel {
        source_url: Set(Some(source_url.to_string())),
        created_at: Set(now),
        ..Default::default()
    }
    .insert(pool)
    .await?;
    Ok(Some(inserted.id))
}
