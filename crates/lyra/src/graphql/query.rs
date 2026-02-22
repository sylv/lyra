use crate::{
    auth::RequestAuth,
    entities::{
        item_metadata, items, libraries, root_metadata,
        roots::{self, RootKind},
        seasons, tasks as tasks_entity, watch_progress,
    },
};
use async_graphql::{
    Context, Enum, InputObject, Object, SimpleObject,
    connection::{self, EmptyFields},
};
use lazy_static::lazy_static;
use regex::Regex;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, JoinType, Order, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use std::collections::HashMap;
use tokio::task::spawn_blocking;

const ACTIVE_TASK_RECENT_WINDOW_SECS: i64 = 60 * 60 * 24;

#[derive(Debug, InputObject, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootNodeFilter {
    pub library_id: Option<i64>,
    pub kinds: Option<Vec<RootKind>>,
    pub order_by: Option<RootNodeOrderBy>,
    pub order_direction: Option<OrderDirection>,
    pub watched: Option<bool>,
}

#[derive(Debug, InputObject, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemNodeFilter {
    pub root_id: String,
    pub season_numbers: Option<Vec<i64>>,
    pub order_by: Option<ItemNodeOrderBy>,
    pub order_direction: Option<OrderDirection>,
    pub watched: Option<bool>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum, serde::Deserialize)]
#[graphql(name = "OrderDirection")]
pub enum OrderDirection {
    Asc,
    Desc,
}

impl OrderDirection {
    pub fn to_sea_orm(self) -> Order {
        match self {
            OrderDirection::Asc => Order::Asc,
            OrderDirection::Desc => Order::Desc,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum, serde::Deserialize)]
pub enum RootNodeOrderBy {
    AddedAt,
    ReleasedAt,
    Alphabetical,
    Rating,
    SeasonEpisode,
}

impl RootNodeOrderBy {
    pub fn default_direction(self) -> OrderDirection {
        match self {
            RootNodeOrderBy::AddedAt | RootNodeOrderBy::ReleasedAt | RootNodeOrderBy::Rating => {
                OrderDirection::Desc
            }
            RootNodeOrderBy::Alphabetical | RootNodeOrderBy::SeasonEpisode => OrderDirection::Asc,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum, serde::Deserialize)]
pub enum ItemNodeOrderBy {
    AddedAt,
    ReleasedAt,
    Alphabetical,
    Rating,
    SeasonEpisode,
}

impl ItemNodeOrderBy {
    pub fn default_direction(self) -> OrderDirection {
        match self {
            ItemNodeOrderBy::AddedAt | ItemNodeOrderBy::ReleasedAt | ItemNodeOrderBy::Rating => {
                OrderDirection::Desc
            }
            ItemNodeOrderBy::SeasonEpisode => OrderDirection::Asc,
            ItemNodeOrderBy::Alphabetical => OrderDirection::Asc,
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
    async fn root_list(
        &self,
        ctx: &Context<'_>,
        filter: RootNodeFilter,
        after: Option<String>,
        first: Option<i32>,
    ) -> Result<
        connection::Connection<u64, roots::Model, EmptyFields, EmptyFields>,
        async_graphql::Error,
    > {
        connection::query(
            after,
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let pool = ctx.data::<DatabaseConnection>()?;
                let mut qb = roots::Entity::find()
                    .join(JoinType::LeftJoin, roots::Relation::RootMetadata.def())
                    .filter(
                        Condition::any()
                            .add(root_metadata::Column::IsPrimary.eq(true))
                            .add(root_metadata::Column::Id.is_null()),
                    );

                if let Some(kinds) = &filter.kinds {
                    qb = qb.filter(roots::Column::Kind.is_in(kinds.clone()));
                }

                if let Some(library_id) = filter.library_id {
                    qb = qb.filter(roots::Column::LibraryId.eq(library_id));
                }

                if let Some(watched) = filter.watched {
                    let auth = ctx.data::<RequestAuth>()?;
                    let user = auth.get_user_or_err()?;

                    let watched_root_ids: Vec<String> = items::Entity::find()
                        .join(JoinType::InnerJoin, items::Relation::WatchProgress.def())
                        .filter(watch_progress::Column::UserId.eq(user.id.clone()))
                        .select_only()
                        .column(items::Column::RootId)
                        .distinct()
                        .into_tuple()
                        .all(pool)
                        .await?;

                    if watched {
                        if watched_root_ids.is_empty() {
                            qb = qb.filter(roots::Column::Id.eq("__never__"));
                        } else {
                            qb = qb.filter(roots::Column::Id.is_in(watched_root_ids));
                        }
                    } else if !watched_root_ids.is_empty() {
                        qb = qb.filter(roots::Column::Id.is_not_in(watched_root_ids));
                    }
                }

                let order_by = filter.order_by.unwrap_or(RootNodeOrderBy::Alphabetical);
                let order_direction = filter
                    .order_direction
                    .unwrap_or_else(|| order_by.default_direction())
                    .to_sea_orm();

                match order_by {
                    RootNodeOrderBy::AddedAt => {
                        qb = qb.order_by(roots::Column::LastAddedAt, order_direction);
                    }
                    RootNodeOrderBy::ReleasedAt => {
                        qb = qb.order_by(root_metadata::Column::ReleasedAt, order_direction);
                    }
                    RootNodeOrderBy::Alphabetical => {
                        qb = qb.order_by(root_metadata::Column::Name, order_direction);
                    }
                    RootNodeOrderBy::Rating => {
                        qb = qb.order_by(root_metadata::Column::ScoreNormalized, order_direction);
                    }
                    RootNodeOrderBy::SeasonEpisode => {
                        qb = qb.order_by(roots::Column::LastAddedAt, order_direction);
                    }
                }

                qb = qb.order_by_asc(roots::Column::Id);

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

    async fn item_list(
        &self,
        ctx: &Context<'_>,
        filter: ItemNodeFilter,
        after: Option<String>,
        first: Option<i32>,
    ) -> Result<
        connection::Connection<u64, items::Model, EmptyFields, EmptyFields>,
        async_graphql::Error,
    > {
        connection::query(
            after,
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let pool = ctx.data::<DatabaseConnection>()?;
                let mut qb = items::Entity::find()
                    .filter(items::Column::RootId.eq(filter.root_id.clone()))
                    .join(JoinType::LeftJoin, items::Relation::ItemMetadata.def())
                    .join(JoinType::LeftJoin, items::Relation::Seasons.def())
                    .filter(
                        Condition::any()
                            .add(item_metadata::Column::IsPrimary.eq(true))
                            .add(item_metadata::Column::Id.is_null()),
                    );

                if let Some(season_numbers) = &filter.season_numbers {
                    qb = qb.filter(seasons::Column::SeasonNumber.is_in(season_numbers.clone()));
                }

                if let Some(watched) = filter.watched {
                    let auth = ctx.data::<RequestAuth>()?;
                    let user = auth.get_user_or_err()?;
                    let watched_item_ids: Vec<String> = watch_progress::Entity::find()
                        .filter(watch_progress::Column::UserId.eq(user.id.clone()))
                        .select_only()
                        .column(watch_progress::Column::ItemId)
                        .into_tuple()
                        .all(pool)
                        .await?;

                    if watched {
                        if watched_item_ids.is_empty() {
                            qb = qb.filter(items::Column::Id.eq("__never__"));
                        } else {
                            qb = qb.filter(items::Column::Id.is_in(watched_item_ids));
                        }
                    } else if !watched_item_ids.is_empty() {
                        qb = qb.filter(items::Column::Id.is_not_in(watched_item_ids));
                    }
                }

                let order_by = filter.order_by.unwrap_or(ItemNodeOrderBy::SeasonEpisode);
                let order_direction = filter
                    .order_direction
                    .unwrap_or_else(|| order_by.default_direction())
                    .to_sea_orm();

                match order_by {
                    ItemNodeOrderBy::AddedAt => {
                        qb = qb.order_by(items::Column::LastAddedAt, order_direction);
                    }
                    ItemNodeOrderBy::ReleasedAt => {
                        qb = qb.order_by(item_metadata::Column::ReleasedAt, order_direction);
                    }
                    ItemNodeOrderBy::Alphabetical => {
                        qb = qb.order_by(item_metadata::Column::Name, order_direction);
                    }
                    ItemNodeOrderBy::Rating => {
                        qb = qb.order_by(item_metadata::Column::ScoreNormalized, order_direction);
                    }
                    ItemNodeOrderBy::SeasonEpisode => {
                        qb = qb.order_by(items::Column::Order, order_direction);
                    }
                }

                qb = qb.order_by_asc(items::Column::Id);

                let count = qb.clone().count(pool).await?;
                let limit: u64 = first.unwrap_or(50) as u64;
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

    async fn root(
        &self,
        ctx: &Context<'_>,
        root_id: String,
    ) -> Result<roots::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        roots::Entity::find_by_id(root_id)
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("Root not found".to_string()))
    }

    async fn season(
        &self,
        ctx: &Context<'_>,
        season_id: String,
    ) -> Result<seasons::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        seasons::Entity::find_by_id(season_id)
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("Season not found".to_string()))
    }

    async fn item(
        &self,
        ctx: &Context<'_>,
        item_id: String,
    ) -> Result<items::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        items::Entity::find_by_id(item_id)
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("Item not found".to_string()))
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
                    title: humanize_task_type(&task_type),
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
