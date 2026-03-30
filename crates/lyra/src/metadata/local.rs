use crate::metadata::store::{
    delete_local_node_metadata_for_root_except, upsert_local_node_metadata_rows,
};
use sea_orm::ConnectionTrait;
use std::collections::HashMap;

pub const LOCAL_METADATA_PROVIDER_ID: &str = "local";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeLocalMetadataInput {
    pub node_id: String,
    pub name: String,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LocalMetadataPlan {
    pub root_id: String,
    pub nodes: HashMap<String, NodeLocalMetadataInput>,
}

pub fn upsert_node_local_metadata_input(
    plan: &mut LocalMetadataPlan,
    next: NodeLocalMetadataInput,
) {
    match plan.nodes.entry(next.node_id.clone()) {
        std::collections::hash_map::Entry::Vacant(entry) => {
            entry.insert(next);
        }
        std::collections::hash_map::Entry::Occupied(mut entry) => {
            let existing = entry.get_mut();
            existing.name = next.name;
            if existing.imdb_id.is_none() {
                existing.imdb_id = next.imdb_id;
            }
            if existing.tmdb_id.is_none() {
                existing.tmdb_id = next.tmdb_id;
            }
        }
    }
}

pub async fn replace_local_metadata_for_root(
    pool: &impl ConnectionTrait,
    plan: &LocalMetadataPlan,
    now: i64,
) -> anyhow::Result<()> {
    let desired_node_ids = plan.nodes.keys().cloned().collect::<Vec<_>>();
    delete_local_node_metadata_for_root_except(pool, &plan.root_id, &desired_node_ids).await?;

    let rows = plan
        .nodes
        .values()
        .cloned()
        .collect::<Vec<NodeLocalMetadataInput>>();
    upsert_local_node_metadata_rows(pool, &rows, now).await?;

    Ok(())
}
