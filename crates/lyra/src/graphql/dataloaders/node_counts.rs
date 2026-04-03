use crate::entities::{node_closure, nodes};
use async_graphql::dataloader::Loader;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult, JoinType, QueryFilter,
    QuerySelect, RelationTrait, sea_query::Expr,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default)]
pub struct NodeCounts {
    pub season_count: i64,
    pub episode_count: i64,
}

#[derive(Clone)]
pub struct NodeCountsLoader {
    pool: DatabaseConnection,
}

impl NodeCountsLoader {
    pub fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromQueryResult)]
struct NodeCountRow {
    ancestor_id: String,
    kind: nodes::NodeKind,
    count: i64,
}

impl Loader<String> for NodeCountsLoader {
    type Value = NodeCounts;
    type Error = String;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut counts = keys
            .iter()
            .cloned()
            .map(|node_id| (node_id, NodeCounts::default()))
            .collect::<HashMap<_, _>>();

        let rows = node_closure::Entity::find()
            .join(
                JoinType::InnerJoin,
                node_closure::Relation::Descendant.def(),
            )
            .filter(node_closure::Column::AncestorId.is_in(keys.to_vec()))
            .filter(nodes::Column::Kind.is_in([nodes::NodeKind::Season, nodes::NodeKind::Episode]))
            .select_only()
            .column_as(node_closure::Column::AncestorId, "ancestor_id")
            .column_as(nodes::Column::Kind, "kind")
            .column_as(
                Expr::col(node_closure::Column::DescendantId).count(),
                "count",
            )
            .group_by(node_closure::Column::AncestorId)
            .group_by(nodes::Column::Kind)
            .into_model::<NodeCountRow>()
            .all(&self.pool)
            .await
            .map_err(|error| error.to_string())?;

        for row in rows {
            let entry = counts.entry(row.ancestor_id).or_default();
            match row.kind {
                nodes::NodeKind::Season => entry.season_count = row.count,
                nodes::NodeKind::Episode => entry.episode_count = row.count,
                nodes::NodeKind::Movie | nodes::NodeKind::Series => {}
            }
        }

        Ok(counts)
    }
}
