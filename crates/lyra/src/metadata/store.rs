use crate::entities::{
    assets::{self, AssetKind, AssetType},
    metadata_source::MetadataSource,
    node_metadata, nodes,
};
use crate::ids;
use crate::metadata::local::{LOCAL_METADATA_PROVIDER_ID, NodeLocalMetadataInput};
use lyra_metadata::{EpisodeMetadata, ImageSet, MovieMetadata, SeasonMetadata, SeriesMetadata};
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
            poster_asset_id: Set(None),
            thumbnail_asset_id: Set(None),
            background_asset_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
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
                node_metadata::Column::PosterAssetId,
                node_metadata::Column::ThumbnailAssetId,
                node_metadata::Column::BackgroundAssetId,
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
    let poster_asset_id = ensure_remote_asset(
        pool,
        metadata.images.poster_url.as_deref(),
        AssetKind::Poster,
        now,
    )
    .await?;
    let thumbnail_asset_id = ensure_remote_asset(
        pool,
        metadata.images.thumbnail_url.as_deref(),
        AssetKind::Thumbnail,
        now,
    )
    .await?;
    let background_asset_id = ensure_remote_asset(
        pool,
        metadata.images.background_url.as_deref(),
        AssetKind::Background,
        now,
    )
    .await?;

    node_metadata::Entity::insert(node_metadata::ActiveModel {
        id: Set(ids::generate_ulid()),
        node_id: Set(node_id.to_string()),
        source: Set(MetadataSource::Remote),
        provider_id: Set(provider_id.to_string()),
        imdb_id: Set(metadata.imdb_id),
        tmdb_id: Set(metadata.tmdb_id),
        name: Set(metadata.name),
        description: Set(metadata.description),
        score_display: Set(metadata.score_display),
        score_normalized: Set(metadata.score_normalized),
        first_aired: Set(metadata.first_aired),
        last_aired: Set(metadata.last_aired),
        poster_asset_id: Set(poster_asset_id),
        thumbnail_asset_id: Set(thumbnail_asset_id),
        background_asset_id: Set(background_asset_id),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
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
                node_metadata::Column::PosterAssetId,
                node_metadata::Column::ThumbnailAssetId,
                node_metadata::Column::BackgroundAssetId,
                node_metadata::Column::UpdatedAt,
            ])
            .to_owned(),
    )
    .exec(pool)
    .await?;

    Ok(())
}

async fn ensure_remote_asset(
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

    // The same upstream URL can legitimately back multiple attachment roles.
    if let Some(existing) = assets::Entity::find()
        .filter(assets::Column::SourceUrl.eq(source_url.to_string()))
        .filter(assets::Column::Kind.eq(kind))
        .order_by_desc(assets::Column::Id)
        .one(pool)
        .await?
    {
        return Ok(Some(existing.id));
    }

    let asset = assets::ActiveModel {
        id: Set(ids::generate_ulid()),
        kind: Set(kind),
        asset_type: Set(AssetType::Image),
        source_url: Set(Some(source_url.to_string())),
        created_at: Set(now),
        ..Default::default()
    }
    .insert(pool)
    .await?;

    Ok(Some(asset.id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{libraries, metadata_source::MetadataSource};
    use sea_orm::{ActiveValue::Set, Database, DatabaseConnection};

    async fn setup_test_db() -> anyhow::Result<DatabaseConnection> {
        let pool = Database::connect("sqlite::memory:").await?;
        sqlx::migrate!("../../migrations")
            .run(pool.get_sqlite_connection_pool())
            .await?;

        Ok(pool)
    }

    async fn insert_library(pool: &DatabaseConnection) -> anyhow::Result<()> {
        libraries::Entity::insert(libraries::ActiveModel {
            id: Set("lib".to_owned()),
            path: Set("/library".to_owned()),
            name: Set("Library".to_owned()),
            pinned: Set(false),
            last_scanned_at: Set(None),
            unavailable_at: Set(None),
            created_at: Set(0),
        })
        .exec(pool)
        .await?;
        Ok(())
    }

    async fn insert_node(
        pool: &DatabaseConnection,
        id: &str,
        root_id: &str,
        parent_id: Option<&str>,
        kind: nodes::NodeKind,
        name: &str,
        season_number: Option<i64>,
        order: i64,
    ) -> anyhow::Result<nodes::Model> {
        nodes::Entity::insert(nodes::ActiveModel {
            id: Set(id.to_owned()),
            library_id: Set("lib".to_owned()),
            root_id: Set(root_id.to_owned()),
            parent_id: Set(parent_id.map(str::to_owned)),
            kind: Set(kind),
            name: Set(name.to_owned()),
            order: Set(order),
            season_number: Set(season_number),
            episode_number: Set(None),
            last_added_at: Set(0),
            last_fingerprint_version: Set(None),
            unavailable_at: Set(None),
            created_at: Set(0),
            updated_at: Set(0),
        })
        .exec(pool)
        .await?;

        Ok(nodes::Entity::find_by_id(id.to_owned())
            .one(pool)
            .await?
            .unwrap())
    }

    #[tokio::test]
    async fn upsert_remote_season_metadata_keeps_root_remote_metadata() -> anyhow::Result<()> {
        let pool = setup_test_db().await?;
        insert_library(&pool).await?;
        insert_node(
            &pool,
            "root",
            "root",
            None,
            nodes::NodeKind::Series,
            "Show",
            None,
            0,
        )
        .await?;
        let season = insert_node(
            &pool,
            "season-1",
            "root",
            Some("root"),
            nodes::NodeKind::Season,
            "Season 1",
            Some(1),
            1,
        )
        .await?;

        upsert_remote_node_metadata_from_series(
            &pool,
            "root",
            "tmdb",
            &SeriesMetadata {
                imdb_id: None,
                tmdb_id: Some(1),
                name: "Show".to_owned(),
                description: Some("root description".to_owned()),
                score_display: None,
                score_normalized: None,
                first_aired: None,
                last_aired: None,
                images: ImageSet::default(),
            },
            1,
        )
        .await?;

        upsert_remote_season_metadata_for_batch(
            &pool,
            "tmdb",
            &[season],
            &[SeasonMetadata {
                root_id: "root".to_owned(),
                season_number: 1,
                name: "Season 1".to_owned(),
                description: Some("season description".to_owned()),
                score_display: None,
                score_normalized: None,
                first_aired: None,
                last_aired: None,
                images: ImageSet::default(),
            }],
            2,
        )
        .await?;

        let root_remote = node_metadata::Entity::find()
            .filter(node_metadata::Column::NodeId.eq("root"))
            .filter(node_metadata::Column::Source.eq(MetadataSource::Remote))
            .one(&pool)
            .await?
            .unwrap();
        let season_remote = node_metadata::Entity::find()
            .filter(node_metadata::Column::NodeId.eq("season-1"))
            .filter(node_metadata::Column::Source.eq(MetadataSource::Remote))
            .one(&pool)
            .await?
            .unwrap();

        assert_eq!(root_remote.description.as_deref(), Some("root description"));
        assert_eq!(
            season_remote.description.as_deref(),
            Some("season description")
        );

        Ok(())
    }

    #[tokio::test]
    async fn upsert_remote_series_metadata_keeps_distinct_asset_kinds_per_relation()
    -> anyhow::Result<()> {
        let pool = setup_test_db().await?;
        insert_library(&pool).await?;
        insert_node(
            &pool,
            "root",
            "root",
            None,
            nodes::NodeKind::Series,
            "Show",
            None,
            0,
        )
        .await?;

        upsert_remote_node_metadata_from_series(
            &pool,
            "root",
            "tmdb",
            &SeriesMetadata {
                imdb_id: None,
                tmdb_id: Some(1),
                name: "Show".to_owned(),
                description: None,
                score_display: None,
                score_normalized: None,
                first_aired: None,
                last_aired: None,
                images: ImageSet {
                    poster_url: Some("https://example.com/shared.jpg".to_owned()),
                    thumbnail_url: Some("https://example.com/shared.jpg".to_owned()),
                    background_url: Some("https://example.com/shared.jpg".to_owned()),
                },
            },
            1,
        )
        .await?;

        let remote = node_metadata::Entity::find()
            .filter(node_metadata::Column::NodeId.eq("root"))
            .filter(node_metadata::Column::Source.eq(MetadataSource::Remote))
            .one(&pool)
            .await?
            .unwrap();

        let poster_asset = assets::Entity::find_by_id(remote.poster_asset_id.unwrap())
            .one(&pool)
            .await?
            .unwrap();
        let thumbnail_asset = assets::Entity::find_by_id(remote.thumbnail_asset_id.unwrap())
            .one(&pool)
            .await?
            .unwrap();
        let background_asset = assets::Entity::find_by_id(remote.background_asset_id.unwrap())
            .one(&pool)
            .await?
            .unwrap();

        assert_ne!(poster_asset.id, thumbnail_asset.id);
        assert_ne!(poster_asset.id, background_asset.id);
        assert_ne!(thumbnail_asset.id, background_asset.id);
        assert_eq!(poster_asset.kind, AssetKind::Poster);
        assert_eq!(thumbnail_asset.kind, AssetKind::Thumbnail);
        assert_eq!(background_asset.kind, AssetKind::Background);
        assert_eq!(poster_asset.asset_type, AssetType::Image);
        assert_eq!(thumbnail_asset.asset_type, AssetType::Image);
        assert_eq!(background_asset.asset_type, AssetType::Image);

        Ok(())
    }
}
