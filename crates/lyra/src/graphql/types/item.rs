use crate::auth::RequestAuth;
use crate::entities::{
    file_probe, files, item_files, item_metadata, items, roots, seasons, watch_progress,
};
use crate::graphql::properties::ItemNodeProperties;
use async_graphql::{ComplexObject, Context};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, FromQueryResult, JoinType,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait,
    sea_query::{Alias, Cond, Expr, JoinType as SeaJoinType, Order, Query},
};

async fn find_primary_available_file(
    pool: &DatabaseConnection,
    item_id: &str,
) -> Result<Option<files::Model>, sea_orm::DbErr> {
    files::Entity::find()
        .join(JoinType::InnerJoin, files::Relation::ItemFiles.def())
        .filter(item_files::Column::ItemId.eq(item_id))
        .filter(files::Column::UnavailableAt.is_null())
        .order_by_asc(item_files::Column::Order)
        .order_by_asc(item_files::Column::FileId)
        .one(pool)
        .await
}

#[ComplexObject]
impl items::Model {
    pub async fn properties(
        &self,
        ctx: &Context<'_>,
    ) -> Result<ItemNodeProperties, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();

        let metadata = item_metadata::Entity::find()
            .filter(item_metadata::Column::ItemId.eq(self.id.clone()))
            .order_by_desc(item_metadata::Column::Source)
            .order_by_desc(item_metadata::Column::UpdatedAt)
            .one(pool)
            .await?;

        let season_number = if let Some(season_id) = &self.season_id {
            seasons::Entity::find_by_id(season_id.clone())
                .one(pool)
                .await?
                .map(|season| season.season_number)
        } else {
            None
        };

        let default_file = find_primary_available_file(pool, &self.id).await?;

        let probe = if let Some(file) = default_file.as_ref() {
            file_probe::Entity::find_by_id(file.id).one(pool).await?
        } else {
            None
        };

        Ok(ItemNodeProperties::from_metadata(
            metadata,
            self.id.clone(),
            season_number,
            self.episode_number,
            default_file,
            probe,
        ))
    }

    pub async fn watch_progress(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<watch_progress::Model>, async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;
        let pool = ctx.data_unchecked::<DatabaseConnection>();

        let progress = watch_progress::Entity::find()
            .filter(watch_progress::Column::UserId.eq(user.id.clone()))
            .filter(watch_progress::Column::ItemId.eq(self.id.clone()))
            .one(pool)
            .await?;

        Ok(progress.filter(|progress| watch_progress::is_in_progress(progress.progress_percent)))
    }

    pub async fn file(&self, ctx: &Context<'_>) -> Result<Option<files::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_primary_available_file(pool, &self.id).await
    }

    pub async fn parent(&self, ctx: &Context<'_>) -> Result<Option<roots::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        roots::Entity::find_by_id(self.root_id.clone())
            .one(pool)
            .await
    }

    pub async fn previous_item(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<items::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let candidate = Alias::new("candidate");
        let candidate_files = Alias::new("candidate_files");
        let current_files = Alias::new("current_files");

        let mut query = Query::select();
        query
            .column((candidate.clone(), items::Column::Id))
            .column((candidate.clone(), items::Column::RootId))
            .column((candidate.clone(), items::Column::SeasonId))
            .column((candidate.clone(), items::Column::Kind))
            .column((candidate.clone(), items::Column::EpisodeNumber))
            .column((candidate.clone(), items::Column::Order))
            .column((candidate.clone(), items::Column::Name))
            .column((candidate.clone(), items::Column::LastAddedAt))
            .column((candidate.clone(), items::Column::CreatedAt))
            .column((candidate.clone(), items::Column::UpdatedAt))
            .from_as(items::Entity, candidate.clone())
            .and_where(
                Expr::col((candidate.clone(), items::Column::RootId)).eq(self.root_id.clone()),
            )
            .cond_where(
                Cond::any()
                    .add(Expr::col((candidate.clone(), items::Column::Order)).lt(self.order))
                    .add(
                        Cond::all()
                            .add(
                                Expr::col((candidate.clone(), items::Column::Order)).eq(self.order),
                            )
                            .add(
                                Expr::col((candidate.clone(), items::Column::Id))
                                    .lt(self.id.clone()),
                            ),
                    ),
            )
            .and_where(Expr::exists(
                Query::select()
                    .expr(Expr::val(1))
                    .from(item_files::Entity)
                    .and_where(
                        Expr::col((item_files::Entity, item_files::Column::ItemId))
                            .equals((candidate.clone(), items::Column::Id)),
                    )
                    .to_owned(),
            ))
            .cond_where(
                Cond::all().not().add(Expr::exists(
                    Query::select()
                        .expr(Expr::val(1))
                        .from_as(item_files::Entity, candidate_files.clone())
                        .join_as(
                            SeaJoinType::InnerJoin,
                            item_files::Entity,
                            current_files.clone(),
                            Expr::col((current_files.clone(), item_files::Column::FileId))
                                .equals((candidate_files.clone(), item_files::Column::FileId)),
                        )
                        .and_where(
                            Expr::col((candidate_files.clone(), item_files::Column::ItemId))
                                .equals((candidate.clone(), items::Column::Id)),
                        )
                        .and_where(
                            Expr::col((current_files.clone(), item_files::Column::ItemId))
                                .eq(self.id.clone()),
                        )
                        .to_owned(),
                )),
            )
            .order_by((candidate.clone(), items::Column::Order), Order::Desc)
            .order_by((candidate.clone(), items::Column::Id), Order::Desc)
            .limit(1);

        let statement = pool.get_database_backend().build(&query);
        items::Model::find_by_statement(statement).one(pool).await
    }

    pub async fn next_item(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<items::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let candidate = Alias::new("candidate");
        let candidate_files = Alias::new("candidate_files");
        let current_files = Alias::new("current_files");

        let mut query = Query::select();
        query
            .column((candidate.clone(), items::Column::Id))
            .column((candidate.clone(), items::Column::RootId))
            .column((candidate.clone(), items::Column::SeasonId))
            .column((candidate.clone(), items::Column::Kind))
            .column((candidate.clone(), items::Column::EpisodeNumber))
            .column((candidate.clone(), items::Column::Order))
            .column((candidate.clone(), items::Column::Name))
            .column((candidate.clone(), items::Column::LastAddedAt))
            .column((candidate.clone(), items::Column::CreatedAt))
            .column((candidate.clone(), items::Column::UpdatedAt))
            .from_as(items::Entity, candidate.clone())
            .and_where(
                Expr::col((candidate.clone(), items::Column::RootId)).eq(self.root_id.clone()),
            )
            .cond_where(
                Cond::any()
                    .add(Expr::col((candidate.clone(), items::Column::Order)).gt(self.order))
                    .add(
                        Cond::all()
                            .add(
                                Expr::col((candidate.clone(), items::Column::Order)).eq(self.order),
                            )
                            .add(
                                Expr::col((candidate.clone(), items::Column::Id))
                                    .gt(self.id.clone()),
                            ),
                    ),
            )
            .and_where(Expr::exists(
                Query::select()
                    .expr(Expr::val(1))
                    .from(item_files::Entity)
                    .and_where(
                        Expr::col((item_files::Entity, item_files::Column::ItemId))
                            .equals((candidate.clone(), items::Column::Id)),
                    )
                    .to_owned(),
            ))
            .cond_where(
                Cond::all().not().add(Expr::exists(
                    Query::select()
                        .expr(Expr::val(1))
                        .from_as(item_files::Entity, candidate_files.clone())
                        .join_as(
                            SeaJoinType::InnerJoin,
                            item_files::Entity,
                            current_files.clone(),
                            Expr::col((current_files.clone(), item_files::Column::FileId))
                                .equals((candidate_files.clone(), item_files::Column::FileId)),
                        )
                        .and_where(
                            Expr::col((candidate_files.clone(), item_files::Column::ItemId))
                                .equals((candidate.clone(), items::Column::Id)),
                        )
                        .and_where(
                            Expr::col((current_files.clone(), item_files::Column::ItemId))
                                .eq(self.id.clone()),
                        )
                        .to_owned(),
                )),
            )
            .order_by((candidate.clone(), items::Column::Order), Order::Asc)
            .order_by((candidate.clone(), items::Column::Id), Order::Asc)
            .limit(1);

        let statement = pool.get_database_backend().build(&query);
        items::Model::find_by_statement(statement).one(pool).await
    }
}
