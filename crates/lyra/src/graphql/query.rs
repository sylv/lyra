use crate::{
    auth::RequestAuth,
    entities::{
        item_metadata, items, jobs as jobs_entity, libraries,
        metadata_source::MetadataSource,
        root_metadata,
        roots::{self, RootKind},
        seasons, watch_progress,
    },
    jobs,
};
use async_graphql::{
    Context, Enum, InputObject, Object, SimpleObject,
    connection::{self, EmptyFields},
};
use lazy_static::lazy_static;
use regex::Regex;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, JoinType, Order, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, prelude::Expr,
};
use std::collections::HashMap;
use tokio::task::spawn_blocking;

const DIRECTORY_PRIORITY_HINTS: &[&str] = &[
    "mnt",
    "media",
    "series",
    "tv",
    "movies",
    "movie",
    "shows",
    "show",
    "pool",
    "array",
    "video",
    "videos",
    "library",
    "libraries",
];

fn directory_sort_key(name: &str) -> (u8, usize, String) {
    let lower = name.to_ascii_lowercase();

    for (index, hint) in DIRECTORY_PRIORITY_HINTS.iter().enumerate() {
        if lower == *hint {
            return (0, index, lower);
        }
    }

    for (index, hint) in DIRECTORY_PRIORITY_HINTS.iter().enumerate() {
        if lower.starts_with(hint) {
            return (1, index, lower);
        }
    }

    for (index, hint) in DIRECTORY_PRIORITY_HINTS.iter().enumerate() {
        if lower.contains(hint) {
            return (2, index, lower);
        }
    }

    (3, usize::MAX, lower)
}

#[derive(Debug, InputObject, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootNodeFilter {
    pub library_id: Option<i64>,
    pub kinds: Option<Vec<RootKind>>,
    pub order_by: Option<OrderBy>,
    pub order_direction: Option<OrderDirection>,
    pub watched: Option<bool>,
}

#[derive(Debug, InputObject, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemNodeFilter {
    pub root_id: String,
    pub season_numbers: Option<Vec<i64>>,
    pub order_by: Option<OrderBy>,
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
pub enum OrderBy {
    AddedAt,
    ReleasedAt,
    Alphabetical,
    Rating,
    SeasonEpisode,
}

impl OrderBy {
    pub fn default_direction(self) -> OrderDirection {
        match self {
            OrderBy::AddedAt | OrderBy::ReleasedAt | OrderBy::Rating => OrderDirection::Desc,
            OrderBy::Alphabetical | OrderBy::SeasonEpisode => OrderDirection::Asc,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct Activity {
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
                            .add(root_metadata::Column::Id.is_null())
                            .add(root_metadata::Column::Source.eq(MetadataSource::Remote))
                            .add(
                                Condition::all()
                                    .add(root_metadata::Column::Source.eq(MetadataSource::Local))
                                    .add(Expr::cust(
                                        "NOT EXISTS (
                                            SELECT 1
                                            FROM root_metadata rm2
                                            WHERE rm2.root_id = root_metadata.root_id
                                              AND rm2.source = 1
                                        )",
                                    )),
                            ),
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

                let order_by = filter.order_by.unwrap_or(OrderBy::Alphabetical);
                let order_direction = filter
                    .order_direction
                    .unwrap_or_else(|| order_by.default_direction())
                    .to_sea_orm();

                match order_by {
                    OrderBy::AddedAt => {
                        qb = qb.order_by(roots::Column::LastAddedAt, order_direction);
                    }
                    OrderBy::ReleasedAt => {
                        qb = qb.order_by(root_metadata::Column::ReleasedAt, order_direction);
                    }
                    OrderBy::Alphabetical => {
                        qb = qb.order_by(root_metadata::Column::Name, order_direction);
                    }
                    OrderBy::Rating => {
                        qb = qb.order_by(root_metadata::Column::ScoreNormalized, order_direction);
                    }
                    OrderBy::SeasonEpisode => {
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
                            .add(item_metadata::Column::Id.is_null())
                            .add(item_metadata::Column::Source.eq(MetadataSource::Remote))
                            .add(
                                Condition::all()
                                    .add(item_metadata::Column::Source.eq(MetadataSource::Local))
                                    .add(Expr::cust(
                                        "NOT EXISTS (
                                            SELECT 1
                                            FROM item_metadata im2
                                            WHERE im2.item_id = item_metadata.item_id
                                              AND im2.source = 1
                                        )",
                                    )),
                            ),
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

                let order_by = filter.order_by.unwrap_or(OrderBy::SeasonEpisode);
                let order_direction = filter
                    .order_direction
                    .unwrap_or_else(|| order_by.default_direction())
                    .to_sea_orm();

                match order_by {
                    OrderBy::AddedAt => {
                        qb = qb.order_by(items::Column::LastAddedAt, order_direction);
                    }
                    OrderBy::ReleasedAt => {
                        qb = qb.order_by(item_metadata::Column::ReleasedAt, order_direction);
                    }
                    OrderBy::Alphabetical => {
                        qb = qb.order_by(item_metadata::Column::Name, order_direction);
                    }
                    OrderBy::Rating => {
                        qb = qb.order_by(item_metadata::Column::ScoreNormalized, order_direction);
                    }
                    OrderBy::SeasonEpisode => {
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

            dirs.sort_by_cached_key(|name| directory_sort_key(name));
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

    async fn activities(&self, ctx: &Context<'_>) -> Result<Vec<Activity>, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let completed_rows: Vec<(jobs_entity::JobKind, i64)> = jobs_entity::Entity::find()
            .select_only()
            .column(jobs_entity::Column::JobKind)
            .column_as(jobs_entity::Column::Id.count(), "completed_count")
            .filter(jobs_entity::Column::RunAfter.is_null())
            .filter(jobs_entity::Column::LastRunAt.gt(0))
            .filter(jobs_entity::Column::AttemptCount.eq(0))
            .group_by(jobs_entity::Column::JobKind)
            .into_tuple()
            .all(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let completed_by_kind: HashMap<jobs_entity::JobKind, i64> =
            completed_rows.into_iter().collect();

        let pending_rows: Vec<(jobs_entity::JobKind, i64)> = jobs_entity::Entity::find()
            .select_only()
            .column(jobs_entity::Column::JobKind)
            .column_as(jobs_entity::Column::Id.count(), "pending_count")
            .filter(jobs_entity::Column::RunAfter.is_not_null())
            .group_by(jobs_entity::Column::JobKind)
            .into_tuple()
            .all(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let pending_by_kind: HashMap<jobs_entity::JobKind, i64> =
            pending_rows.into_iter().collect();

        let handlers = jobs::registry::get_registered_job_handlers();
        let mut activities = Vec::new();
        for handler in handlers {
            let job_kind = handler.job_kind();
            let pending = *pending_by_kind.get(&job_kind).unwrap_or(&0);
            let completed = *completed_by_kind.get(&job_kind).unwrap_or(&0);
            let total = completed + pending;

            if pending == 0 || total == 0 {
                continue;
            }

            activities.push(Activity {
                task_type: job_kind.code().to_string(),
                title: job_kind.title().to_string(),
                current: completed,
                total,
                progress_percent: completed as f64 / total as f64,
            });
        }

        activities.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(activities)
    }
}
