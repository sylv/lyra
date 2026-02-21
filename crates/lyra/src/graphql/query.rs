use crate::{
    auth::RequestAuth,
    entities::{
        libraries, metadata, node_metadata,
        nodes::{self, NodeKind},
        tasks as tasks_entity, watch_progress,
    },
};
use async_graphql::{
    Context, Enum, InputObject, Object, SimpleObject,
    connection::{self, EmptyFields},
};
use lazy_static::lazy_static;
use regex::Regex;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, Order, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait,
};
use std::collections::HashMap;
use tokio::task::spawn_blocking;

const ACTIVE_TASK_RECENT_WINDOW_SECS: i64 = 60 * 60 * 24;

#[derive(Debug, InputObject, serde::Deserialize)]
pub struct NodeFilter {
    pub parent_id: Option<String>,
    pub season_numbers: Option<Vec<i64>>,
    pub kinds: Option<Vec<NodeKind>>,
    pub search: Option<String>,
    pub order_by: Option<NodeOrderBy>,
    pub order_direction: Option<NodeOrderDirection>,
    pub watched: Option<bool>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum, serde::Deserialize)]
#[graphql(name = "OrderDirection")]
pub enum NodeOrderDirection {
    Asc,
    Desc,
}

impl NodeOrderDirection {
    pub fn to_sea_orm(self) -> Order {
        match self {
            NodeOrderDirection::Asc => Order::Asc,
            NodeOrderDirection::Desc => Order::Desc,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum, serde::Deserialize)]
#[graphql(name = "NodeOrderBy")]
pub enum NodeOrderBy {
    AddedAt,
    ReleasedAt,
    Alphabetical,
    Rating,
    SeasonEpisode,
}

impl NodeOrderBy {
    pub fn get_default_direction(self) -> NodeOrderDirection {
        match self {
            NodeOrderBy::AddedAt | NodeOrderBy::ReleasedAt | NodeOrderBy::Rating => {
                NodeOrderDirection::Desc
            }
            NodeOrderBy::Alphabetical | NodeOrderBy::SeasonEpisode => NodeOrderDirection::Asc,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(name = "ActiveTask")]
pub struct ActiveTask {
    pub task_type: String,
    pub title: String,
    pub current: i64,
    pub total: i64,
    pub progress_percent: f64,
}

pub struct Query;

#[Object]
impl Query {
    async fn node_list(
        &self,
        ctx: &Context<'_>,
        filter: NodeFilter,
        after: Option<String>,
        first: Option<i32>,
    ) -> Result<
        connection::Connection<u64, nodes::Model, EmptyFields, EmptyFields>,
        async_graphql::Error,
    > {
        connection::query(
            after,
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let pool = ctx.data::<DatabaseConnection>()?;
                let mut qb = nodes::Entity::find()
                    .join(
                        sea_orm::JoinType::LeftJoin,
                        nodes::Relation::NodeMetadata.def(),
                    )
                    .join(
                        sea_orm::JoinType::LeftJoin,
                        node_metadata::Relation::Metadata.def(),
                    )
                    .filter(node_metadata::Column::IsPrimary.eq(true));

                if let Some(parent_id) = &filter.parent_id {
                    let querying_episodes = filter
                        .kinds
                        .as_ref()
                        .map(|kinds| kinds.contains(&NodeKind::Episode))
                        .unwrap_or(false);

                    if querying_episodes {
                        qb = qb.filter(nodes::Column::RootId.eq(parent_id.clone()));
                    } else {
                        qb = qb.filter(nodes::Column::ParentId.eq(parent_id.clone()));
                    }
                } else {
                    qb = qb.filter(nodes::Column::ParentId.is_null());
                }

                if let Some(season_numbers) = &filter.season_numbers {
                    qb = qb.filter(metadata::Column::SeasonNumber.is_in(season_numbers.clone()));
                }

                if let Some(kinds) = &filter.kinds {
                    qb = qb.filter(nodes::Column::Kind.is_in(kinds.clone()));
                }

                if let Some(search) = &filter.search {
                    qb = qb.filter(metadata::Column::Name.contains(search));
                }

                if let Some(watched) = filter.watched {
                    let auth = ctx.data::<RequestAuth>()?;
                    let user = auth.get_user_or_err()?;
                    let watched_ids: Vec<String> = watch_progress::Entity::find()
                        .filter(watch_progress::Column::UserId.eq(user.id.clone()))
                        .filter(watch_progress::Column::NodeId.is_not_null())
                        .select_only()
                        .column(watch_progress::Column::NodeId)
                        .into_tuple()
                        .all(pool)
                        .await?;

                    if watched {
                        if watched_ids.is_empty() {
                            qb = qb.filter(nodes::Column::Id.eq("__never__"));
                        } else {
                            qb = qb.filter(nodes::Column::Id.is_in(watched_ids));
                        }
                    } else if !watched_ids.is_empty() {
                        qb = qb.filter(nodes::Column::Id.is_not_in(watched_ids));
                    }
                }

                let order_by = filter.order_by.unwrap_or(NodeOrderBy::Alphabetical);
                let order_direction = filter
                    .order_direction
                    .unwrap_or_else(|| order_by.get_default_direction())
                    .to_sea_orm();

                match order_by {
                    NodeOrderBy::AddedAt => {
                        qb = qb.order_by(nodes::Column::Id, order_direction);
                    }
                    NodeOrderBy::ReleasedAt => {
                        qb = qb.order_by(metadata::Column::ReleasedAt, order_direction);
                    }
                    NodeOrderBy::Alphabetical => {
                        qb = qb.order_by(metadata::Column::Name, order_direction);
                    }
                    NodeOrderBy::Rating => {
                        qb = qb.order_by(metadata::Column::ScoreNormalized, order_direction);
                    }
                    NodeOrderBy::SeasonEpisode => {
                        qb = qb
                            .order_by(metadata::Column::SeasonNumber, order_direction.clone())
                            .order_by(metadata::Column::EpisodeNumber, order_direction);
                    }
                }

                let count = qb.clone().count(pool).await?;
                let limit: u64 = first.unwrap_or(25) as u64;
                let offset: u64 = after.map(|a| a + 1).unwrap_or(0);

                let nodes = qb
                    .limit(Some(limit))
                    .offset(Some(offset))
                    .all(pool)
                    .await
                    .map_err(|e| async_graphql::Error::new(e.to_string()))?;

                let has_previous_page = offset > 0;
                let has_next_page = offset + limit < count;

                let mut connection = connection::Connection::new(has_previous_page, has_next_page);
                connection
                    .edges
                    .extend(nodes.into_iter().enumerate().map(|(index, node)| {
                        let cursor = (offset + index as u64) as u64;
                        connection::Edge::new(cursor, node)
                    }));

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn node(
        &self,
        ctx: &Context<'_>,
        node_id: String,
    ) -> Result<nodes::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let node = nodes::Entity::find_by_id(node_id)
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("Node not found".to_string()))?;

        Ok(node)
    }

    /// Used during library setup to pick the library path
    async fn list_files(&self, path: String) -> Result<Vec<String>, async_graphql::Error> {
        if !path.starts_with('/') || path.contains("..") || path.contains("/.") {
            return Err(async_graphql::Error::new("Invalid path".to_string()));
        }

        spawn_blocking(|| {
            lazy_static! {
                static ref SKIP_PATTERN: Regex =
                    Regex::new(r"^/(etc|proc|sys|dev|run|boot|lib|lib64|sbin|bin|var)").unwrap();
            }

            let mut dirs = std::fs::read_dir(path)?
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.starts_with('.') {
                        return None;
                    }

                    let full_path = e.path().to_string_lossy().to_string();
                    if SKIP_PATTERN.is_match(&full_path) {
                        return None;
                    }

                    Some(name)
                })
                .collect::<Vec<_>>();

            dirs.sort();
            Ok(dirs)
        })
        .await?
    }

    async fn libraries(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<libraries::Model>, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let libraries = libraries::Entity::find()
            .all(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(libraries)
    }

    async fn active_tasks(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<ActiveTask>, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let now = chrono::Utc::now().timestamp();
        let recent_cutoff = now - ACTIVE_TASK_RECENT_WINDOW_SECS;

        let active_task_types: Vec<String> = tasks_entity::Entity::find()
            .filter(
                Condition::any()
                    .add(tasks_entity::Column::LockedAt.is_not_null())
                    .add(
                        Condition::all()
                            .add(tasks_entity::Column::ExecuteAfter.is_not_null())
                            .add(tasks_entity::Column::ExecuteAfter.lte(now)),
                    ),
            )
            .select_only()
            .column(tasks_entity::Column::TaskType)
            .distinct()
            .into_tuple()
            .all(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        if active_task_types.is_empty() {
            return Ok(Vec::new());
        }

        let all_tasks = tasks_entity::Entity::find()
            .filter(tasks_entity::Column::TaskType.is_in(active_task_types))
            .filter(
                Condition::any()
                    .add(
                        Condition::all()
                            .add(tasks_entity::Column::ExecuteAfter.is_not_null())
                            .add(tasks_entity::Column::ExecuteAfter.gte(recent_cutoff))
                            .add(tasks_entity::Column::ExecuteAfter.lte(now)),
                    )
                    .add(tasks_entity::Column::LockedAt.gte(recent_cutoff))
                    .add(tasks_entity::Column::LastRunAt.gte(recent_cutoff)),
            )
            .all(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let mut grouped: HashMap<String, (i64, i64, i64)> = HashMap::new();
        for task in all_tasks {
            let entry = grouped.entry(task.task_type.clone()).or_insert((0, 0, 0));
            entry.0 += 1;
            if task
                .execute_after
                .is_some_and(|execute_after| execute_after <= now)
            {
                entry.1 += 1;
            }
            if task.locked_at.is_some() {
                entry.2 += 1;
            }
        }

        let mut active = grouped
            .into_iter()
            .filter_map(|(task_type, (total, pending, running))| {
                if pending == 0 && running == 0 {
                    return None;
                }

                let current = (total - pending + running).clamp(0, total);
                let progress_percent = if total > 0 {
                    current as f64 / total as f64
                } else {
                    0.0
                };

                Some(ActiveTask {
                    task_type: task_type.clone(),
                    title: get_task_title(&task_type),
                    current,
                    total,
                    progress_percent,
                })
            })
            .collect::<Vec<_>>();

        active.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(active)
    }
}

fn get_task_title(task_type: &str) -> String {
    match task_type {
        "file.generate_timeline_preview" => "Generating Timeline Previews".to_string(),
        _ => humanize_task_type(task_type),
    }
}

fn humanize_task_type(task_type: &str) -> String {
    task_type
        .split(['.', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
