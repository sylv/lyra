use crate::entities::{
    jobs as jobs_entity, metadata_source::MetadataSource, node_metadata, nodes, nodes::NodeKind,
};
use crate::jobs::{Job, JobExecutionPolicy, JobLease, JobOutcome};
use crate::metadata::METADATA_RETRY_BACKOFF_SECONDS;
use crate::metadata::job_root::{StoredRootMatchCandidate, decode_root_candidates};
use crate::metadata::store::{
    clear_remote_node_metadata_for_batch, overwrite_remote_episode_metadata_for_batch,
    overwrite_remote_movie_metadata_for_batch, overwrite_remote_season_metadata_for_batch,
};
use lyra_metadata::{MetadataProvider, SeriesCandidate, SeriesItem, SeriesItemsRequest};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Select,
    sea_query::{Alias, Expr, Query},
};
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
impl Job for NodeMetadataMatchGroupsJob {
    type Entity = nodes::Entity;
    type Model = nodes::Model;

    const JOB_KIND: jobs_entity::JobKind = jobs_entity::JobKind::NodeMatchMetadataGroups;

    fn execution_policy(&self) -> JobExecutionPolicy {
        JobExecutionPolicy::with_backoff_seconds(METADATA_RETRY_BACKOFF_SECONDS)
    }

    fn query(&self) -> Select<Self::Entity> {
        let playable_nodes = Alias::new("playable_nodes");

        nodes::Entity::find()
            .filter(nodes::Column::ParentId.is_null())
            .filter(nodes::Column::Kind.is_in([NodeKind::Movie, NodeKind::Series]))
            .filter(nodes::Column::MatchCandidatesJson.is_not_null())
            .filter(Expr::exists(
                Query::select()
                    .expr(Expr::val(1))
                    .from_as(nodes::Entity, playable_nodes.clone())
                    .and_where(
                        Expr::col((playable_nodes.clone(), nodes::Column::RootId))
                            .equals((nodes::Entity, nodes::Column::Id)),
                    )
                    .and_where(
                        Expr::col((playable_nodes.clone(), nodes::Column::Kind))
                            .is_in([NodeKind::Episode, NodeKind::Movie]),
                    )
                    .and_where(
                        Expr::col((playable_nodes.clone(), nodes::Column::Id)).not_in_subquery(
                            Query::select()
                                .column(node_metadata::Column::NodeId)
                                .from(node_metadata::Entity)
                                .and_where(
                                    Expr::col((
                                        node_metadata::Entity,
                                        node_metadata::Column::Source,
                                    ))
                                    .eq(MetadataSource::Remote),
                                )
                                .to_owned(),
                        ),
                    )
                    .to_owned(),
            ))
            .order_by_asc(nodes::Column::Id)
    }

    fn target_id(&self, target: &Self::Model) -> String {
        target.id.clone()
    }

    async fn run(
        &self,
        db: &DatabaseConnection,
        group_node: Self::Model,
        _ctx: &JobLease,
    ) -> anyhow::Result<JobOutcome> {
        let root_node = group_node.clone();
        let group_items = load_group_items(db, &group_node).await?;
        if group_items.is_empty() {
            return Ok(JobOutcome::Complete);
        }

        let candidates = decode_root_candidates(root_node.match_candidates_json.as_deref())?;
        if candidates.is_empty() {
            return Ok(JobOutcome::Complete);
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
                                db,
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
                        db,
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
                    if let Err(error) = clear_remote_node_metadata_for_batch(db, &node_ids).await {
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

        Ok(JobOutcome::Complete)
    }
}

async fn load_group_items(
    db: &DatabaseConnection,
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
        NodeKind::Series => query,
        NodeKind::Movie => query.filter(nodes::Column::Id.eq(group_node.id.clone())),
        _ => query.filter(nodes::Column::Id.eq("__never__")),
    };

    Ok(query.all(db).await?)
}

struct GroupMatchSummary {
    unmatched_node_ids: Vec<String>,
}

async fn match_series_group(
    db: &DatabaseConnection,
    provider: &dyn MetadataProvider,
    provider_id: &str,
    group_node: &nodes::Model,
    candidate: &SeriesCandidate,
    group_items: &[nodes::Model],
) -> anyhow::Result<GroupMatchSummary> {
    let items = group_items
        .iter()
        .map(|node| SeriesItem {
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

    let items_result = provider
        .lookup_series_items(SeriesItemsRequest {
            root_id: group_node.root_id.clone(),
            candidate: candidate.clone(),
            items,
        })
        .await?;
    let season_nodes = nodes::Entity::find()
        .filter(nodes::Column::RootId.eq(group_node.root_id.clone()))
        .filter(nodes::Column::Kind.eq(NodeKind::Season))
        .all(db)
        .await?;

    let unmatched_node_ids = overwrite_remote_episode_metadata_for_batch(
        db,
        provider_id,
        group_items,
        &items_result.episodes,
        chrono::Utc::now().timestamp(),
    )
    .await
    .map(|_| Vec::new())?;
    overwrite_remote_season_metadata_for_batch(
        db,
        provider_id,
        &season_nodes,
        &items_result.seasons,
        chrono::Utc::now().timestamp(),
    )
    .await?;

    Ok(GroupMatchSummary { unmatched_node_ids })
}
