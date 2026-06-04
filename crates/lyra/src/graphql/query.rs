use crate::{
    activity::ACTIVITY_REGISTRY,
    auth::{AuthenticatedGuard, PermissionGuard, RequestAuth},
    entities::{libraries, node_metadata, nodes, users, watch_progress},
    metadata,
};
use async_graphql::{
    Context, Enum, InputObject, Object, SimpleObject,
    connection::{self, EmptyFields},
};
use lazy_static::lazy_static;
use regex::Regex;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, ExprTrait, JoinType, Order, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, QueryTrait, RelationTrait,
    prelude::Expr,
    sea_query::{Alias, Func, Query as SeaQuery},
};
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

#[derive(Debug, Clone, Default, InputObject, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeFilter {
    pub library_id: Option<String>,
    pub root_id: Option<String>,
    pub parent_id: Option<String>,
    pub kinds: Option<Vec<nodes::NodeKind>>,
    pub search_term: Option<String>,
    pub availability: Option<NodeAvailability>,
    pub order_by: Option<OrderBy>,
    pub order_direction: Option<OrderDirection>,
    pub season_numbers: Option<Vec<i32>>,
    pub watched: Option<bool>,
    pub continue_watching: Option<bool>,
    pub released_after: Option<i64>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum, serde::Deserialize, serde::Serialize)]
pub enum NodeAvailability {
    Available,
    Unavailable,
    Both,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum, serde::Deserialize, serde::Serialize)]
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum, serde::Deserialize, serde::Serialize)]
pub enum OrderBy {
    AddedAt,
    LastAddedAt,
    FirstAired,
    LastAired,
    ReleasedAt,
    Alphabetical,
    Rating,
    Order,
    WatchProgressUpdatedAt,
}

impl OrderBy {
    pub fn default_direction(self) -> OrderDirection {
        match self {
            OrderBy::AddedAt
            | OrderBy::LastAddedAt
            | OrderBy::FirstAired
            | OrderBy::LastAired
            | OrderBy::ReleasedAt
            | OrderBy::Rating => OrderDirection::Desc,
            OrderBy::Alphabetical | OrderBy::Order => OrderDirection::Asc,
            OrderBy::WatchProgressUpdatedAt => OrderDirection::Desc,
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

pub fn current_user_id(ctx: &Context<'_>) -> Option<String> {
    let auth = ctx.data_opt::<RequestAuth>()?;
    let user = auth.get_user_or_err().ok()?;
    Some(user.id.clone())
}

pub async fn build_node_query(
    pool: &DatabaseConnection,
    auth: &RequestAuth,
    filter: &NodeFilter,
) -> Result<sea_orm::Select<nodes::Entity>, async_graphql::Error> {
    let viewer_id = auth.get_user_or_err()?.id.clone();
    build_node_query_for_viewer(pool, &viewer_id, filter).await
}

pub async fn build_node_query_for_viewer(
    pool: &DatabaseConnection,
    viewer_id: &str,
    filter: &NodeFilter,
) -> Result<sea_orm::Select<nodes::Entity>, async_graphql::Error> {
    let mut qb = metadata::join_preferred_node_metadata(nodes::Entity::find());
    let search_term = filter.search_term.as_deref().map(str::trim);
    let fts_query = search_term
        .filter(|term| !term.is_empty())
        .and_then(build_fts_query);

    if let Some(library_id) = &filter.library_id {
        qb = qb.filter(nodes::Column::LibraryId.eq(library_id.clone()));
    }

    if let Some(root_id) = &filter.root_id {
        qb = qb.filter(nodes::Column::RootId.eq(root_id.clone()));
    }
    if let Some(parent_id) = &filter.parent_id {
        qb = qb.filter(nodes::Column::ParentId.eq(parent_id.clone()));
    }
    if let Some(season_numbers) = &filter.season_numbers {
        if season_numbers.is_empty() {
            return Err(async_graphql::Error::new(
                "seasonNumbers should not be empty",
            ));
        }

        qb = qb.filter(nodes::Column::SeasonNumber.is_in(season_numbers.clone()));
    }
    if let Some(kinds) = &filter.kinds {
        if kinds.is_empty() {
            return Err(async_graphql::Error::new("kinds should not be empty"));
        }

        qb = qb.filter(nodes::Column::Kind.is_in(kinds.clone()));
    }

    if search_term.is_some() && fts_query.is_none() {
        return Err(async_graphql::Error::new(
            "searchTerm must contain at least one searchable term",
        ));
    } else if let Some(fts_query) = fts_query.as_deref() {
        qb = join_node_search(qb, fts_query);
    }

    match filter.availability.unwrap_or(NodeAvailability::Available) {
        NodeAvailability::Available => {
            qb = qb.filter(nodes::Column::UnavailableAt.is_null());
        }
        NodeAvailability::Unavailable => {
            qb = qb.filter(nodes::Column::UnavailableAt.is_not_null());
        }
        NodeAvailability::Both => {}
    }

    if let Some(watched) = filter.watched {
        let watched_node_ids = watch_progress::Entity::find()
            .filter(watch_progress::Column::UserId.eq(viewer_id.to_string()))
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

    if let Some(continue_watching) = filter.continue_watching {
        let continue_watching_node_ids = watch_progress::Entity::find()
            .filter(watch_progress::Column::UserId.eq(viewer_id.to_string()))
            .filter(
                watch_progress::Column::ProgressPercent
                    .gt(watch_progress::minimum_progress_threshold()),
            )
            .filter(
                watch_progress::Column::ProgressPercent
                    .lte(watch_progress::completed_progress_threshold()),
            )
            .select_only()
            .column(watch_progress::Column::NodeId)
            .into_tuple::<String>()
            .all(pool)
            .await?;

        if continue_watching {
            if continue_watching_node_ids.is_empty() {
                qb = qb.filter(nodes::Column::Id.eq("__never__"));
            } else {
                qb = qb.filter(nodes::Column::Id.is_in(continue_watching_node_ids));
            }
        } else if !continue_watching_node_ids.is_empty() {
            qb = qb.filter(nodes::Column::Id.is_not_in(continue_watching_node_ids));
        }
    }

    if let Some(released_after) = filter.released_after {
        qb = qb.filter(
            Expr::expr(Func::coalesce([
                Expr::col(node_metadata::Column::LastAired),
                Expr::col(node_metadata::Column::FirstAired),
            ]))
            .gte(released_after),
        );
    }

    if fts_query.is_some() {
        let search_matches = Alias::new("search_matches");
        qb = qb
            .order_by(
                Expr::col((search_matches.clone(), Alias::new("search_rank"))),
                Order::Asc,
            )
            .order_by(
                Expr::col((search_matches, Alias::new("search_rowid"))),
                Order::Asc,
            );
    } else {
        let order_by = filter.order_by.unwrap_or(OrderBy::Order);
        let order_direction = filter
            .order_direction
            .unwrap_or_else(|| order_by.default_direction())
            .to_sea_orm();

        match order_by {
            OrderBy::AddedAt => {
                qb = qb
                    .order_by(nodes::Column::LastAddedAt, order_direction.clone())
                    .order_by(node_metadata::Column::LastAired, order_direction.clone())
                    .order_by(node_metadata::Column::FirstAired, order_direction)
            }
            OrderBy::LastAddedAt => {
                qb = qb
                    .order_by(nodes::Column::LastAddedAt, order_direction.clone())
                    .order_by(node_metadata::Column::LastAired, order_direction)
            }
            OrderBy::FirstAired => {
                qb = qb.order_by(node_metadata::Column::FirstAired, order_direction)
            }
            OrderBy::LastAired => {
                qb = qb.order_by(node_metadata::Column::LastAired, order_direction)
            }
            OrderBy::ReleasedAt => {
                qb = qb.order_by(
                    Expr::expr(Func::coalesce([
                        Expr::col(node_metadata::Column::LastAired),
                        Expr::col(node_metadata::Column::FirstAired),
                    ])),
                    order_direction,
                )
            }
            OrderBy::Alphabetical => qb = qb.order_by(node_metadata::Column::Name, order_direction),
            OrderBy::Rating => {
                qb = qb.order_by(node_metadata::Column::ScoreNormalized, order_direction)
            }
            OrderBy::Order => qb = qb.order_by(nodes::Column::Order, order_direction),
            OrderBy::WatchProgressUpdatedAt => {
                qb = qb
                    .join(JoinType::InnerJoin, nodes::Relation::WatchProgress.def())
                    .filter(watch_progress::Column::UserId.eq(viewer_id.to_string()))
                    .order_by(watch_progress::Column::UpdatedAt, order_direction)
            }
        }
    }

    qb = qb.order_by_asc(nodes::Column::Id);
    Ok(qb)
}

pub async fn paginate_node_query(
    pool: &DatabaseConnection,
    qb: sea_orm::Select<nodes::Entity>,
    after: Option<String>,
    first: Option<i32>,
) -> Result<connection::Connection<u64, nodes::Model, EmptyFields, EmptyFields>, async_graphql::Error>
{
    connection::query(after, None, first, None, |after, _before, first, _last| {
        let qb = qb.clone();
        async move {
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
        }
    })
    .await
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

// Keep FTS isolated in a subquery so nodeList can reuse the existing node filters,
// availability handling, and pagination without a second bespoke search query path.
fn join_node_search(
    mut qb: sea_orm::Select<nodes::Entity>,
    fts_query: &str,
) -> sea_orm::Select<nodes::Entity> {
    let search_matches = Alias::new("search_matches");
    let search_node_id = Alias::new("node_id");
    let search_rank = Alias::new("search_rank");
    let search_rowid = Alias::new("search_rowid");
    let search_query = SeaQuery::select()
        .expr_as(Expr::col(Alias::new("node_id")), search_node_id.clone())
        .expr_as(
            Expr::cust("bm25(node_search_fts, 8.0, 1.0)"),
            search_rank.clone(),
        )
        .expr_as(Expr::cust("rowid"), search_rowid.clone())
        .from(Alias::new("node_search_fts"))
        .and_where(Expr::cust_with_values(
            "node_search_fts MATCH ?",
            [fts_query.to_owned()],
        ))
        .to_owned();

    QueryTrait::query(&mut qb).join_subquery(
        JoinType::InnerJoin,
        search_query,
        search_matches.clone(),
        Expr::col((nodes::Entity, nodes::Column::Id)).equals((search_matches, search_node_id)),
    );

    qb
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
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let qb = build_node_query(pool, auth, &filter).await?;
        paginate_node_query(pool, qb, after, first).await
    }

    #[graphql(guard = AuthenticatedGuard::new())]
    async fn node(
        &self,
        ctx: &Context<'_>,
        node_id: String,
    ) -> Result<nodes::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        nodes::Entity::find()
            .filter(nodes::Column::Id.eq(node_id))
            .one(pool)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Node not found"))
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
        libraries::Entity::find()
            .filter(libraries::Column::Id.eq(library_id))
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
        Ok(libraries::Entity::find()
            .order_by_desc(libraries::Column::Pinned)
            .order_by_asc(libraries::Column::CreatedAt)
            .all(pool)
            .await?)
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
