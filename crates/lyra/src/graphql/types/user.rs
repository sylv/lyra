use crate::entities::{user_sessions, users};
use async_graphql::{ComplexObject, Context};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

#[ComplexObject]
impl users::Model {
    pub async fn last_seen_at(&self, ctx: &Context<'_>) -> Result<Option<i64>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();

        let session = user_sessions::Entity::find()
            .filter(user_sessions::Column::UserId.eq(&self.id))
            .order_by_desc(user_sessions::Column::LastSeenAt)
            .one(pool)
            .await?;

        Ok(session.map(|s| s.last_seen_at))
    }
}
