use crate::entities::{
    assets::{self, AssetKind, AssetType},
    metadata_source::MetadataSource,
    node_metadata, node_metadata_content_ratings, node_metadata_genres, node_metadata_images,
    node_metadata_images::NodeMetadataImageKind,
    node_metadata_recommendations,
    node_metadata_recommendations::RecommendationMediaKind,
    nodes, people, root_node_cast,
};
use crate::ids;
use crate::metadata::local::{LOCAL_METADATA_PROVIDER_ID, NodeLocalMetadataInput};
use lyra_metadata::{
    CastCredit, ContentRating, EpisodeMetadata, ImageSet, MetadataGenre, MetadataStatus,
    MovieMetadata, PersonMetadata, Recommendation, RecommendedMediaKind, SeasonMetadata,
    SeriesMetadata,
};
use sea_orm::sea_query::{Expr, OnConflict, Query};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
    QueryOrder,
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
            provider_id: Set(LOCAL_METADATA_PROVIDER_ID.to_owned()),
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
            next_aired: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        }
    }))
    .on_conflict(
        OnConflict::columns([node_metadata::Column::NodeId, node_metadata::Column::Source])
            .update_columns([
                node_metadata::Column::ProviderId,
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
    provider_id: &str,
    metadata: &SeriesMetadata,
    now: i64,
) -> anyhow::Result<()> {
    upsert_remote_node_metadata(
        pool,
        node_id,
        provider_id,
        metadata_fields_from_series(metadata),
        now,
    )
    .await
}

pub async fn upsert_remote_node_metadata_from_movie(
    pool: &impl ConnectionTrait,
    node_id: &str,
    provider_id: &str,
    metadata: &MovieMetadata,
    now: i64,
) -> anyhow::Result<()> {
    upsert_remote_node_metadata(
        pool,
        node_id,
        provider_id,
        metadata_fields_from_movie(metadata),
        now,
    )
    .await
}

pub async fn upsert_remote_episode_metadata_for_batch(
    pool: &impl ConnectionTrait,
    provider_id: &str,
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
            provider_id,
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
                genres: episode.genres.clone(),
                content_ratings: episode.content_ratings.clone(),
                recommendations: episode.recommendations.clone(),
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
    provider_id: &str,
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
            provider_id,
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
                genres: season.genres.clone(),
                content_ratings: season.content_ratings.clone(),
                recommendations: season.recommendations.clone(),
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

pub async fn clear_root_cast(pool: &impl ConnectionTrait, root_id: &str) -> anyhow::Result<()> {
    root_node_cast::Entity::delete_many()
        .filter(root_node_cast::Column::RootNodeId.eq(root_id.to_string()))
        .exec(pool)
        .await?;
    Ok(())
}

pub async fn replace_root_cast(
    pool: &impl ConnectionTrait,
    root_id: &str,
    provider_id: &str,
    cast: &[CastCredit],
    people_metadata: &[PersonMetadata],
    now: i64,
) -> anyhow::Result<()> {
    root_node_cast::Entity::delete_many()
        .filter(root_node_cast::Column::RootNodeId.eq(root_id.to_string()))
        .exec(pool)
        .await?;

    if cast.is_empty() {
        return Ok(());
    }

    let people_by_provider_person_id = people_metadata
        .iter()
        .map(|person| (person.provider_person_id.as_str(), person))
        .collect::<HashMap<_, _>>();

    let mut person_id_by_provider_person_id = HashMap::new();
    for credit in cast {
        if person_id_by_provider_person_id.contains_key(credit.provider_person_id.as_str()) {
            continue;
        }

        let fallback = PersonMetadata {
            provider_person_id: credit.provider_person_id.clone(),
            name: credit.name.clone(),
            birthday: None,
            description: None,
            profile_image_url: None,
        };
        let person = people_by_provider_person_id
            .get(credit.provider_person_id.as_str())
            .copied()
            .unwrap_or(&fallback);
        let person_id = upsert_person(pool, provider_id, person, now).await?;
        person_id_by_provider_person_id.insert(credit.provider_person_id.clone(), person_id);
    }

    root_node_cast::Entity::insert_many(cast.iter().enumerate().map(|(position, credit)| {
        root_node_cast::ActiveModel {
            id: Set(ids::generate_ulid()),
            root_node_id: Set(root_id.to_string()),
            person_id: Set(
                person_id_by_provider_person_id[credit.provider_person_id.as_str()].clone(),
            ),
            character_name: Set(credit.character_name.clone()),
            department: Set(credit.department.clone()),
            position: Set(position as i64),
            created_at: Set(now),
        }
    }))
    .exec(pool)
    .await?;

    Ok(())
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
    genres: Vec<MetadataGenre>,
    content_ratings: Vec<ContentRating>,
    recommendations: Vec<Recommendation>,
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
        genres: metadata.genres.clone(),
        content_ratings: metadata.content_ratings.clone(),
        recommendations: metadata.recommendations.clone(),
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
        genres: metadata.genres.clone(),
        content_ratings: metadata.content_ratings.clone(),
        recommendations: metadata.recommendations.clone(),
        images: metadata.images.clone(),
    }
}

async fn upsert_remote_node_metadata(
    pool: &impl ConnectionTrait,
    node_id: &str,
    provider_id: &str,
    metadata: MetadataFields,
    now: i64,
) -> anyhow::Result<()> {
    node_metadata::Entity::insert(node_metadata::ActiveModel {
        id: Set(ids::generate_ulid()),
        node_id: Set(node_id.to_string()),
        source: Set(MetadataSource::Remote),
        provider_id: Set(provider_id.to_string()),
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
        next_aired: Set(metadata.next_aired),
        created_at: Set(now),
        updated_at: Set(now),
    })
    .on_conflict(
        OnConflict::columns([node_metadata::Column::NodeId, node_metadata::Column::Source])
            .update_columns([
                node_metadata::Column::ProviderId,
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

    replace_metadata_children(pool, &row.id, metadata, now).await?;

    Ok(())
}

async fn replace_metadata_children(
    pool: &impl ConnectionTrait,
    metadata_id: &str,
    metadata: MetadataFields,
    now: i64,
) -> anyhow::Result<()> {
    node_metadata_images::Entity::delete_many()
        .filter(node_metadata_images::Column::NodeMetadataId.eq(metadata_id.to_string()))
        .exec(pool)
        .await?;
    node_metadata_recommendations::Entity::delete_many()
        .filter(node_metadata_recommendations::Column::NodeMetadataId.eq(metadata_id.to_string()))
        .exec(pool)
        .await?;
    node_metadata_genres::Entity::delete_many()
        .filter(node_metadata_genres::Column::NodeMetadataId.eq(metadata_id.to_string()))
        .exec(pool)
        .await?;
    node_metadata_content_ratings::Entity::delete_many()
        .filter(node_metadata_content_ratings::Column::NodeMetadataId.eq(metadata_id.to_string()))
        .exec(pool)
        .await?;

    insert_metadata_images(pool, metadata_id, metadata.images, now).await?;

    if !metadata.recommendations.is_empty() {
        node_metadata_recommendations::Entity::insert_many(
            metadata
                .recommendations
                .into_iter()
                .enumerate()
                .map(
                    |(position, row)| node_metadata_recommendations::ActiveModel {
                        id: Set(ids::generate_ulid()),
                        node_metadata_id: Set(metadata_id.to_string()),
                        provider_id: Set("tmdb".to_string()),
                        media_kind: Set(match row.media_kind {
                            RecommendedMediaKind::Movie => RecommendationMediaKind::Movie,
                            RecommendedMediaKind::Series => RecommendationMediaKind::Series,
                        }),
                        tmdb_id: Set(row.tmdb_id.and_then(|value| i64::try_from(value).ok())),
                        imdb_id: Set(row.imdb_id),
                        name: Set(row.name),
                        first_aired: Set(row.first_aired),
                        position: Set(position as i64),
                        created_at: Set(now),
                    },
                ),
        )
        .exec(pool)
        .await?;
    }

    if !metadata.genres.is_empty() {
        node_metadata_genres::Entity::insert_many(metadata.genres.into_iter().enumerate().map(
            |(position, row)| node_metadata_genres::ActiveModel {
                id: Set(ids::generate_ulid()),
                node_metadata_id: Set(metadata_id.to_string()),
                provider_id: Set(row.provider_id),
                external_id: Set(row.external_id),
                name: Set(row.name),
                position: Set(position as i64),
                created_at: Set(now),
            },
        ))
        .exec(pool)
        .await?;
    }

    if !metadata.content_ratings.is_empty() {
        node_metadata_content_ratings::Entity::insert_many(
            metadata
                .content_ratings
                .into_iter()
                .enumerate()
                .map(
                    |(position, row)| node_metadata_content_ratings::ActiveModel {
                        id: Set(ids::generate_ulid()),
                        node_metadata_id: Set(metadata_id.to_string()),
                        country_code: Set(row.country_code),
                        rating: Set(row.rating),
                        release_date: Set(row.release_date),
                        release_type: Set(row.release_type),
                        position: Set(position as i64),
                        created_at: Set(now),
                    },
                ),
        )
        .exec(pool)
        .await?;
    }

    Ok(())
}

async fn insert_metadata_images(
    pool: &impl ConnectionTrait,
    metadata_id: &str,
    images: ImageSet,
    now: i64,
) -> anyhow::Result<()> {
    let grouped = [
        (NodeMetadataImageKind::Poster, images.posters),
        (NodeMetadataImageKind::Thumbnail, images.thumbnails),
        (NodeMetadataImageKind::Backdrop, images.backdrops),
        (NodeMetadataImageKind::Logo, images.logos),
    ];

    let mut rows = Vec::new();
    for (kind, group) in grouped {
        for (position, image) in group.into_iter().enumerate() {
            let asset_kind = match kind {
                NodeMetadataImageKind::Poster => AssetKind::Poster,
                NodeMetadataImageKind::Thumbnail => AssetKind::Thumbnail,
                NodeMetadataImageKind::Backdrop => AssetKind::Backdrop,
                NodeMetadataImageKind::Logo => AssetKind::Logo,
            };
            let asset_id =
                ensure_remote_asset(pool, Some(image.url.as_str()), asset_kind, now).await?;
            let Some(asset_id) = asset_id else {
                continue;
            };
            rows.push(node_metadata_images::ActiveModel {
                id: Set(ids::generate_ulid()),
                node_metadata_id: Set(metadata_id.to_string()),
                asset_id: Set(asset_id),
                kind: Set(kind),
                position: Set(position as i64),
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
        node_metadata_images::Entity::insert_many(rows)
            .exec(pool)
            .await?;
    }

    Ok(())
}

pub async fn ensure_remote_asset(
    pool: &impl ConnectionTrait,
    source_url: Option<&str>,
    kind: AssetKind,
    now: i64,
) -> anyhow::Result<Option<String>> {
    let Some(source_url) = source_url else {
        return Ok(None);
    };
    if source_url.trim().is_empty() {
        return Ok(None);
    }

    if let Some(existing) = assets::Entity::find()
        .filter(assets::Column::SourceUrl.eq(source_url.to_string()))
        .filter(assets::Column::Kind.eq(kind))
        .order_by_desc(assets::Column::Id)
        .one(pool)
        .await?
    {
        let mut active: assets::ActiveModel = existing.clone().into();
        active.updated_at = Set(Some(now));
        let updated = active.update(pool).await?;
        return Ok(Some(updated.id));
    }

    let asset_id = crate::ids::generate_prefixed_hashid("a", [source_url]);
    let asset = assets::ActiveModel {
        id: Set(asset_id),
        kind: Set(kind),
        asset_type: Set(AssetType::Image),
        source_url: Set(Some(source_url.to_string())),
        created_at: Set(now),
        updated_at: Set(Some(now)),
        ..Default::default()
    }
    .insert(pool)
    .await?;

    Ok(Some(asset.id))
}

async fn upsert_person(
    pool: &impl ConnectionTrait,
    provider_id: &str,
    person: &PersonMetadata,
    now: i64,
) -> anyhow::Result<String> {
    let person_id = ids::generate_hashid([provider_id, person.provider_person_id.as_str()]);
    let profile_asset_id = ensure_remote_asset(
        pool,
        person.profile_image_url.as_deref(),
        AssetKind::Profile,
        now,
    )
    .await?;

    people::Entity::insert(people::ActiveModel {
        id: Set(person_id.clone()),
        provider_id: Set(provider_id.to_string()),
        provider_person_id: Set(person.provider_person_id.clone()),
        name: Set(person.name.clone()),
        birthday: Set(person.birthday.clone()),
        description: Set(person.description.clone()),
        profile_asset_id: Set(profile_asset_id.clone()),
        created_at: Set(now),
        updated_at: Set(now),
    })
    .on_conflict(
        OnConflict::columns([people::Column::ProviderId, people::Column::ProviderPersonId])
            .update_columns([
                people::Column::Name,
                people::Column::Birthday,
                people::Column::Description,
                people::Column::ProfileAssetId,
                people::Column::UpdatedAt,
            ])
            .to_owned(),
    )
    .exec(pool)
    .await?;

    Ok(person_id)
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
