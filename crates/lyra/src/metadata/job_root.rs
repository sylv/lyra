use crate::entities::{
    files, item_files, items, jobs as jobs_entity,
    roots::{self, RootKind},
};
use crate::jobs::{JobExecutionPolicy, JobHandler, JobTarget, ROOT_ID_COLUMN, VERSION_KEY_COLUMN};
use crate::json_encoding;
use crate::metadata::METADATA_RETRY_BACKOFF_SECONDS;
use crate::metadata::store::{
    upsert_remote_root_metadata_from_movie, upsert_remote_root_metadata_from_series,
};
use anyhow::Context;
use lyra_metadata::{
    MetadataProvider, MovieCandidate, MovieMetadata, MovieRootMatchRequest, RootMatchHint,
    SeriesCandidate, SeriesMetadata, SeriesRootMatchRequest,
};
use lyra_parser::parse_files;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait, sea_query::SelectStatement,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

const MAX_HINT_FILES: u64 = 150;

pub(crate) type RootCandidatesByProvider = HashMap<String, StoredRootMatchCandidate>;

pub struct RootMetadataMatchRootJob {
    providers: Vec<Arc<dyn MetadataProvider>>,
}

impl RootMetadataMatchRootJob {
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
impl JobHandler for RootMetadataMatchRootJob {
    fn job_kind(&self) -> jobs_entity::JobKind {
        jobs_entity::JobKind::RootMatchMetadataRoot
    }

    fn execution_policy(&self) -> JobExecutionPolicy {
        JobExecutionPolicy::with_backoff_seconds(METADATA_RETRY_BACKOFF_SECONDS)
    }

    fn targets(&self) -> (JobTarget, SelectStatement) {
        let mut query = roots::Entity::find()
            .select_only()
            .column_as(roots::Column::Id, ROOT_ID_COLUMN)
            .column_as(roots::Column::LastAddedAt, VERSION_KEY_COLUMN)
            .order_by_asc(roots::Column::LastAddedAt)
            .order_by_asc(roots::Column::Id);

        (JobTarget::Root, QuerySelect::query(&mut query).to_owned())
    }

    async fn execute(
        &self,
        pool: &DatabaseConnection,
        job: &jobs_entity::Model,
    ) -> anyhow::Result<()> {
        let root_id = job
            .root_id
            .as_deref()
            .with_context(|| format!("job {} missing root_id", job.id))?;

        let Some(root) = roots::Entity::find_by_id(root_id.to_string())
            .one(pool)
            .await?
        else {
            return Ok(());
        };

        let mut candidates = decode_root_candidates(root.match_candidates_json.as_deref())?;
        let mut failures = Vec::new();
        for provider in &self.providers {
            match match_root(pool, provider.as_ref(), &root).await {
                Ok(Some(matched_root)) => {
                    if let Err(error) =
                        upsert_root_metadata_for_match(pool, provider.id(), &root.id, &matched_root)
                            .await
                    {
                        failures.push(format!(
                            "provider {} failed to upsert root metadata: {error:#}",
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
                        "provider {} did not match root {}",
                        provider.id(),
                        root.id
                    ));
                }
                Err(error) => failures.push(format!(
                    "provider {} failed to match root {}: {error:#}",
                    provider.id(),
                    root.id
                )),
            }
        }

        let payload = encode_root_candidates(&candidates)?;
        if payload != root.match_candidates_json {
            roots::Entity::update(roots::ActiveModel {
                id: Set(root.id),
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

async fn upsert_root_metadata_for_match(
    pool: &DatabaseConnection,
    provider_id: &str,
    root_id: &str,
    matched_root: &MatchedRoot,
) -> anyhow::Result<()> {
    let now = chrono::Utc::now().timestamp();
    match matched_root {
        MatchedRoot::Series { metadata, .. } => {
            upsert_remote_root_metadata_from_series(pool, root_id, provider_id, metadata, now).await
        }
        MatchedRoot::Movie { metadata, .. } => {
            upsert_remote_root_metadata_from_movie(pool, root_id, provider_id, metadata, now).await
        }
    }
}

async fn match_root(
    pool: &DatabaseConnection,
    provider: &dyn MetadataProvider,
    root: &roots::Model,
) -> anyhow::Result<Option<MatchedRoot>> {
    let hint = load_root_match_hint(pool, root).await?;

    match root.kind {
        RootKind::Series => {
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
        RootKind::Movie => {
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
    }
}

async fn load_root_match_hint(
    pool: &DatabaseConnection,
    root: &roots::Model,
) -> anyhow::Result<RootMatchHint> {
    let file_paths = item_files::Entity::find()
        .join(JoinType::InnerJoin, item_files::Relation::Items.def())
        .join(JoinType::InnerJoin, item_files::Relation::Files.def())
        .filter(items::Column::RootId.eq(root.id.clone()))
        .select_only()
        .column(files::Column::RelativePath)
        .distinct()
        .limit(MAX_HINT_FILES)
        .into_tuple::<String>()
        .all(pool)
        .await?;

    if file_paths.is_empty() {
        return Ok(RootMatchHint {
            title: root.name.clone(),
            start_year: None,
            end_year: None,
            imdb_id: None,
            tmdb_id: None,
        });
    }

    let parsed_files = parse_files(file_paths).await;
    let mut years = HashMap::<i32, usize>::new();
    for parsed in &parsed_files {
        if let Some(year) = parsed
            .start_year
            .and_then(|value| i32::try_from(value).ok())
        {
            years
                .entry(year)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    Ok(RootMatchHint {
        title: root.name.clone(),
        start_year: years
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(year, _)| year),
        end_year: parsed_files
            .iter()
            .filter_map(|parsed| parsed.end_year.and_then(|value| i32::try_from(value).ok()))
            .max(),
        imdb_id: parsed_files
            .iter()
            .find_map(|parsed| parsed.imdb_id.clone()),
        tmdb_id: parsed_files.iter().find_map(|parsed| parsed.tmdb_id),
    })
}
