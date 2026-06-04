use crate::entities::{
    images::{self, ImageKind, ImageSource},
    node_images, node_metadata,
    node_metadata::MetadataSource,
    nodes,
};
use crate::ids;
use crate::json_encoding;
use crate::metadata::NodeLocalMetadataInput;
use lyra_metadata::{
    ContentRating, EpisodeMetadata, ImageSet, MetadataStatus, MovieMetadata, SeasonMetadata,
    SeriesMetadata,
};
use sea_orm::sea_query::{Expr, OnConflict, Query};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, ExprTrait,
    QueryFilter,
};
use std::collections::{HashMap, HashSet};

pub async fn upsert_local_node_metadata_rows(
    pool: &impl ConnectionTrait,
    rows: &[NodeLocalMetadataInput],
    now: i64,
) -> anyhow::Result<()> {
    if rows.is_empty() {
        return Ok(());
    }

    node_metadata::Entity::insert_many(rows.iter().cloned().map(|row| {
        node_metadata::ActiveModel {
            id: Set(ids::generate_ulid()),
            node_id: Set(row.node_id),
            source: Set(MetadataSource::Local),
            imdb_id: Set(row.imdb_id),
            tmdb_id: Set(row.tmdb_id),
            name: Set(row.name),
            description: Set(None),
            score_display: Set(None),
            score_normalized: Set(None),
            first_aired: Set(None),
            last_aired: Set(None),
            status: Set(None),
            tagline: Set(None),
            content_ratings: Set(None),
            next_aired: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        }
    }))
    .on_conflict(
        OnConflict::columns([node_metadata::Column::NodeId, node_metadata::Column::Source])
            .update_columns([
                node_metadata::Column::ImdbId,
                node_metadata::Column::TmdbId,
                node_metadata::Column::Name,
                node_metadata::Column::Description,
                node_metadata::Column::ScoreDisplay,
                node_metadata::Column::ScoreNormalized,
                node_metadata::Column::FirstAired,
                node_metadata::Column::LastAired,
                node_metadata::Column::Status,
                node_metadata::Column::Tagline,
                node_metadata::Column::ContentRatings,
                node_metadata::Column::NextAired,
                node_metadata::Column::UpdatedAt,
            ])
            .to_owned(),
    )
    .exec(pool)
    .await?;

    Ok(())
}

pub async fn delete_local_node_metadata_for_root_except(
    pool: &impl ConnectionTrait,
    root_id: &str,
    keep_node_ids: &[String],
) -> anyhow::Result<()> {
    delete_metadata_for_root_except(pool, root_id, MetadataSource::Local, keep_node_ids).await
}

pub async fn upsert_remote_node_metadata_from_series(
    pool: &impl ConnectionTrait,
    node_id: &str,
    metadata: &SeriesMetadata,
    now: i64,
) -> anyhow::Result<()> {
    upsert_remote_node_metadata(pool, node_id, metadata_fields_from_series(metadata), now).await
}

pub async fn upsert_remote_node_metadata_from_movie(
    pool: &impl ConnectionTrait,
    node_id: &str,
    metadata: &MovieMetadata,
    now: i64,
) -> anyhow::Result<()> {
    upsert_remote_node_metadata(pool, node_id, metadata_fields_from_movie(metadata), now).await
}

pub async fn upsert_remote_episode_metadata_for_batch(
    pool: &impl ConnectionTrait,
    batch: &[nodes::Model],
    episodes: &[EpisodeMetadata],
    now: i64,
) -> anyhow::Result<Vec<String>> {
    let batch_node_ids = batch
        .iter()
        .map(|node| node.id.clone())
        .collect::<HashSet<_>>();
    let mut matched_node_ids = Vec::new();

    for episode in episodes {
        if !batch_node_ids.contains(&episode.item_id) {
            continue;
        }

        matched_node_ids.push(episode.item_id.clone());
        upsert_remote_node_metadata(
            pool,
            &episode.item_id,
            MetadataFields {
                imdb_id: None,
                tmdb_id: None,
                name: episode.name.clone(),
                description: episode.description.clone(),
                score_display: episode.score_display.clone(),
                score_normalized: episode.score_normalized,
                first_aired: episode.first_aired,
                last_aired: episode.last_aired,
                status: map_status(episode.status),
                tagline: episode.tagline.clone(),
                next_aired: episode.next_aired,
                content_ratings: episode.content_ratings.clone(),
                images: episode.images.clone(),
            },
            now,
        )
        .await?;
    }

    Ok(matched_node_ids)
}

pub async fn upsert_remote_season_metadata_for_batch(
    pool: &impl ConnectionTrait,
    batch: &[nodes::Model],
    seasons: &[SeasonMetadata],
    now: i64,
) -> anyhow::Result<Vec<String>> {
    let season_number_map = batch
        .iter()
        .filter_map(|node| {
            node.season_number
                .map(|season_number| (season_number, node.id.clone()))
        })
        .collect::<HashMap<_, _>>();
    let mut matched_node_ids = Vec::new();

    for season in seasons {
        let Some(season_id) = season_number_map.get(&(season.season_number as i64)) else {
            continue;
        };

        matched_node_ids.push(season_id.clone());
        upsert_remote_node_metadata(
            pool,
            season_id,
            MetadataFields {
                imdb_id: None,
                tmdb_id: None,
                name: season.name.clone(),
                description: season.description.clone(),
                score_display: season.score_display.clone(),
                score_normalized: season.score_normalized,
                first_aired: season.first_aired,
                last_aired: season.last_aired,
                status: map_status(season.status),
                tagline: season.tagline.clone(),
                next_aired: season.next_aired,
                content_ratings: season.content_ratings.clone(),
                images: season.images.clone(),
            },
            now,
        )
        .await?;
    }

    Ok(matched_node_ids)
}

pub async fn clear_remote_node_metadata_for_root(
    pool: &impl ConnectionTrait,
    root_id: &str,
) -> anyhow::Result<()> {
    delete_metadata_for_root_except(pool, root_id, MetadataSource::Remote, &[]).await
}

pub async fn clear_remote_node_metadata_for_root_except(
    pool: &impl ConnectionTrait,
    root_id: &str,
    keep_node_ids: &[String],
) -> anyhow::Result<()> {
    delete_metadata_for_root_except(pool, root_id, MetadataSource::Remote, keep_node_ids).await
}

async fn delete_metadata_for_root_except(
    pool: &impl ConnectionTrait,
    root_id: &str,
    source: MetadataSource,
    keep_node_ids: &[String],
) -> anyhow::Result<()> {
    let mut delete_query = node_metadata::Entity::delete_many()
        .filter(node_metadata::Column::Source.eq(source))
        .filter(
            node_metadata::Column::NodeId.in_subquery(
                Query::select()
                    .column(nodes::Column::Id)
                    .from(nodes::Entity)
                    .and_where(Expr::col((nodes::Entity, nodes::Column::RootId)).eq(root_id))
                    .to_owned(),
            ),
        );

    if !keep_node_ids.is_empty() {
        delete_query =
            delete_query.filter(node_metadata::Column::NodeId.is_not_in(keep_node_ids.to_vec()));
    }

    delete_query.exec(pool).await?;
    Ok(())
}

#[derive(Clone)]
struct MetadataFields {
    imdb_id: Option<String>,
    tmdb_id: Option<i64>,
    name: String,
    description: Option<String>,
    score_display: Option<String>,
    score_normalized: Option<i64>,
    first_aired: Option<i64>,
    last_aired: Option<i64>,
    status: Option<node_metadata::MetadataStatus>,
    tagline: Option<String>,
    next_aired: Option<i64>,
    content_ratings: Vec<ContentRating>,
    images: ImageSet,
}

fn metadata_fields_from_series(metadata: &SeriesMetadata) -> MetadataFields {
    MetadataFields {
        imdb_id: metadata.imdb_id.clone(),
        tmdb_id: metadata.tmdb_id.and_then(|value| i64::try_from(value).ok()),
        name: metadata.name.clone(),
        description: metadata.description.clone(),
        score_display: metadata.score_display.clone(),
        score_normalized: metadata.score_normalized,
        first_aired: metadata.first_aired,
        last_aired: metadata.last_aired,
        status: map_status(metadata.status),
        tagline: metadata.tagline.clone(),
        next_aired: metadata.next_aired,
        content_ratings: metadata.content_ratings.clone(),
        images: metadata.images.clone(),
    }
}

fn metadata_fields_from_movie(metadata: &MovieMetadata) -> MetadataFields {
    MetadataFields {
        imdb_id: metadata.imdb_id.clone(),
        tmdb_id: metadata.tmdb_id.and_then(|value| i64::try_from(value).ok()),
        name: metadata.name.clone(),
        description: metadata.description.clone(),
        score_display: metadata.score_display.clone(),
        score_normalized: metadata.score_normalized,
        first_aired: metadata.first_aired,
        last_aired: metadata.last_aired,
        status: map_status(metadata.status),
        tagline: metadata.tagline.clone(),
        next_aired: None,
        content_ratings: metadata.content_ratings.clone(),
        images: metadata.images.clone(),
    }
}

async fn upsert_remote_node_metadata(
    pool: &impl ConnectionTrait,
    node_id: &str,
    metadata: MetadataFields,
    now: i64,
) -> anyhow::Result<()> {
    node_metadata::Entity::insert(node_metadata::ActiveModel {
        id: Set(ids::generate_ulid()),
        node_id: Set(node_id.to_string()),
        source: Set(MetadataSource::Remote),
        imdb_id: Set(metadata.imdb_id.clone()),
        tmdb_id: Set(metadata.tmdb_id),
        name: Set(metadata.name.clone()),
        description: Set(metadata.description.clone()),
        score_display: Set(metadata.score_display.clone()),
        score_normalized: Set(metadata.score_normalized),
        first_aired: Set(metadata.first_aired),
        last_aired: Set(metadata.last_aired),
        status: Set(metadata.status),
        tagline: Set(metadata.tagline.clone()),
        content_ratings: Set(encode_content_ratings(&metadata.content_ratings)?),
        next_aired: Set(metadata.next_aired),
        created_at: Set(now),
        updated_at: Set(now),
    })
    .on_conflict(
        OnConflict::columns([node_metadata::Column::NodeId, node_metadata::Column::Source])
            .update_columns([
                node_metadata::Column::ImdbId,
                node_metadata::Column::TmdbId,
                node_metadata::Column::Name,
                node_metadata::Column::Description,
                node_metadata::Column::ScoreDisplay,
                node_metadata::Column::ScoreNormalized,
                node_metadata::Column::FirstAired,
                node_metadata::Column::LastAired,
                node_metadata::Column::Status,
                node_metadata::Column::Tagline,
                node_metadata::Column::ContentRatings,
                node_metadata::Column::NextAired,
                node_metadata::Column::UpdatedAt,
            ])
            .to_owned(),
    )
    .exec(pool)
    .await?;

    let row = node_metadata::Entity::find()
        .filter(node_metadata::Column::NodeId.eq(node_id.to_string()))
        .filter(node_metadata::Column::Source.eq(MetadataSource::Remote))
        .one(pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("remote metadata row missing after upsert"))?;

    replace_metadata_images(pool, &row.node_id, metadata.images, now).await?;

    Ok(())
}

async fn replace_metadata_images(
    pool: &impl ConnectionTrait,
    node_id: &str,
    images: ImageSet,
    now: i64,
) -> anyhow::Result<()> {
    node_images::Entity::delete_many()
        .filter(node_images::Column::NodeId.eq(node_id.to_string()))
        .exec(pool)
        .await?;

    let grouped = [
        (ImageKind::Poster, images.posters),
        (ImageKind::Thumbnail, images.thumbnails),
        (ImageKind::Backdrop, images.backdrops),
        (ImageKind::Logo, images.logos),
    ];

    let mut rows = Vec::new();
    for (kind, group) in grouped {
        for (position, image) in group.into_iter().enumerate() {
            let image_id = ensure_remote_image(pool, Some(image.url.as_str()), kind, now).await?;
            let Some(image_id) = image_id else {
                continue;
            };
            rows.push(node_images::ActiveModel {
                id: Set(ids::generate_ulid()),
                node_id: Set(node_id.to_string()),
                image_id: Set(image_id),
                source: Set(ImageSource::Remote),
                kind: Set(kind),
                language: Set(image.language),
                vote_average: Set(image.vote_average),
                vote_count: Set(image.vote_count),
                width: Set(image.width),
                height: Set(image.height),
                file_type: Set(image.file_type),
                is_active: Set(position == 0),
                created_at: Set(now),
            });
        }
    }

    if !rows.is_empty() {
        node_images::Entity::insert_many(rows).exec(pool).await?;
    }

    Ok(())
}

pub async fn ensure_remote_image(
    pool: &impl ConnectionTrait,
    source_url: Option<&str>,
    kind: ImageKind,
    now: i64,
) -> anyhow::Result<Option<String>> {
    let Some(source_url) = source_url else {
        return Ok(None);
    };
    if source_url.trim().is_empty() {
        return Ok(None);
    }

    let image_id = crate::assets::remote_image_id(source_url);
    if let Some(existing) = images::Entity::find_by_id(image_id.clone())
        .one(pool)
        .await?
    {
        let mut active: images::ActiveModel = existing.clone().into();
        active.updated_at = Set(Some(now));
        let updated = active.update(pool).await?;
        return Ok(Some(updated.id));
    }

    let image = images::ActiveModel {
        id: Set(image_id),
        kind: Set(kind),
        source: Set(ImageSource::Remote),
        source_url: Set(Some(source_url.to_string())),
        created_at: Set(now),
        updated_at: Set(Some(now)),
        ..Default::default()
    }
    .insert(pool)
    .await?;

    Ok(Some(image.id))
}

fn encode_content_ratings(rows: &[ContentRating]) -> anyhow::Result<Option<Vec<u8>>> {
    if rows.is_empty() {
        return Ok(None);
    }
    Ok(Some(json_encoding::encode_json_zstd(rows)?))
}

fn map_status(status: Option<MetadataStatus>) -> Option<node_metadata::MetadataStatus> {
    match status? {
        MetadataStatus::Upcoming => Some(node_metadata::MetadataStatus::Upcoming),
        MetadataStatus::Airing => Some(node_metadata::MetadataStatus::Airing),
        MetadataStatus::Returning => Some(node_metadata::MetadataStatus::Returning),
        MetadataStatus::Finished => Some(node_metadata::MetadataStatus::Finished),
        MetadataStatus::Cancelled => Some(node_metadata::MetadataStatus::Cancelled),
        MetadataStatus::InTheaters => Some(node_metadata::MetadataStatus::InTheaters),
        MetadataStatus::Released => Some(node_metadata::MetadataStatus::Released),
    }
}
