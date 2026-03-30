use crate::entities::{
    jobs as jobs_entity, metadata_source::MetadataSource, node_metadata, nodes, nodes::NodeKind,
};
use crate::jobs::{Job, JobExecutionPolicy, JobLease, JobOutcome};
use crate::metadata::METADATA_RETRY_BACKOFF_SECONDS;
use crate::metadata::sync;
use lyra_metadata::MetadataProvider;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Select,
    sea_query::{Expr, Query},
};
use std::sync::Arc;

pub struct NodeMetadataSyncRootJob {
    providers: Vec<Arc<dyn MetadataProvider>>,
}

impl NodeMetadataSyncRootJob {
    pub fn new(providers: Vec<Arc<dyn MetadataProvider>>) -> Self {
        Self { providers }
    }
}

#[async_trait::async_trait]
impl Job for NodeMetadataSyncRootJob {
    type Entity = nodes::Entity;
    type Model = nodes::Model;

    const JOB_KIND: jobs_entity::JobKind = jobs_entity::JobKind::NodeSyncMetadataRoot;

    fn execution_policy(&self) -> JobExecutionPolicy {
        JobExecutionPolicy::with_backoff_seconds(METADATA_RETRY_BACKOFF_SECONDS)
    }

    fn query(&self) -> Select<Self::Entity> {
        nodes::Entity::find()
            .filter(nodes::Column::ParentId.is_null())
            .filter(nodes::Column::Kind.is_in([NodeKind::Movie, NodeKind::Series]))
            .filter(Expr::exists(local_metadata_exists_query()))
            .filter(
                Condition::any()
                    .add(
                        Condition::all()
                            .not()
                            .add(Expr::exists(remote_metadata_exists_query())),
                    )
                    .add(Expr::exists(stale_remote_metadata_query())),
            )
            .order_by_asc(nodes::Column::LastAddedAt)
            .order_by_asc(nodes::Column::Id)
    }

    fn target_id(&self, target: &Self::Model) -> String {
        target.id.clone()
    }

    async fn run(
        &self,
        db: &DatabaseConnection,
        root: Self::Model,
        _ctx: &JobLease,
    ) -> anyhow::Result<JobOutcome> {
        sync::sync_root(db, &self.providers, &root).await?;
        Ok(JobOutcome::Complete)
    }
}

fn local_metadata_exists_query() -> sea_orm::sea_query::SelectStatement {
    Query::select()
        .expr(Expr::val(1))
        .from(node_metadata::Entity)
        .and_where(
            Expr::col((node_metadata::Entity, node_metadata::Column::NodeId))
                .equals((nodes::Entity, nodes::Column::Id)),
        )
        .and_where(
            Expr::col((node_metadata::Entity, node_metadata::Column::Source))
                .eq(MetadataSource::Local),
        )
        .to_owned()
}

fn remote_metadata_exists_query() -> sea_orm::sea_query::SelectStatement {
    Query::select()
        .expr(Expr::val(1))
        .from(node_metadata::Entity)
        .and_where(
            Expr::col((node_metadata::Entity, node_metadata::Column::NodeId))
                .equals((nodes::Entity, nodes::Column::Id)),
        )
        .and_where(
            Expr::col((node_metadata::Entity, node_metadata::Column::Source))
                .eq(MetadataSource::Remote),
        )
        .to_owned()
}

fn stale_remote_metadata_query() -> sea_orm::sea_query::SelectStatement {
    Query::select()
        .expr(Expr::val(1))
        .from(node_metadata::Entity)
        .and_where(
            Expr::col((node_metadata::Entity, node_metadata::Column::NodeId))
                .equals((nodes::Entity, nodes::Column::Id)),
        )
        .and_where(
            Expr::col((node_metadata::Entity, node_metadata::Column::Source))
                .eq(MetadataSource::Remote),
        )
        .and_where(
            Expr::col((node_metadata::Entity, node_metadata::Column::UpdatedAt))
                .lt(Expr::col((nodes::Entity, nodes::Column::UpdatedAt))),
        )
        .to_owned()
}
