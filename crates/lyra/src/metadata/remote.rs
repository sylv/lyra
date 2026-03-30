use crate::entities::metadata_source::MetadataSource;
use crate::entities::{node_metadata, nodes, nodes::NodeKind};
use anyhow::Context;
use chrono::Datelike;
use lyra_metadata::{
    MetadataProvider, MovieMetadata, MovieRootMatchRequest, RootMatchHint, Scored, SeriesCandidate,
    SeriesItemsRequest, SeriesItemsResult, SeriesMetadata, SeriesRootMatchRequest,
};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

pub enum MatchedRoot {
    Series {
        candidate: SeriesCandidate,
        metadata: SeriesMetadata,
    },
    Movie {
        metadata: MovieMetadata,
    },
}

pub async fn match_root(
    db: &impl ConnectionTrait,
    provider: &dyn MetadataProvider,
    node: &nodes::Model,
) -> anyhow::Result<Option<MatchedRoot>> {
    let hint = load_root_match_hint(db, node).await?;

    match node.kind {
        NodeKind::Series => {
            let candidates = provider
                .match_series_root(SeriesRootMatchRequest { hint })
                .await?;

            if let Some(Scored {
                value: candidate, ..
            }) = candidates.into_iter().next()
            {
                let metadata = provider.lookup_series_metadata(&candidate).await?;
                Ok(Some(MatchedRoot::Series {
                    candidate,
                    metadata,
                }))
            } else {
                Ok(None)
            }
        }
        NodeKind::Movie => {
            let candidates = provider
                .match_movie_root(MovieRootMatchRequest { hint })
                .await?;

            if let Some(Scored {
                value: candidate, ..
            }) = candidates.into_iter().next()
            {
                let metadata = provider.lookup_movie_metadata(&candidate).await?;
                Ok(Some(MatchedRoot::Movie { metadata }))
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

pub async fn lookup_series_items(
    provider: &dyn MetadataProvider,
    root_id: &str,
    candidate: &SeriesCandidate,
    episode_nodes: &[nodes::Model],
) -> anyhow::Result<SeriesItemsResult> {
    let items = episode_nodes
        .iter()
        .map(|node| lyra_metadata::SeriesItem {
            item_id: node.id.clone(),
            season_number: node
                .season_number
                .and_then(|value| i32::try_from(value).ok()),
            episode_number: node
                .episode_number
                .and_then(|value| i32::try_from(value).ok()),
            name: node.name.clone(),
        })
        .collect::<Vec<_>>();

    provider
        .lookup_series_items(SeriesItemsRequest {
            root_id: root_id.to_owned(),
            candidate: candidate.clone(),
            items,
        })
        .await
}

async fn load_root_match_hint(
    db: &impl ConnectionTrait,
    node: &nodes::Model,
) -> anyhow::Result<RootMatchHint> {
    let local_metadata = node_metadata::Entity::find()
        .filter(node_metadata::Column::NodeId.eq(node.id.clone()))
        .filter(node_metadata::Column::Source.eq(MetadataSource::Local))
        .one(db)
        .await?
        .with_context(|| format!("missing local metadata for node {}", node.id))?;

    let year = local_metadata
        .released_at
        .and_then(|timestamp| chrono::DateTime::from_timestamp(timestamp, 0))
        .map(|timestamp| timestamp.year());

    Ok(RootMatchHint {
        title: local_metadata.name,
        start_year: year,
        end_year: year,
        imdb_id: local_metadata.imdb_id,
        tmdb_id: local_metadata
            .tmdb_id
            .and_then(|value| u64::try_from(value).ok()),
    })
}
