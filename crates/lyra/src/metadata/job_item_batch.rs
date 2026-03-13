use crate::entities::{
    items, jobs as jobs_entity,
    roots::{self, RootKind},
    seasons,
};
use crate::jobs::{
    JobExecutionPolicy, JobHandler, JobTarget, ROOT_ID_COLUMN, SEASON_ID_COLUMN, VERSION_KEY_COLUMN,
};
use crate::metadata::METADATA_RETRY_BACKOFF_SECONDS;
use crate::metadata::job_root::{StoredRootMatchCandidate, decode_root_candidates};
use crate::metadata::store::{
    clear_remote_item_metadata_for_batch, overwrite_remote_item_metadata_for_batch,
    overwrite_remote_movie_metadata_for_batch, overwrite_remote_season_metadata_for_batch,
};
use anyhow::Context;
use lyra_metadata::{MetadataProvider, SeriesCandidate, SeriesItem, SeriesItemsRequest};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait,
    sea_query::{Expr, SelectStatement},
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct RootMetadataMatchGroupsJob {
    providers: Vec<Arc<dyn MetadataProvider>>,
}

impl RootMetadataMatchGroupsJob {
    pub fn new(providers: Vec<Arc<dyn MetadataProvider>>) -> Self {
        Self { providers }
    }
}

#[async_trait::async_trait]
impl JobHandler for RootMetadataMatchGroupsJob {
    fn job_kind(&self) -> jobs_entity::JobKind {
        jobs_entity::JobKind::RootMatchMetadataGroups
    }

    fn execution_policy(&self) -> JobExecutionPolicy {
        JobExecutionPolicy::with_backoff_seconds(METADATA_RETRY_BACKOFF_SECONDS)
    }

    fn targets(&self) -> (JobTarget, SelectStatement) {
        let mut query = items::Entity::find()
            .join(JoinType::InnerJoin, items::Relation::Roots.def())
            .select_only()
            .column_as(items::Column::RootId, ROOT_ID_COLUMN)
            .column_as(items::Column::SeasonId, SEASON_ID_COLUMN)
            .column_as(
                Expr::expr(Expr::col((items::Entity, items::Column::LastAddedAt)).max())
                    .add(Expr::col((roots::Entity, roots::Column::UpdatedAt))),
                VERSION_KEY_COLUMN,
            )
            .filter(roots::Column::MatchCandidatesJson.is_not_null())
            .group_by(items::Column::RootId)
            .group_by(items::Column::SeasonId)
            .order_by_asc(items::Column::RootId)
            .order_by_asc(items::Column::SeasonId);

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
            .with_context(|| format!("job {} missing root_id", job.id))?
            .to_string();
        let season_id = job.season_id.clone();

        let Some(root) = roots::Entity::find_by_id(root_id.clone()).one(pool).await? else {
            return Ok(());
        };

        let group_items = load_group_items(pool, &root_id, season_id.as_deref()).await?;
        if group_items.is_empty() {
            return Ok(());
        }

        let candidates = decode_root_candidates(root.match_candidates_json.as_deref())?;
        if candidates.is_empty() {
            return Ok(());
        }

        let mut failures = Vec::new();
        for provider in &self.providers {
            let Some(candidate) = candidates.get(provider.id()) else {
                continue;
            };

            match (root.kind, candidate) {
                (RootKind::Movie, StoredRootMatchCandidate::Movie(candidate)) => {
                    match provider.lookup_movie_metadata(candidate).await {
                        Ok(metadata) => {
                            if let Err(error) = overwrite_remote_movie_metadata_for_batch(
                                pool,
                                provider.id(),
                                &group_items,
                                &metadata,
                                chrono::Utc::now().timestamp(),
                            )
                            .await
                            {
                                failures.push(format!(
                                    "provider {} failed to write movie batch for root {}: {error:#}",
                                    provider.id(),
                                    root.id
                                ));
                            }
                        }
                        Err(error) => failures.push(format!(
                            "provider {} failed to lookup movie metadata for root {}: {error:#}",
                            provider.id(),
                            root.id
                        )),
                    }
                }
                (RootKind::Series, StoredRootMatchCandidate::Series(candidate)) => {
                    match match_series_group(
                        pool,
                        provider.as_ref(),
                        provider.id(),
                        &root.id,
                        candidate,
                        &group_items,
                    )
                    .await
                    {
                        Ok(summary) => {
                            if !summary.unmatched_item_ids.is_empty() {
                                failures.push(format!(
                                    "provider {} left {} unmatched items in root {}{}",
                                    provider.id(),
                                    summary.unmatched_item_ids.len(),
                                    root.id,
                                    summary
                                        .season_id
                                        .as_deref()
                                        .map(|season_id| format!(", season {}", season_id))
                                        .unwrap_or_default()
                                ));
                            }
                        }
                        Err(error) => failures.push(format!(
                            "provider {} failed to match series group for root {}{}: {error:#}",
                            provider.id(),
                            root.id,
                            season_id
                                .as_deref()
                                .map(|season_id| format!(", season {}", season_id))
                                .unwrap_or_default()
                        )),
                    }
                }
                (root_kind, match_candidate) => {
                    tracing::warn!(
                        root_id = %root.id,
                        provider_id = provider.id(),
                        season_id,
                        root_kind = ?root_kind,
                        match_candidate = ?match_candidate,
                        "root kind and stored root candidate mismatch"
                    );

                    let item_ids = group_items
                        .iter()
                        .map(|item| item.id.clone())
                        .collect::<Vec<_>>();
                    if let Err(error) = clear_remote_item_metadata_for_batch(pool, &item_ids).await
                    {
                        failures.push(format!(
                            "provider {} failed to clear mismatched item metadata for root {}: {error:#}",
                            provider.id(),
                            root.id
                        ));
                    }
                    if let Err(error) = overwrite_remote_season_metadata_for_batch(
                        pool,
                        &root.id,
                        provider.id(),
                        &group_items,
                        &[],
                        chrono::Utc::now().timestamp(),
                    )
                    .await
                    {
                        failures.push(format!(
                            "provider {} failed to clear mismatched season metadata for root {}: {error:#}",
                            provider.id(),
                            root.id
                        ));
                    }
                    failures.push(format!(
                        "provider {} has root/candidate kind mismatch for root {}",
                        provider.id(),
                        root.id
                    ));
                }
            }
        }

        if !failures.is_empty() {
            anyhow::bail!(
                "metadata group matching completed with failures: {}",
                failures.join("; ")
            );
        }

        Ok(())
    }
}

async fn load_group_items(
    pool: &DatabaseConnection,
    root_id: &str,
    season_id: Option<&str>,
) -> anyhow::Result<Vec<items::Model>> {
    let mut query = items::Entity::find()
        .filter(items::Column::RootId.eq(root_id.to_string()))
        .order_by_asc(items::Column::Order)
        .order_by_asc(items::Column::Id);

    query = if let Some(season_id) = season_id {
        query.filter(items::Column::SeasonId.eq(season_id.to_string()))
    } else {
        query.filter(items::Column::SeasonId.is_null())
    };

    Ok(query.all(pool).await?)
}

async fn match_series_group(
    pool: &DatabaseConnection,
    provider: &dyn MetadataProvider,
    provider_id: &str,
    root_id: &str,
    candidate: &SeriesCandidate,
    group_items: &[items::Model],
) -> anyhow::Result<GroupMatchSummary> {
    let season_id = group_items.first().and_then(|item| item.season_id.clone());
    let season_numbers = load_season_numbers(pool, group_items).await?;
    let req = SeriesItemsRequest {
        root_id: root_id.to_string(),
        candidate: candidate.clone(),
        items: group_items
            .iter()
            .map(|item| SeriesItem {
                item_id: item.id.clone(),
                season_number: item
                    .season_id
                    .as_ref()
                    .and_then(|season_id| season_numbers.get(season_id).copied())
                    .and_then(|value| i32::try_from(value).ok()),
                episode_number: item
                    .episode_number
                    .and_then(|value| i32::try_from(value).ok()),
                name: item.name.clone(),
            })
            .collect::<Vec<_>>(),
    };

    let results = provider.lookup_series_items(req).await?;
    let now = chrono::Utc::now().timestamp();

    overwrite_remote_season_metadata_for_batch(
        pool,
        root_id,
        provider_id,
        group_items,
        &results.seasons,
        now,
    )
    .await?;
    overwrite_remote_item_metadata_for_batch(
        pool,
        provider_id,
        group_items,
        &results.episodes,
        now,
    )
    .await?;

    tracing::debug!(
        provider_id,
        root_id,
        item_count = group_items.len(),
        season_count = results.seasons.len(),
        episode_count = results.episodes.len(),
        "matched metadata for series item group"
    );

    let matched_item_ids = results
        .episodes
        .iter()
        .map(|episode| episode.item_id.as_str())
        .collect::<HashSet<_>>();
    let unmatched_item_ids = group_items
        .iter()
        .filter(|item| !matched_item_ids.contains(item.id.as_str()))
        .map(|item| item.id.clone())
        .collect::<Vec<_>>();

    Ok(GroupMatchSummary {
        season_id,
        unmatched_item_ids,
    })
}

async fn load_season_numbers(
    pool: &DatabaseConnection,
    batch: &[items::Model],
) -> anyhow::Result<HashMap<String, i64>> {
    let season_ids = batch
        .iter()
        .filter_map(|item| item.season_id.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    if season_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = seasons::Entity::find()
        .filter(seasons::Column::Id.is_in(season_ids))
        .all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|season| (season.id, season.season_number))
        .collect::<HashMap<_, _>>())
}

struct GroupMatchSummary {
    season_id: Option<String>,
    unmatched_item_ids: Vec<String>,
}
