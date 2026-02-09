use crate::{
    RequestAuth,
    entities::{
        file,
        media::{self, MediaKind},
        media_connection, watch_state,
    },
};
use async_graphql::{ComplexObject, Context};
use sea_orm::{JoinType, QueryOrder, QuerySelect, entity::prelude::*};

#[ComplexObject]
impl media::Model {
    /// Gets the default file connection for this media item, including child connections.
    /// (this is what should be played if the user hits "play" on this media item)
    pub async fn default_connection(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<file::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        if self.kind == MediaKind::Show {
            // essentially we find the first episode, sorted by season/episode number, that has a file connection
            let result = file::Entity::find()
                .join(JoinType::LeftJoin, file::Relation::MediaConnection.def())
                .join(JoinType::LeftJoin, media_connection::Relation::Media.def())
                .filter(media::Column::ParentId.eq(self.id))
                .filter(file::Column::UnavailableAt.is_null())
                .order_by_asc(media::Column::SeasonNumber)
                .order_by_asc(media::Column::EpisodeNumber)
                .limit(1)
                .one(pool)
                .await?;

            Ok(result)
        } else {
            let result = file::Entity::find()
                .join(JoinType::LeftJoin, file::Relation::MediaConnection.def())
                .filter(media_connection::Column::MediaId.eq(self.id))
                .limit(1)
                .one(pool)
                .await?;

            Ok(result)
        }
    }

    pub async fn seasons(&self, ctx: &Context<'_>) -> Result<Vec<i64>, sea_orm::DbErr> {
        match self.kind {
            MediaKind::Show => {
                let pool = ctx.data_unchecked::<DatabaseConnection>();
                let result: Vec<i64> = media::Entity::find()
                    .filter(media::Column::ParentId.eq(self.id))
                    .select_only()
                    .column(media::Column::SeasonNumber)
                    .distinct()
                    .into_tuple()
                    .all(pool)
                    .await?;

                Ok(result)
            }
            _ => Ok(vec![]),
        }
    }

    pub async fn parent(&self, ctx: &Context<'_>) -> Result<Option<media::Model>, sea_orm::DbErr> {
        match self.kind {
            MediaKind::Episode => {
                let pool = ctx.data_unchecked::<DatabaseConnection>();
                let parent = media::Entity::find()
                    .filter(media::Column::Id.eq(self.parent_id))
                    .filter(media::Column::Kind.eq(MediaKind::Show))
                    .one(pool)
                    .await?;

                Ok(parent)
            }
            _ => Ok(None),
        }
    }

    pub async fn watch_state(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<watch_state::Model>, async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;

        match self.kind {
            MediaKind::Episode | MediaKind::Movie => {
                let pool = ctx.data_unchecked::<DatabaseConnection>();
                let result = watch_state::Entity::find()
                    .filter(watch_state::Column::MediaId.eq(self.id))
                    .filter(watch_state::Column::UserId.eq(user.id.clone()))
                    .one(pool)
                    .await?;

                Ok(result)
            }
            _ => Ok(None),
        }
    }
}
