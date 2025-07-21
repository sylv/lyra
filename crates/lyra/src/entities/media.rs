use crate::entities::{file, media, media_connection};
use async_graphql::{ComplexObject, Context, Enum, SimpleObject};
use sea_orm::{JoinType, QueryOrder, QuerySelect, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Enum, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum MediaType {
    Movie = 0,
    Show = 1,
    Episode = 2,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, SimpleObject)]
#[sea_orm(table_name = "media")]
#[graphql(complex, name = "Media")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub poster_url: Option<String>,
    pub background_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub parent_id: Option<i64>,
    pub media_type: MediaType,
    pub imdb_parent_id: Option<String>,
    pub imdb_item_id: Option<String>,
    pub tmdb_parent_id: i64,
    pub tmdb_item_id: i64,
    pub rating: Option<f64>,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub runtime_minutes: Option<i64>,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    SelfRef,
    #[sea_orm(has_many = "super::media_connection::Entity")]
    MediaConnection,
}

impl Related<super::media_connection::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MediaConnection.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[ComplexObject]
impl Model {
    /// Gets the default file connection for this media item, including child connections.
    /// (this is what should be played if the user hits "play" on this media item)
    pub async fn default_connection(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<file::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        if self.media_type == MediaType::Show {
            // essentially we find the first episode, sorted by season/episode number, that has a file connection
            let result = file::Entity::find()
                .join(JoinType::LeftJoin, file::Relation::MediaConnection.def())
                .join(JoinType::LeftJoin, media_connection::Relation::Media.def())
                .filter(media::Column::ParentId.eq(self.id))
                .order_by_desc(media::Column::SeasonNumber)
                .order_by_desc(media::Column::EpisodeNumber)
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

    /// Gets file connections that are directly connected to this media item (excluding child connections)
    pub async fn direct_connections(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<file::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let result = file::Entity::find()
            .join(JoinType::LeftJoin, media::Relation::MediaConnection.def())
            .filter(media_connection::Column::MediaId.eq(self.id))
            .all(pool)
            .await?;

        Ok(result)
    }

    pub async fn seasons(&self, ctx: &Context<'_>) -> Result<Vec<i64>, sea_orm::DbErr> {
        match self.media_type {
            MediaType::Show => {
                let pool = ctx.data_unchecked::<DatabaseConnection>();
                // let seasons = sqlx::query_scalar!(
                //     "SELECT DISTINCT season_number FROM media WHERE parent_id = ?",
                //     self.id
                // )
                // .fetch_all(pool)
                // .await?;

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

    pub async fn parent(&self, ctx: &Context<'_>) -> Result<Option<Model>, sea_orm::DbErr> {
        match self.media_type {
            MediaType::Episode => {
                let pool = ctx.data_unchecked::<DatabaseConnection>();
                let parent = media::Entity::find()
                    .filter(media::Column::Id.eq(self.parent_id))
                    .filter(media::Column::MediaType.eq(MediaType::Show))
                    .one(pool)
                    .await?;

                Ok(parent)
            }
            _ => Ok(None),
        }
    }
}
