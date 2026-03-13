use crate::auth::RequestAuth;
use crate::entities::{
    file_probe, files, item_files, item_metadata, items, roots, seasons, watch_progress,
};
use crate::graphql::properties::ItemNodeProperties;
use async_graphql::{ComplexObject, Context};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait,
    sea_query::{Cond, Expr, Query},
};
use std::cmp::Ordering;

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

async fn find_adjacent_item(
    pool: &DatabaseConnection,
    current_item: &items::Model,
    direction: Ordering,
) -> Result<Option<items::Model>, sea_orm::DbErr> {
    // find the nearest item in the same root before/after the current one,
    // but skip any item that points at one of the same files. This matters for
    // multi-episode files, where multiple item rows can map to the same media
    // file and would otherwise navigate to the same playback target.
    let order_condition = match direction {
        Ordering::Less => Cond::any()
            .add(items::Column::Order.lt(current_item.order))
            .add(
                Cond::all()
                    .add(items::Column::Order.eq(current_item.order))
                    .add(items::Column::Id.lt(current_item.id.clone())),
            ),
        Ordering::Greater => Cond::any()
            .add(items::Column::Order.gt(current_item.order))
            .add(
                Cond::all()
                    .add(items::Column::Order.eq(current_item.order))
                    .add(items::Column::Id.gt(current_item.id.clone())),
            ),
        Ordering::Equal => unreachable!(),
    };

    let mut query = items::Entity::find()
        .filter(items::Column::RootId.eq(current_item.root_id.clone()))
        .filter(order_condition)
        .filter(
            Expr::col((items::Entity, items::Column::Id)).in_subquery(
                Query::select()
                    .column(item_files::Column::ItemId)
                    .from(item_files::Entity)
                    .to_owned(),
            ),
        )
        .filter(
            Expr::col((items::Entity, items::Column::Id)).not_in_subquery(
                Query::select()
                    .column(item_files::Column::ItemId)
                    .from(item_files::Entity)
                    .and_where(
                        Expr::col((item_files::Entity, item_files::Column::FileId)).in_subquery(
                            Query::select()
                                .column(item_files::Column::FileId)
                                .from(item_files::Entity)
                                .and_where(
                                    Expr::col((item_files::Entity, item_files::Column::ItemId))
                                        .eq(current_item.id.clone()),
                                )
                                .to_owned(),
                        ),
                    )
                    .to_owned(),
            ),
        );

    match direction {
        Ordering::Less => {
            query = query
                .order_by_desc(items::Column::Order)
                .order_by_desc(items::Column::Id);
        }
        Ordering::Greater => {
            query = query
                .order_by_asc(items::Column::Order)
                .order_by_asc(items::Column::Id);
        }
        Ordering::Equal => unreachable!(),
    }

    query.one(pool).await
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

        Ok(progress)
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
        find_adjacent_item(pool, self, Ordering::Less).await
    }

    pub async fn next_item(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<items::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_adjacent_item(pool, self, Ordering::Greater).await
    }
}
