use crate::entities::metadata_source::MetadataSource;
use crate::entities::{jobs as jobs_entity, node_metadata, nodes, nodes::NodeKind};
use crate::jobs::{Job, JobExecutionPolicy, JobLease, JobOutcome};
use crate::json_encoding;
use crate::metadata::METADATA_RETRY_BACKOFF_SECONDS;
use crate::metadata::store::{
    upsert_remote_node_metadata_from_movie, upsert_remote_node_metadata_from_series,
};
use anyhow::Context;
use chrono::Datelike;
use lyra_metadata::{
    MetadataProvider, MovieCandidate, MovieMetadata, MovieRootMatchRequest, RootMatchHint, Scored,
    SeriesCandidate, SeriesMetadata, SeriesRootMatchRequest,
};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Select,
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
impl Job for NodeMetadataMatchRootJob {
    type Entity = nodes::Entity;
    type Model = nodes::Model;

    const JOB_KIND: jobs_entity::JobKind = jobs_entity::JobKind::NodeMatchMetadataRoot;

    fn execution_policy(&self) -> JobExecutionPolicy {
        JobExecutionPolicy::with_backoff_seconds(METADATA_RETRY_BACKOFF_SECONDS)
    }

    fn query(&self) -> Select<Self::Entity> {
        nodes::Entity::find()
            .filter(nodes::Column::ParentId.is_null())
            .filter(nodes::Column::Kind.is_in([NodeKind::Movie, NodeKind::Series]))
            .filter(nodes::Column::MatchCandidatesJson.is_null())
            .order_by_asc(nodes::Column::LastAddedAt)
            .order_by_asc(nodes::Column::Id)
    }

    fn target_id(&self, target: &Self::Model) -> String {
        target.id.clone()
    }

    async fn run(
        &self,
        db: &DatabaseConnection,
        node: Self::Model,
        _ctx: &JobLease,
    ) -> anyhow::Result<JobOutcome> {
        let mut candidates = decode_root_candidates(node.match_candidates_json.as_deref())?;
        let mut failures = Vec::new();
        for provider in &self.providers {
            match match_root(db, provider.as_ref(), &node).await {
                Ok(Some(matched_root)) => {
                    if let Err(error) =
                        upsert_node_metadata_for_match(db, provider.id(), &node.id, &matched_root)
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
            .exec(db)
            .await?;
        }

        if !failures.is_empty() {
            anyhow::bail!(
                "metadata root matching completed with failures: {}",
                failures.join("; ")
            );
        }

        Ok(JobOutcome::Complete)
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
    db: &DatabaseConnection,
    provider_id: &str,
    node_id: &str,
    matched_root: &MatchedRoot,
) -> anyhow::Result<()> {
    let now = chrono::Utc::now().timestamp();
    match matched_root {
        MatchedRoot::Series { metadata, .. } => {
            upsert_remote_node_metadata_from_series(db, node_id, provider_id, metadata, now).await
        }
        MatchedRoot::Movie { metadata, .. } => {
            upsert_remote_node_metadata_from_movie(db, node_id, provider_id, metadata, now).await
        }
    }
}

async fn match_root(
    db: &DatabaseConnection,
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
                Ok(Some(MatchedRoot::Movie {
                    candidate,
                    metadata,
                }))
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
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
