use crate::{
    activity::ACTIVITY_REGISTRY,
    auth::{AuthenticatedGuard, PermissionGuard, RequestAuth, accessible_library_ids},
    entities::{libraries, node_metadata, nodes, users, watch_progress},
    metadata::read,
    watch_session::WatchSessionRegistry,
};
use async_graphql::{
    Context, Enum, InputObject, Object, SimpleObject,
    connection::{self, EmptyFields},
};
use lazy_static::lazy_static;
use regex::Regex;
use sea_orm::{
    ActiveEnum, ColumnTrait, DatabaseConnection, DbBackend, EntityTrait, Order, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Statement, Value, prelude::Expr,
};
use tokio::task::spawn_blocking;

const SECONDS_PER_DAY: i64 = 86_400;
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
pub struct NodeFilter {
    pub library_id: Option<String>,
    pub root_id: Option<String>,
    pub parent_id: Option<String>,
    pub kinds: Option<Vec<nodes::NodeKind>>,
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
    Order,
}

impl OrderBy {
    pub fn default_direction(self) -> OrderDirection {
        match self {
            OrderBy::AddedAt | OrderBy::ReleasedAt | OrderBy::Rating => OrderDirection::Desc,
            OrderBy::Alphabetical | OrderBy::Order => OrderDirection::Asc,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct Activity {
    pub task_type: String,
    pub title: String,
    pub current: Option<i64>,
    pub total: Option<i64>,
    pub progress_percent: Option<f64>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SearchResults {
    pub roots: Vec<nodes::Model>,
    pub episodes: Vec<nodes::Model>,
}

fn build_fts_query(raw_query: &str) -> Option<String> {
    let mut terms = Vec::new();
    let mut current = String::new();

    for ch in raw_query.chars() {
        if ch.is_alphanumeric() {
            current.push(ch);
            continue;
        }
        if !current.is_empty() {
            terms.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        terms.push(current);
    }

    if terms.is_empty() {
        return None;
    }

    Some(
        terms
            .into_iter()
            .take(8)
            .map(|term| format!("{term}*"))
            .collect::<Vec<_>>()
            .join(" "),
    )
}

// keep the search buckets explicit so the client can render posters first
// without having to reconstruct root-vs-episode ordering from a flat list.
async fn search_nodes_by_kinds(
    pool: &DatabaseConnection,
    fts_query: &str,
    limit: i64,
    kinds: &[nodes::NodeKind],
    visible_library_ids: Option<&[String]>,
) -> Result<Vec<nodes::Model>, sea_orm::DbErr> {
    if kinds.is_empty() {
        return Ok(Vec::new());
    }

    if visible_library_ids.is_some_and(|library_ids| library_ids.is_empty()) {
        return Ok(Vec::new());
    }

    let kind_placeholders = std::iter::repeat_n("?", kinds.len())
        .collect::<Vec<_>>()
        .join(", ");
    let library_sql = visible_library_ids
        .map(|library_ids| {
            let placeholders = std::iter::repeat_n("?", library_ids.len())
                .collect::<Vec<_>>()
                .join(", ");
            format!(" AND nodes.library_id IN ({placeholders})")
        })
        .unwrap_or_default();
    let sql = format!(
        r#"
            SELECT DISTINCT nodes.*
            FROM nodes
            JOIN node_search_fts ON node_search_fts.node_id = nodes.id
            WHERE node_search_fts MATCH ?
              AND nodes.kind IN ({kind_placeholders}){library_sql}
            ORDER BY bm25(node_search_fts, 8.0, 1.0) ASC, node_search_fts.rowid ASC
            LIMIT ?
        "#
    );

    let mut values: Vec<Value> =
        Vec::with_capacity(kinds.len() + visible_library_ids.map_or(0, |ids| ids.len()) + 2);
    values.push(fts_query.to_owned().into());
    values.extend(kinds.iter().map(|kind| Value::from(kind.to_value())));
    if let Some(library_ids) = visible_library_ids {
        values.extend(library_ids.iter().cloned().map(Value::from));
    }
    values.push(limit.into());

    let statement = Statement::from_sql_and_values(DbBackend::Sqlite, sql, values);
    nodes::Entity::find()
        .from_raw_sql(statement)
        .all(pool)
        .await
}

pub struct Query;

#[Object]
impl Query {
    #[graphql(guard = AuthenticatedGuard::new())]
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
                let auth = ctx.data::<RequestAuth>()?;
                let visible_library_ids = accessible_library_ids(pool, auth)
                    .await
                    .map_err(async_graphql::Error::from)?;
                let mut qb = read::join_preferred_node_metadata(nodes::Entity::find());

                if let Some(library_id) = filter.library_id {
                    if let Some(visible_library_ids) = visible_library_ids.as_ref() {
                        if !visible_library_ids
                            .iter()
                            .any(|visible_id| visible_id == &library_id)
                        {
                            let connection = connection::Connection::new(false, false);
                            return Ok::<_, async_graphql::Error>(connection);
                        }
                    }
                    qb = qb.filter(nodes::Column::LibraryId.eq(library_id));
                } else if let Some(visible_library_ids) = visible_library_ids.as_ref() {
                    if visible_library_ids.is_empty() {
                        let connection = connection::Connection::new(false, false);
                        return Ok::<_, async_graphql::Error>(connection);
                    }

                    qb = qb.filter(nodes::Column::LibraryId.is_in(visible_library_ids.clone()));
                }
                if let Some(root_id) = &filter.root_id {
                    qb = qb.filter(nodes::Column::RootId.eq(root_id.clone()));
                }
                if let Some(parent_id) = &filter.parent_id {
                    qb = qb.filter(nodes::Column::ParentId.eq(parent_id.clone()));
                }
                if let Some(kinds) = &filter.kinds {
                    qb = qb.filter(nodes::Column::Kind.is_in(kinds.clone()));
                }

                if let Some(watched) = filter.watched {
                    let auth = ctx.data::<RequestAuth>()?;
                    let user = auth.get_user_or_err()?;
                    let watched_node_ids = watch_progress::Entity::find()
                        .filter(watch_progress::Column::UserId.eq(user.id.clone()))
                        .filter(
                            watch_progress::Column::ProgressPercent
                                .gt(watch_progress::completed_progress_threshold()),
                        )
                        .select_only()
                        .column(watch_progress::Column::NodeId)
                        .into_tuple::<String>()
                        .all(pool)
                        .await?;

                    if watched {
                        if watched_node_ids.is_empty() {
                            qb = qb.filter(nodes::Column::Id.eq("__never__"));
                        } else {
                            qb = qb.filter(nodes::Column::Id.is_in(watched_node_ids));
                        }
                    } else if !watched_node_ids.is_empty() {
                        qb = qb.filter(nodes::Column::Id.is_not_in(watched_node_ids));
                    }
                }

                let order_by = filter.order_by.unwrap_or(OrderBy::Order);
                let order_direction = filter
                    .order_direction
                    .unwrap_or_else(|| order_by.default_direction())
                    .to_sea_orm();

                match order_by {
                    OrderBy::AddedAt => {
                        // group by discovery day first so recent imports stay together, then
                        // use release date to make same-day batches feel intentional.
                        qb = qb
                            .order_by(
                                Expr::col(nodes::Column::LastAddedAt).div(SECONDS_PER_DAY),
                                order_direction.clone(),
                            )
                            .order_by(node_metadata::Column::ReleasedAt, order_direction)
                    }
                    OrderBy::ReleasedAt => {
                        qb = qb.order_by(node_metadata::Column::ReleasedAt, order_direction)
                    }
                    OrderBy::Alphabetical => {
                        qb = qb.order_by(node_metadata::Column::Name, order_direction)
                    }
                    OrderBy::Rating => {
                        qb = qb.order_by(node_metadata::Column::ScoreNormalized, order_direction)
                    }
                    OrderBy::Order => qb = qb.order_by(nodes::Column::Order, order_direction),
                }

                qb = qb.order_by_asc(nodes::Column::Id);

                let count = qb.clone().count(pool).await?;
                let limit = first.unwrap_or(50) as u64;
                let offset = after.map(|cursor| cursor + 1).unwrap_or(0);
                let records = qb.limit(Some(limit)).offset(Some(offset)).all(pool).await?;

                let has_previous_page = offset > 0;
                let has_next_page = offset + limit < count;
                let mut connection = connection::Connection::new(has_previous_page, has_next_page);
                connection.edges.extend(
                    records
                        .into_iter()
                        .enumerate()
                        .map(|(index, node)| connection::Edge::new(offset + index as u64, node)),
                );

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    #[graphql(guard = AuthenticatedGuard::new())]
    async fn node(
        &self,
        ctx: &Context<'_>,
        node_id: String,
    ) -> Result<nodes::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let mut query = nodes::Entity::find().filter(nodes::Column::Id.eq(node_id));
        if let Some(visible_library_ids) = accessible_library_ids(pool, auth)
            .await
            .map_err(async_graphql::Error::from)?
        {
            if visible_library_ids.is_empty() {
                return Err(async_graphql::Error::new("Node not found"));
            }

            query = query.filter(nodes::Column::LibraryId.is_in(visible_library_ids));
        }

        query
            .one(pool)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Node not found"))
    }

    #[graphql(guard = AuthenticatedGuard::new())]
    async fn search(
        &self,
        ctx: &Context<'_>,
        query: String,
        limit: Option<i32>,
    ) -> Result<SearchResults, async_graphql::Error> {
        let Some(fts_query) = build_fts_query(&query) else {
            return Ok(SearchResults {
                roots: Vec::new(),
                episodes: Vec::new(),
            });
        };

        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let limit = limit.unwrap_or(10).clamp(1, 20) as i64;
        let visible_library_ids = accessible_library_ids(pool, auth)
            .await
            .map_err(async_graphql::Error::from)?;

        let roots = search_nodes_by_kinds(
            pool,
            &fts_query,
            limit,
            &[nodes::NodeKind::Movie, nodes::NodeKind::Series],
            visible_library_ids.as_deref(),
        )
        .await?;
        let episodes = search_nodes_by_kinds(
            pool,
            &fts_query,
            limit,
            &[nodes::NodeKind::Episode],
            visible_library_ids.as_deref(),
        )
        .await?;

        Ok(SearchResults { roots, episodes })
    }

    #[graphql(guard = PermissionGuard::new(users::UserPerms::ADMIN))]
    async fn list_files(&self, path: String) -> Result<Vec<String>, async_graphql::Error> {
        if !path.starts_with('/') || path.contains("..") || path.contains("/.") {
            return Err(async_graphql::Error::new("Invalid path"));
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

    #[graphql(guard = AuthenticatedGuard::new())]
    async fn library(
        &self,
        ctx: &Context<'_>,
        library_id: String,
    ) -> Result<libraries::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let mut query = libraries::Entity::find().filter(libraries::Column::Id.eq(library_id));
        if let Some(visible_library_ids) = accessible_library_ids(pool, auth)
            .await
            .map_err(async_graphql::Error::from)?
        {
            if visible_library_ids.is_empty() {
                return Err(async_graphql::Error::new("Library not found"));
            }

            query = query.filter(libraries::Column::Id.is_in(visible_library_ids));
        }

        query
            .one(pool)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Library not found"))
    }

    #[graphql(guard = AuthenticatedGuard::new())]
    async fn libraries(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<libraries::Model>, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let mut query = libraries::Entity::find();
        if let Some(visible_library_ids) = accessible_library_ids(pool, auth)
            .await
            .map_err(async_graphql::Error::from)?
        {
            if visible_library_ids.is_empty() {
                return Ok(Vec::new());
            }

            query = query.filter(libraries::Column::Id.is_in(visible_library_ids));
        }

        Ok(query.all(pool).await?)
    }

    #[graphql(guard = AuthenticatedGuard::new())]
    async fn viewer(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<users::Model>, async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        Ok(ctx
            .data_opt::<RequestAuth>()
            .and_then(|_| auth.get_user().cloned()))
    }

    #[graphql(guard = AuthenticatedGuard::new())]
    async fn watch_session(
        &self,
        ctx: &Context<'_>,
        session_id: String,
    ) -> Result<Option<crate::watch_session::WatchSession>, async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        let registry = ctx.data::<WatchSessionRegistry>()?;
        registry.session_for_view(auth, &session_id).await
    }

    #[graphql(guard = PermissionGuard::new(users::UserPerms::ADMIN))]
    async fn watch_sessions(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<crate::watch_session::WatchSession>, async_graphql::Error> {
        let registry = ctx.data::<WatchSessionRegistry>()?;
        Ok(registry.sessions_snapshot().await)
    }

    #[graphql(guard = PermissionGuard::new(users::UserPerms::ADMIN))]
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<users::Model>, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        Ok(users::Entity::find()
            .order_by_asc(users::Column::CreatedAt)
            .all(pool)
            .await?)
    }

    #[graphql(guard = PermissionGuard::new(users::UserPerms::ADMIN))]
    async fn activities(&self, _ctx: &Context<'_>) -> Result<Vec<Activity>, async_graphql::Error> {
        Ok(ACTIVITY_REGISTRY
            .snapshot()
            .into_iter()
            .map(|activity| Activity {
                task_type: activity.task_type,
                title: activity.title,
                current: activity.current,
                total: activity.total,
                progress_percent: activity.progress_percent,
            })
            .collect())
    }
}
