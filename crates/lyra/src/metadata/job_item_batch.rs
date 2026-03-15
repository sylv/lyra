use crate::entities::{jobs as jobs_entity, nodes, nodes::NodeKind};
use crate::jobs::{JobExecutionPolicy, JobHandler, JobTarget, NODE_ID_COLUMN, VERSION_KEY_COLUMN};
use crate::metadata::METADATA_RETRY_BACKOFF_SECONDS;
use crate::metadata::job_root::{StoredRootMatchCandidate, decode_root_candidates};
use crate::metadata::store::{
    clear_remote_node_metadata_for_batch, overwrite_remote_episode_metadata_for_batch,
    overwrite_remote_movie_metadata_for_batch, overwrite_remote_season_metadata_for_batch,
};
use anyhow::Context;
use lyra_metadata::{MetadataProvider, SeriesCandidate, SeriesItem, SeriesItemsRequest};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    sea_query::{Expr, SelectStatement},
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct NodeMetadataMatchGroupsJob {
    providers: Vec<Arc<dyn MetadataProvider>>,
}

impl NodeMetadataMatchGroupsJob {
    pub fn new(providers: Vec<Arc<dyn MetadataProvider>>) -> Self {
        Self { providers }
    }
}

#[async_trait::async_trait]
impl JobHandler for NodeMetadataMatchGroupsJob {
    fn job_kind(&self) -> jobs_entity::JobKind {
        jobs_entity::JobKind::NodeMatchMetadataGroups
    }

    fn execution_policy(&self) -> JobExecutionPolicy {
        JobExecutionPolicy::with_backoff_seconds(METADATA_RETRY_BACKOFF_SECONDS)
    }

    fn targets(&self) -> (JobTarget, SelectStatement) {
        let mut query = nodes::Entity::find()
            .select_only()
            .column_as(nodes::Column::Id, NODE_ID_COLUMN)
            .column_as(
                Expr::col(nodes::Column::LastAddedAt).add(Expr::col(nodes::Column::UpdatedAt)),
                VERSION_KEY_COLUMN,
            )
            .filter(nodes::Column::MatchCandidatesJson.is_not_null())
            .filter(
                nodes::Column::Kind
                    .eq(NodeKind::Season)
                    .or(nodes::Column::Kind
                        .eq(NodeKind::Series)
                        .and(nodes::Column::ParentId.is_null())),
            )
            .order_by_asc(nodes::Column::RootId)
            .order_by_asc(nodes::Column::Order)
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
            .with_context(|| format!("job {} missing node_id", job.id))?
            .to_string();

        let Some(group_node) = nodes::Entity::find_by_id(node_id.clone()).one(pool).await? else {
            return Ok(());
        };
        let Some(root_node) = nodes::Entity::find_by_id(group_node.root_id.clone())
            .one(pool)
            .await?
        else {
            return Ok(());
        };

        let group_items = load_group_items(pool, &group_node).await?;
        if group_items.is_empty() {
            return Ok(());
        }

        let candidates = decode_root_candidates(root_node.match_candidates_json.as_deref())?;
        if candidates.is_empty() {
            return Ok(());
        }

        let mut failures = Vec::new();
        for provider in &self.providers {
            let Some(candidate) = candidates.get(provider.id()) else {
                continue;
            };

            match (root_node.kind, candidate) {
                (NodeKind::Movie, StoredRootMatchCandidate::Movie(candidate)) => {
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
                                    "provider {} failed to write movie batch for node {}: {error:#}",
                                    provider.id(),
                                    root_node.id
                                ));
                            }
                        }
                        Err(error) => failures.push(format!(
                            "provider {} failed to lookup movie metadata for node {}: {error:#}",
                            provider.id(),
                            root_node.id
                        )),
                    }
                }
                (NodeKind::Series, StoredRootMatchCandidate::Series(candidate)) => {
                    match match_series_group(
                        pool,
                        provider.as_ref(),
                        provider.id(),
                        &group_node,
                        candidate,
                        &group_items,
                    )
                    .await
                    {
                        Ok(summary) => {
                            if !summary.unmatched_node_ids.is_empty() {
                                failures.push(format!(
                                    "provider {} left {} unmatched playable nodes under {}",
                                    provider.id(),
                                    summary.unmatched_node_ids.len(),
                                    group_node.id
                                ));
                            }
                        }
                        Err(error) => failures.push(format!(
                            "provider {} failed to match node group {}: {error:#}",
                            provider.id(),
                            group_node.id,
                        )),
                    }
                }
                _ => {
                    let node_ids = group_items
                        .iter()
                        .map(|node| node.id.clone())
                        .collect::<Vec<_>>();
                    if let Err(error) = clear_remote_node_metadata_for_batch(pool, &node_ids).await
                    {
                        failures.push(format!(
                            "provider {} failed to clear mismatched node metadata for {}: {error:#}",
                            provider.id(),
                            group_node.id
                        ));
                    }
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
    group_node: &nodes::Model,
) -> anyhow::Result<Vec<nodes::Model>> {
    let mut query = nodes::Entity::find()
        .filter(nodes::Column::RootId.eq(group_node.root_id.clone()))
        .filter(
            nodes::Column::Kind
                .eq(NodeKind::Episode)
                .or(nodes::Column::Kind.eq(NodeKind::Movie)),
        )
        .order_by_asc(nodes::Column::Order)
        .order_by_asc(nodes::Column::Id);

    query = match group_node.kind {
        NodeKind::Season => query.filter(nodes::Column::ParentId.eq(group_node.id.clone())),
        // series metadata matching needs every playable node under the root, not just direct
        // children, because episodes usually sit under season nodes.
        NodeKind::Series => query,
        NodeKind::Movie => query.filter(nodes::Column::Id.eq(group_node.id.clone())),
        _ => query.filter(nodes::Column::Id.eq("__never__")),
    };

    Ok(query.all(pool).await?)
}

async fn match_series_group(
    pool: &DatabaseConnection,
    provider: &dyn MetadataProvider,
    provider_id: &str,
    group_node: &nodes::Model,
    candidate: &SeriesCandidate,
    group_items: &[nodes::Model],
) -> anyhow::Result<GroupMatchSummary> {
    let season_numbers = load_season_numbers(pool, group_items).await?;
    let req = SeriesItemsRequest {
        root_id: group_node.root_id.clone(),
        candidate: candidate.clone(),
        items: group_items
            .iter()
            .map(|node| SeriesItem {
                item_id: node.id.clone(),
                season_number: season_numbers
                    .get(&node.id)
                    .and_then(|value| i32::try_from(*value).ok()),
                episode_number: node
                    .episode_number
                    .and_then(|value| i32::try_from(value).ok()),
                name: node.name.clone(),
            })
            .collect::<Vec<_>>(),
    };

    let results = provider.lookup_series_items(req).await?;
    let now = chrono::Utc::now().timestamp();

    overwrite_remote_season_metadata_for_batch(
        pool,
        provider_id,
        group_items,
        &results.seasons,
        now,
    )
    .await?;
    overwrite_remote_episode_metadata_for_batch(
        pool,
        provider_id,
        group_items,
        &results.episodes,
        now,
    )
    .await?;

    let matched_node_ids = results
        .episodes
        .iter()
        .map(|episode| episode.item_id.as_str())
        .collect::<HashSet<_>>();
    let unmatched_node_ids = group_items
        .iter()
        .filter(|node| !matched_node_ids.contains(node.id.as_str()))
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();

    Ok(GroupMatchSummary { unmatched_node_ids })
}

async fn load_season_numbers(
    pool: &DatabaseConnection,
    batch: &[nodes::Model],
) -> anyhow::Result<HashMap<String, i64>> {
    let parent_ids = batch
        .iter()
        .filter_map(|node| node.parent_id.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    if parent_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let parents = nodes::Entity::find()
        .filter(nodes::Column::Id.is_in(parent_ids))
        .all(pool)
        .await?
        .into_iter()
        .map(|node| (node.id, node.season_number))
        .collect::<HashMap<_, _>>();

    Ok(batch
        .iter()
        .filter_map(|node| {
            let season_number = node
                .parent_id
                .as_ref()
                .and_then(|parent_id| parents.get(parent_id))
                .and_then(|value| *value);
            season_number.map(|season_number| (node.id.clone(), season_number))
        })
        .collect())
}

struct GroupMatchSummary {
    unmatched_node_ids: Vec<String>,
}
