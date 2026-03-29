use crate::entities::{
    assets::{self},
    metadata_source::MetadataSource,
    node_metadata, nodes,
    nodes::NodeKind,
};
use crate::ids;
use lyra_metadata::{EpisodeMetadata, ImageSet, MovieMetadata, SeasonMetadata, SeriesMetadata};
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder, TransactionTrait,
};
use std::collections::HashSet;

pub async fn upsert_remote_node_metadata_from_series(
    pool: &DatabaseConnection,
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
    pool: &DatabaseConnection,
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

pub async fn overwrite_remote_episode_metadata_for_batch(
    pool: &DatabaseConnection,
    provider_id: &str,
    batch: &[nodes::Model],
    episodes: &[EpisodeMetadata],
    now: i64,
) -> anyhow::Result<()> {
    let batch_node_ids = batch
        .iter()
        .map(|node| node.id.clone())
        .collect::<HashSet<_>>();
    let matched_node_ids = episodes
        .iter()
        .map(|episode| episode.item_id.clone())
        .collect::<HashSet<_>>();
    let stale_node_ids = batch
        .iter()
        .filter(|node| !matched_node_ids.contains(&node.id))
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    let txn = pool.begin().await?;

    // upsert before deleting stale rows so matched metadata keeps a stable backing rowid.
    for episode in episodes {
        if !batch_node_ids.contains(&episode.item_id) {
            continue;
        }

        upsert_remote_node_metadata(
            &txn,
            &episode.item_id,
            provider_id,
            MetadataFields {
                imdb_id: None,
                tmdb_id: None,
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

    clear_remote_node_metadata_for_batch(&txn, &stale_node_ids).await?;
    txn.commit().await?;

    Ok(())
}

pub async fn overwrite_remote_movie_metadata_for_batch(
    pool: &DatabaseConnection,
    provider_id: &str,
    batch: &[nodes::Model],
    metadata: &MovieMetadata,
    now: i64,
) -> anyhow::Result<()> {
    let txn = pool.begin().await?;

    for node in batch {
        upsert_remote_node_metadata(
            &txn,
            &node.id,
            provider_id,
            metadata_fields_from_movie(metadata),
            now,
        )
        .await?;
    }

    txn.commit().await?;

    Ok(())
}

pub async fn overwrite_remote_season_metadata_for_batch(
    pool: &DatabaseConnection,
    provider_id: &str,
    batch: &[nodes::Model],
    seasons_result: &[SeasonMetadata],
    now: i64,
) -> anyhow::Result<()> {
    let season_ids = batch
        .iter()
        .map(|node| node.id.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if season_ids.is_empty() {
        return Ok(());
    }

    let season_number_map = nodes::Entity::find()
        .filter(nodes::Column::Id.is_in(season_ids.clone()))
        .filter(nodes::Column::Kind.eq(NodeKind::Season))
        .all(pool)
        .await?
        .into_iter()
        .filter_map(|node| {
            node.season_number
                .map(|season_number| (season_number, node.id))
        })
        .collect::<std::collections::HashMap<_, _>>();
    let mut matched_season_ids = HashSet::new();
    let txn = pool.begin().await?;

    // keep matched season rows alive and only clear seasons the provider no longer returned.
    for season in seasons_result {
        let Some(season_id) = season_number_map.get(&(season.season_number as i64)) else {
            continue;
        };
        matched_season_ids.insert(season_id.clone());

        upsert_remote_node_metadata(
            &txn,
            season_id,
            provider_id,
            MetadataFields {
                imdb_id: None,
                tmdb_id: None,
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

    let stale_season_ids = season_ids
        .into_iter()
        .filter(|season_id| !matched_season_ids.contains(season_id))
        .collect::<Vec<_>>();
    clear_remote_node_metadata_for_batch(&txn, &stale_season_ids).await?;
    txn.commit().await?;

    Ok(())
}

pub async fn clear_remote_node_metadata_for_batch<C: ConnectionTrait>(
    pool: &C,
    node_ids: &[String],
) -> anyhow::Result<()> {
    if node_ids.is_empty() {
        return Ok(());
    }

    node_metadata::Entity::delete_many()
        .filter(node_metadata::Column::NodeId.is_in(node_ids.to_vec()))
        .filter(node_metadata::Column::Source.eq(MetadataSource::Remote))
        .exec(pool)
        .await?;

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
    released_at: Option<i64>,
    ended_at: Option<i64>,
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
        released_at: metadata.released_at,
        ended_at: metadata.ended_at,
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
        released_at: metadata.released_at,
        ended_at: metadata.ended_at,
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
    let poster_asset_id =
        ensure_remote_asset(pool, metadata.images.poster_url.as_deref(), now).await?;
    let thumbnail_asset_id =
        ensure_remote_asset(pool, metadata.images.thumbnail_url.as_deref(), now).await?;
    let background_asset_id =
        ensure_remote_asset(pool, metadata.images.background_url.as_deref(), now).await?;

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
        OnConflict::columns([node_metadata::Column::NodeId, node_metadata::Column::Source])
            .update_columns([
                node_metadata::Column::ProviderId,
                node_metadata::Column::ImdbId,
                node_metadata::Column::TmdbId,
                node_metadata::Column::Name,
                node_metadata::Column::Description,
                node_metadata::Column::ScoreDisplay,
                node_metadata::Column::ScoreNormalized,
                node_metadata::Column::ReleasedAt,
                node_metadata::Column::EndedAt,
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
        .order_by_desc(assets::Column::Id)
        .one(pool)
        .await?
    {
        return Ok(Some(existing.id));
    }

    let asset = assets::ActiveModel {
        id: Set(ids::generate_ulid()),
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
    use sea_orm::{ActiveValue::Set, Database};

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
            last_scanned_at: Set(None),
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
        kind: NodeKind,
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
            match_candidates_json: Set(None),
            last_added_at: Set(0),
            created_at: Set(0),
            updated_at: Set(0),
        })
        .exec(pool)
        .await?;

        Ok(nodes::Entity::find_by_id(id.to_owned()).one(pool).await?.unwrap())
    }

    #[tokio::test]
    async fn overwrite_remote_season_metadata_keeps_root_remote_metadata() -> anyhow::Result<()> {
        let pool = setup_test_db().await?;
        insert_library(&pool).await?;
        insert_node(&pool, "root", "root", None, NodeKind::Series, "Show", None, 0).await?;
        let season = insert_node(
            &pool,
            "season-1",
            "root",
            Some("root"),
            NodeKind::Season,
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
                released_at: None,
                ended_at: None,
                images: ImageSet::default(),
            },
            1,
        )
        .await?;

        overwrite_remote_season_metadata_for_batch(
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
                released_at: None,
                ended_at: None,
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
        assert_eq!(season_remote.description.as_deref(), Some("season description"));

        Ok(())
    }
}
