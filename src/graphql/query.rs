use crate::{
    auth::RequestAuth,
    entities::{
        media::{self, MediaType},
        watch_state,
    },
};
use async_graphql::{
    Context, Enum, InputObject, Object,
    connection::{self, EmptyFields},
};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, JoinType, Order, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait,
    prelude::Expr,
    sea_query::{Alias, SelectStatement},
};

#[derive(Debug, InputObject, serde::Deserialize)]
pub struct MediaFilter {
    pub parent_id: Option<i64>,
    pub season_numbers: Option<Vec<i64>>,
    pub media_types: Option<Vec<MediaType>>,
    pub search: Option<String>,
    pub order_by: Option<MediaOrderBy>,
    pub order_direction: Option<MediaOrderDirection>,
    pub watched: Option<bool>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum, serde::Deserialize)]
#[graphql(name = "OrderDirection")]
pub enum MediaOrderDirection {
    Asc,
    Desc,
}

impl MediaOrderDirection {
    pub fn to_sea_orm(&self) -> Order {
        match self {
            MediaOrderDirection::Asc => Order::Asc,
            MediaOrderDirection::Desc => Order::Desc,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum, serde::Deserialize)]
#[graphql(name = "MediaOrderBy")]
pub enum MediaOrderBy {
    AddedAt,
    ReleasedAt,
    Alphabetical,
    Rating,
    SeasonEpisode,
}

impl MediaOrderBy {
    pub fn get_default_direction(&self) -> MediaOrderDirection {
        match self {
            MediaOrderBy::AddedAt | MediaOrderBy::ReleasedAt | MediaOrderBy::Rating => {
                MediaOrderDirection::Desc
            }
            MediaOrderBy::Alphabetical | MediaOrderBy::SeasonEpisode => MediaOrderDirection::Asc,
        }
    }
}

pub struct Query;

#[Object]
impl Query {
    async fn media_list(
        &self,
        ctx: &Context<'_>,
        filter: MediaFilter,
        after: Option<String>,
        first: Option<i32>,
    ) -> Result<
        connection::Connection<u64, media::Model, EmptyFields, EmptyFields>,
        async_graphql::Error,
    > {
        connection::query(
            after,
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let mut qb = media::Entity::find();

                if let Some(parent_id) = filter.parent_id {
                    qb = qb.filter(media::Column::ParentId.eq(parent_id));
                }

                if let Some(season_numbers) = filter.season_numbers {
                    qb = qb.filter(media::Column::SeasonNumber.is_in(season_numbers));
                }

                if let Some(media_types) = filter.media_types {
                    qb = qb.filter(media::Column::MediaType.is_in(media_types));
                }

                if let Some(watched) = filter.watched {
                    let auth = ctx.data::<RequestAuth>()?;
                    let user = auth.get_user_or_err()?;

                    qb = qb
                        .join(JoinType::Join, media::Relation::WatchState.def())
                        .filter(watch_state::Column::UserId.eq(user.id.clone()));

                    if watched {
                        qb = qb.filter(watch_state::Column::MediaId.is_not_null());
                    } else {
                        qb = qb.filter(watch_state::Column::MediaId.is_null());
                    }
                }

                if let Some(search) = &filter.search {
                    let sub_alias = Alias::new("search");

                    // this queries the "media_fts5" table using bm25 across both fields
                    // (N, N, N) is (id, title, description) weights respectively
                    // "5" for title and "1" for description means 1 occurence in the title is worth the same as 5 in the description
                    // the "0" for id ignores it in ranking
                    let mut sub_stmt = SelectStatement::new();
                    sub_stmt
                        .column(Alias::new("id"))
                        .expr_as(Expr::cust("bm25(media_fts5, 0, 5, 1)"), Alias::new("rank"))
                        .from(Alias::new("media_fts5"))
                        .and_where(Expr::cust_with_values("media_fts5 MATCH ?", [search]));

                    // this grabs the inner QuerySelect from the qb and
                    // adds the subquery to it. not sure if this is the best way to do this,
                    // but it sure does work.
                    let mut inner = sea_orm::QuerySelect::query(&mut qb);
                    inner = inner.join_subquery(
                        JoinType::InnerJoin,
                        sub_stmt,
                        sub_alias.clone(),
                        Condition::all().add(
                            Expr::col((media::Entity, media::Column::Id))
                                .eq(Expr::col((sub_alias.clone(), Alias::new("id")))),
                        ),
                    );

                    inner.order_by((sub_alias.clone(), Alias::new("rank")), Order::Asc);
                }

                let order_by = filter.order_by.unwrap_or(MediaOrderBy::Alphabetical);
                let order_direction = filter
                    .order_direction
                    .unwrap_or_else(|| order_by.get_default_direction())
                    .to_sea_orm();

                match order_by {
                    MediaOrderBy::AddedAt => {
                        qb = qb.order_by(media::Column::FirstLinkedAt, order_direction);
                    }
                    MediaOrderBy::ReleasedAt => {
                        // todo: for shows, sort by latest episode release date?
                        // or maybe that would make more sense as another order by?
                        qb = qb.order_by(media::Column::StartDate, order_direction);
                    }
                    MediaOrderBy::Alphabetical => {
                        qb = qb.order_by(media::Column::Name, order_direction);
                    }
                    MediaOrderBy::Rating => {
                        qb = qb.order_by(media::Column::Rating, order_direction);
                    }
                    MediaOrderBy::SeasonEpisode => {
                        qb = qb
                            .order_by(media::Column::SeasonNumber, order_direction.clone())
                            .order_by(media::Column::EpisodeNumber, order_direction);
                    }
                };

                let pool = ctx.data::<DatabaseConnection>()?;

                let count = qb.clone().count(pool).await?;

                let limit: u64 = first.unwrap_or(25) as u64;
                let offset: u64 = after.map(|a| a + 1).unwrap_or(0);

                let media = qb
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
                    .extend(media.into_iter().enumerate().map(|(index, media)| {
                        let cursor = (offset + index as u64) as u64;
                        connection::Edge::new(cursor, media)
                    }));

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn media(
        &self,
        ctx: &Context<'_>,
        media_id: i64,
    ) -> Result<media::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let media = media::Entity::find_by_id(media_id)
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("Media not found".to_string()))?;

        Ok(media)
    }
}
