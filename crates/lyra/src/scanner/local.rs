use crate::entities::{
    metadata::{self, MetadataKind},
    node_metadata,
    nodes::{self, NodeKind},
};
use crate::scanner::ensure_node_metadata_link;
use anyhow::{Result, anyhow};
use lyra_parser::ParsedFile;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, JoinType,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
};

pub async fn upsert_local_metadata_for_node(
    pool: &DatabaseConnection,
    node: &nodes::Model,
    parsed: &ParsedFile,
    episode_number_hint: Option<i32>,
) -> Result<()> {
    let (kind, season_number, episode_number) =
        metadata_shape_for_node(node, parsed, episode_number_hint)?;
    let (root_id, parent_id) = resolve_local_lineage(pool, node).await?;
    let now = chrono::Utc::now().timestamp();

    let existing = metadata::Entity::find()
        .join(JoinType::InnerJoin, metadata::Relation::NodeMetadata.def())
        .filter(node_metadata::Column::NodeId.eq(node.id.clone()))
        .filter(metadata::Column::Source.eq("local"))
        .order_by_asc(metadata::Column::Id)
        .all(pool)
        .await?;

    let local_metadata = if let Some(existing_local) = existing.first() {
        let mut active = existing_local.clone().into_active_model();
        active.root_id = Set(root_id);
        active.parent_id = Set(parent_id);
        active.kind = Set(kind);
        active.source = Set("local".to_string());
        active.source_key = Set(None);
        active.name = Set(node.name.clone());
        active.description = Set(None);
        active.score_display = Set(None);
        active.score_normalized = Set(None);
        active.season_number = Set(season_number);
        active.episode_number = Set(episode_number);
        active.released_at = Set(None);
        active.ended_at = Set(None);
        active.poster_asset_id = Set(None);
        active.thumbnail_asset_id = Set(None);
        active.background_asset_id = Set(None);
        active.updated_at = Set(now);
        active.update(pool).await?
    } else {
        metadata::Entity::insert(metadata::ActiveModel {
            root_id: Set(root_id),
            parent_id: Set(parent_id),
            source: Set("local".to_string()),
            source_key: Set(None),
            kind: Set(kind),
            name: Set(node.name.clone()),
            description: Set(None),
            score_display: Set(None),
            score_normalized: Set(None),
            season_number: Set(season_number),
            episode_number: Set(episode_number),
            released_at: Set(None),
            ended_at: Set(None),
            poster_asset_id: Set(None),
            thumbnail_asset_id: Set(None),
            background_asset_id: Set(None),
            updated_at: Set(now),
            ..Default::default()
        })
        .exec_with_returning(pool)
        .await?
    };

    ensure_node_metadata_link(pool, &node.id, local_metadata.id, true).await?;
    Ok(())
}

fn metadata_shape_for_node(
    node: &nodes::Model,
    parsed: &ParsedFile,
    episode_number_hint: Option<i32>,
) -> Result<(MetadataKind, Option<i64>, Option<i64>)> {
    let parsed_season_number = parsed_season_number(parsed);

    match node.kind {
        NodeKind::Movie => Ok((MetadataKind::Movie, None, None)),
        NodeKind::Series => Ok((MetadataKind::Series, None, None)),
        NodeKind::Season => Ok((
            MetadataKind::Season,
            extract_number_from_name(&node.name, "Season")
                .or(parsed_season_number)
                .map(i64::from),
            None,
        )),
        NodeKind::Episode => {
            let episode_number = episode_number_hint
                .or_else(|| extract_number_from_name(&node.name, "Episode"))
                .or_else(|| parsed_episode_numbers(parsed).first().copied())
                .ok_or_else(|| {
                    anyhow!("episode node '{}' is missing an episode number", node.id)
                })?;

            Ok((
                MetadataKind::Episode,
                parsed_season_number.map(i64::from),
                Some(i64::from(episode_number)),
            ))
        }
    }
}

async fn resolve_local_lineage(
    pool: &DatabaseConnection,
    node: &nodes::Model,
) -> Result<(Option<i64>, Option<i64>)> {
    match node.kind {
        NodeKind::Movie | NodeKind::Series => Ok((None, None)),
        NodeKind::Season | NodeKind::Episode => {
            let root_node_id = node
                .root_id
                .as_ref()
                .ok_or_else(|| anyhow!("node '{}' has no root_id", node.id))?;
            let parent_node_id = node
                .parent_id
                .as_ref()
                .ok_or_else(|| anyhow!("node '{}' has no parent_id", node.id))?;

            let root_metadata_id = get_primary_local_metadata_id(pool, root_node_id)
                .await?
                .ok_or_else(|| {
                    anyhow!(
                        "missing local root metadata for node '{}' (root '{}')",
                        node.id,
                        root_node_id
                    )
                })?;

            let parent_metadata_id = get_primary_local_metadata_id(pool, parent_node_id)
                .await?
                .ok_or_else(|| {
                    anyhow!(
                        "missing local parent metadata for node '{}' (parent '{}')",
                        node.id,
                        parent_node_id
                    )
                })?;

            Ok((Some(root_metadata_id), Some(parent_metadata_id)))
        }
    }
}

async fn get_primary_local_metadata_id(
    pool: &DatabaseConnection,
    node_id: &str,
) -> Result<Option<i64>> {
    let metadata = metadata::Entity::find()
        .join(JoinType::InnerJoin, metadata::Relation::NodeMetadata.def())
        .filter(node_metadata::Column::NodeId.eq(node_id.to_string()))
        .filter(node_metadata::Column::IsPrimary.eq(true))
        .filter(metadata::Column::Source.eq("local"))
        .one(pool)
        .await?;

    Ok(metadata.map(|row| row.id))
}

fn extract_number_from_name(name: &str, prefix: &str) -> Option<i32> {
    let trimmed = name.trim();
    if !trimmed.starts_with(prefix) {
        return None;
    }

    trimmed
        .trim_start_matches(prefix)
        .trim()
        .parse::<i32>()
        .ok()
}

fn parsed_season_number(parsed: &ParsedFile) -> Option<i32> {
    parsed
        .season_numbers
        .iter()
        .filter_map(|&season| i32::try_from(season).ok())
        .min()
}

fn parsed_episode_numbers(parsed: &ParsedFile) -> Vec<i32> {
    let mut episodes = parsed
        .episode_numbers
        .iter()
        .filter_map(|&episode| i32::try_from(episode).ok())
        .collect::<Vec<_>>();
    episodes.sort_unstable();
    episodes.dedup();
    episodes
}
