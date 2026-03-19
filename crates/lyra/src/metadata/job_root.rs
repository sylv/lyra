use crate::entities::metadata_source::MetadataSource;
use crate::entities::{jobs as jobs_entity, node_metadata, nodes, nodes::NodeKind};
use crate::jobs::{JobExecutionPolicy, JobHandler, JobTarget, NODE_ID_COLUMN, VERSION_KEY_COLUMN};
use crate::json_encoding;
use crate::metadata::METADATA_RETRY_BACKOFF_SECONDS;
use crate::metadata::store::{
    upsert_remote_node_metadata_from_movie, upsert_remote_node_metadata_from_series,
};
use anyhow::Context;
use chrono::Datelike;
use lyra_metadata::{
    MetadataProvider, MovieCandidate, MovieMetadata, MovieRootMatchRequest, RootMatchHint,
    SeriesCandidate, SeriesMetadata, SeriesRootMatchRequest,
};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, sea_query::SelectStatement,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) type RootCandidatesByProvider = HashMap<String, StoredRootMatchCandidate>;

pub struct NodeMetadataMatchRootJob {
    providers: Vec<Arc<dyn MetadataProvider>>,
}

impl NodeMetadataMatchRootJob {
    pub fn new(providers: Vec<Arc<dyn MetadataProvider>>) -> Self {
        Self { providers }
    }
}

enum MatchedRoot {
    Series {
        candidate: SeriesCandidate,
        metadata: SeriesMetadata,
    },
    Movie {
        candidate: MovieCandidate,
        metadata: MovieMetadata,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "candidate")]
pub(crate) enum StoredRootMatchCandidate {
    Series(SeriesCandidate),
    Movie(MovieCandidate),
}

#[async_trait::async_trait]
impl JobHandler for NodeMetadataMatchRootJob {
    fn job_kind(&self) -> jobs_entity::JobKind {
        jobs_entity::JobKind::NodeMatchMetadataRoot
    }

    fn execution_policy(&self) -> JobExecutionPolicy {
        JobExecutionPolicy::with_backoff_seconds(METADATA_RETRY_BACKOFF_SECONDS)
    }

    fn targets(&self) -> (JobTarget, SelectStatement) {
        let mut query = nodes::Entity::find()
            .select_only()
            .column_as(nodes::Column::Id, NODE_ID_COLUMN)
            .column_as(nodes::Column::LastAddedAt, VERSION_KEY_COLUMN)
            .filter(nodes::Column::ParentId.is_null())
            .filter(nodes::Column::Kind.is_in([NodeKind::Movie, NodeKind::Series]))
            .order_by_asc(nodes::Column::LastAddedAt)
            .order_by_asc(nodes::Column::Id);

        (JobTarget::Node, QuerySelect::query(&mut query).to_owned())
    }

    async fn execute(
        &self,
        pool: &DatabaseConnection,
        job: &jobs_entity::Model,
    ) -> anyhow::Result<()> {
        let node_id = job
            .node_id
            .as_deref()
            .with_context(|| format!("job {} missing node_id", job.id))?;

        let Some(node) = nodes::Entity::find_by_id(node_id.to_string())
            .one(pool)
            .await?
        else {
            return Ok(());
        };

        let mut candidates = decode_root_candidates(node.match_candidates_json.as_deref())?;
        let mut failures = Vec::new();
        for provider in &self.providers {
            match match_root(pool, provider.as_ref(), &node).await {
                Ok(Some(matched_root)) => {
                    if let Err(error) =
                        upsert_node_metadata_for_match(pool, provider.id(), &node.id, &matched_root)
                            .await
                    {
                        failures.push(format!(
                            "provider {} failed to upsert node metadata: {error:#}",
                            provider.id()
                        ));
                        continue;
                    }

                    candidates.insert(
                        provider.id().to_string(),
                        stored_root_candidate_for_match(&matched_root),
                    );
                }
                Ok(None) => {
                    candidates.remove(provider.id());
                    failures.push(format!(
                        "provider {} did not match node {}",
                        provider.id(),
                        node.id
                    ));
                }
                Err(error) => failures.push(format!(
                    "provider {} failed to match node {}: {error:#}",
                    provider.id(),
                    node.id
                )),
            }
        }

        let payload = encode_root_candidates(&candidates)?;
        if payload != node.match_candidates_json {
            nodes::Entity::update(nodes::ActiveModel {
                id: Set(node.id),
                match_candidates_json: Set(payload),
                updated_at: Set(chrono::Utc::now().timestamp()),
                ..Default::default()
            })
            .exec(pool)
            .await?;
        }

        if !failures.is_empty() {
            anyhow::bail!(
                "metadata root matching completed with failures: {}",
                failures.join("; ")
            );
        }

        Ok(())
    }
}

fn stored_root_candidate_for_match(matched_root: &MatchedRoot) -> StoredRootMatchCandidate {
    match matched_root {
        MatchedRoot::Series { candidate, .. } => {
            StoredRootMatchCandidate::Series(candidate.clone())
        }
        MatchedRoot::Movie { candidate, .. } => StoredRootMatchCandidate::Movie(candidate.clone()),
    }
}

pub(crate) fn decode_root_candidates(
    payload: Option<&[u8]>,
) -> anyhow::Result<RootCandidatesByProvider> {
    let Some(payload) = payload else {
        return Ok(HashMap::new());
    };

    json_encoding::decode_json_zstd::<RootCandidatesByProvider>(payload)
        .context("failed to decode root match candidates")
}

fn encode_root_candidates(
    candidates: &RootCandidatesByProvider,
) -> anyhow::Result<Option<Vec<u8>>> {
    if candidates.is_empty() {
        return Ok(None);
    }

    let payload = json_encoding::encode_json_zstd(candidates)
        .context("failed to encode root match candidates")?;
    Ok(Some(payload))
}

async fn upsert_node_metadata_for_match(
    pool: &DatabaseConnection,
    provider_id: &str,
    node_id: &str,
    matched_root: &MatchedRoot,
) -> anyhow::Result<()> {
    let now = chrono::Utc::now().timestamp();
    match matched_root {
        MatchedRoot::Series { metadata, .. } => {
            upsert_remote_node_metadata_from_series(pool, node_id, provider_id, metadata, now).await
        }
        MatchedRoot::Movie { metadata, .. } => {
            upsert_remote_node_metadata_from_movie(pool, node_id, provider_id, metadata, now).await
        }
    }
}

async fn match_root(
    pool: &DatabaseConnection,
    provider: &dyn MetadataProvider,
    node: &nodes::Model,
) -> anyhow::Result<Option<MatchedRoot>> {
    let hint = load_root_match_hint(pool, node).await?;

    match node.kind {
        NodeKind::Series => {
            let candidates = provider
                .match_series_root(SeriesRootMatchRequest { hint })
                .await?;
            let Some(candidate) = candidates.first() else {
                return Ok(None);
            };

            let metadata = provider.lookup_series_metadata(&candidate.value).await?;
            Ok(Some(MatchedRoot::Series {
                candidate: candidate.value.clone(),
                metadata,
            }))
        }
        NodeKind::Movie => {
            let candidates = provider
                .match_movie_root(MovieRootMatchRequest { hint })
                .await?;
            let Some(candidate) = candidates.first() else {
                return Ok(None);
            };

            let metadata = provider.lookup_movie_metadata(&candidate.value).await?;
            Ok(Some(MatchedRoot::Movie {
                candidate: candidate.value.clone(),
                metadata,
            }))
        }
        _ => Ok(None),
    }
}

async fn load_root_match_hint(
    pool: &DatabaseConnection,
    node: &nodes::Model,
) -> anyhow::Result<RootMatchHint> {
    let metadata_rows = node_metadata::Entity::find()
        .filter(node_metadata::Column::NodeId.eq(node.id.clone()))
        .order_by_desc(node_metadata::Column::Source)
        .all(pool)
        .await?;

    let years = if node.kind == NodeKind::Movie {
        metadata_rows
            .iter()
            .find_map(|row| row.released_at)
            .map(|timestamp| chrono::DateTime::from_timestamp(timestamp, 0).map(|dt| dt.year()))
            .flatten()
    } else {
        None
    };

    // prefer parser-derived ids from local metadata, but keep remote ids as a fallback.
    let local_metadata = metadata_rows
        .iter()
        .find(|row| row.source == MetadataSource::Local);

    Ok(RootMatchHint {
        title: local_metadata
            .map(|row| row.name.clone())
            .unwrap_or_else(|| node.name.clone()),
        start_year: years,
        end_year: None,
        imdb_id: local_metadata
            .and_then(|row| row.imdb_id.clone())
            .or_else(|| local_metadata.and_then(|row| row.imdb_id.clone())),
        tmdb_id: local_metadata
            .and_then(|row| row.tmdb_id)
            .or_else(|| local_metadata.and_then(|row| row.tmdb_id))
            .and_then(|value| u64::try_from(value).ok()),
    })
}
