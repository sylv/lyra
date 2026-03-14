use super::{current_user_id, saturating_i32_from_u64};
use crate::auth::RequestAuth;
use crate::entities::{items, season_metadata, seasons, watch_progress};
use crate::graphql::properties::SeasonNodeProperties;
use async_graphql::{ComplexObject, Context};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    sea_query::{Expr, Query},
};

#[ComplexObject]
impl seasons::Model {
    pub async fn properties(
        &self,
        ctx: &Context<'_>,
    ) -> Result<SeasonNodeProperties, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let metadata = season_metadata::Entity::find()
            .filter(season_metadata::Column::SeasonId.eq(self.id.clone()))
            .order_by_desc(season_metadata::Column::Source)
            .order_by_desc(season_metadata::Column::UpdatedAt)
            .one(pool)
            .await?;

        Ok(SeasonNodeProperties::from_metadata(
            metadata,
            Some(self.season_number),
        ))
    }

    pub async fn files(&self, ctx: &Context<'_>) -> Result<Vec<items::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        items::Entity::find()
            .filter(items::Column::SeasonId.eq(self.id.clone()))
            .order_by_asc(items::Column::Order)
            .all(pool)
            .await
    }

    pub async fn next_item(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<items::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let fallback_query = items::Entity::find()
            .filter(items::Column::SeasonId.eq(self.id.clone()))
            .order_by_asc(items::Column::Order)
            .order_by_asc(items::Column::Id);

        if let Some(user_id) = current_user_id(ctx) {
            let next_item = fallback_query
                .clone()
                .filter(
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
                )
                .one(pool)
                .await?;

            if next_item.is_some() {
                return Ok(next_item);
            }
        }

        fallback_query.one(pool).await
    }

    pub async fn unplayed_items(&self, ctx: &Context<'_>) -> Result<i32, async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;
        let pool = ctx.data_unchecked::<DatabaseConnection>();

        let count = items::Entity::find()
            .filter(items::Column::SeasonId.eq(self.id.clone()))
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

    pub async fn episode_count(&self, ctx: &Context<'_>) -> Result<i32, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let count = items::Entity::find()
            .filter(items::Column::SeasonId.eq(self.id.clone()))
            .count(pool)
            .await?;
        Ok(saturating_i32_from_u64(count))
    }
}
