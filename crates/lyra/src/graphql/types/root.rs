use super::{current_user_id, saturating_i32_from_u64};
use crate::auth::RequestAuth;
use crate::entities::{items, root_metadata, roots, seasons, watch_progress};
use crate::graphql::properties::RootNodeProperties;
use async_graphql::{ComplexObject, Context, Union};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    sea_query::{Expr, Query},
};

#[derive(Union)]
pub enum RootChild {
    SeasonNode(seasons::Model),
    ItemNode(items::Model),
}

#[ComplexObject]
impl roots::Model {
    pub async fn properties(
        &self,
        ctx: &Context<'_>,
    ) -> Result<RootNodeProperties, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let metadata = root_metadata::Entity::find()
            .filter(root_metadata::Column::RootId.eq(self.id.clone()))
            .order_by_desc(root_metadata::Column::Source)
            .order_by_desc(root_metadata::Column::UpdatedAt)
            .one(pool)
            .await?;

        Ok(RootNodeProperties::from_metadata(metadata))
    }

    pub async fn seasons(&self, ctx: &Context<'_>) -> Result<Vec<seasons::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        seasons::Entity::find()
            .filter(seasons::Column::RootId.eq(self.id.clone()))
            .order_by_asc(seasons::Column::Order)
            .all(pool)
            .await
    }

    pub async fn files(&self, ctx: &Context<'_>) -> Result<Vec<items::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        items::Entity::find()
            .filter(items::Column::RootId.eq(self.id.clone()))
            .order_by_asc(items::Column::Order)
            .all(pool)
            .await
    }

    pub async fn children(&self, ctx: &Context<'_>) -> Result<Vec<RootChild>, sea_orm::DbErr> {
        let seasons = self.seasons(ctx).await?;
        if !seasons.is_empty() {
            return Ok(seasons
                .into_iter()
                .map(RootChild::SeasonNode)
                .collect::<Vec<_>>());
        }

        let items = self.files(ctx).await?;
        Ok(items
            .into_iter()
            .map(RootChild::ItemNode)
            .collect::<Vec<_>>())
    }

    pub async fn next_item(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<items::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let mut query = items::Entity::find()
            .filter(items::Column::RootId.eq(self.id.clone()))
            .order_by_asc(items::Column::Order)
            .order_by_asc(items::Column::Id);

        if let Some(user_id) = current_user_id(ctx) {
            query = query.filter(
                Expr::col((items::Entity, items::Column::Id)).not_in_subquery(
                    Query::select()
                        .column(watch_progress::Column::ItemId)
                        .from(watch_progress::Entity)
                        .and_where(
                            Expr::col((watch_progress::Entity, watch_progress::Column::UserId))
                                .eq(user_id),
                        )
                        .and_where(
                            Expr::col((
                                watch_progress::Entity,
                                watch_progress::Column::ProgressPercent,
                            ))
                            .gt(watch_progress::completed_progress_threshold()),
                        )
                        .to_owned(),
                ),
            );
        }

        query.one(pool).await
    }

    pub async fn unplayed_items(&self, ctx: &Context<'_>) -> Result<i32, async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;
        let pool = ctx.data_unchecked::<DatabaseConnection>();

        let count = items::Entity::find()
            .filter(items::Column::RootId.eq(self.id.clone()))
            .filter(
                Expr::col((items::Entity, items::Column::Id)).not_in_subquery(
                    Query::select()
                        .column(watch_progress::Column::ItemId)
                        .from(watch_progress::Entity)
                        .and_where(
                            Expr::col((watch_progress::Entity, watch_progress::Column::UserId))
                                .eq(user.id.clone()),
                        )
                        .and_where(
                            Expr::col((
                                watch_progress::Entity,
                                watch_progress::Column::ProgressPercent,
                            ))
                            .gt(watch_progress::completed_progress_threshold()),
                        )
                        .to_owned(),
                ),
            )
            .count(pool)
            .await?;

        Ok(saturating_i32_from_u64(count))
    }

    pub async fn season_count(&self, ctx: &Context<'_>) -> Result<i32, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let count = seasons::Entity::find()
            .filter(seasons::Column::RootId.eq(self.id.clone()))
            .count(pool)
            .await?;
        Ok(saturating_i32_from_u64(count))
    }

    pub async fn episode_count(&self, ctx: &Context<'_>) -> Result<i32, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let count = items::Entity::find()
            .filter(items::Column::RootId.eq(self.id.clone()))
            .filter(items::Column::SeasonId.is_null())
            .count(pool)
            .await?;
        Ok(saturating_i32_from_u64(count))
    }
}
