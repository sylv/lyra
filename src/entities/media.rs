use async_graphql::{Enum, SimpleObject};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Enum, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum MediaKind {
    Movie = 0,
    Show = 1,
    Episode = 2,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, SimpleObject)]
#[graphql(name = "Media")]
#[sea_orm(table_name = "media")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_type = "Text", nullable, unique)]
    pub imdb_id: Option<String>,
    #[sea_orm(column_type = "Text", nullable, unique)]
    pub tmdb_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub kind: MediaKind,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub rating: Option<f64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub poster_url: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub background_url: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub thumbnail_url: Option<String>,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
    pub runtime_minutes: Option<i64>,
    pub released_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: Option<i64>,
    pub first_linked_at: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::media_connection::Entity")]
    MediaConnection,
    #[sea_orm(has_many = "super::season::Entity")]
    Season,
    #[sea_orm(has_many = "super::watch_state::Entity")]
    WatchState,
}

impl Related<super::media_connection::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MediaConnection.def()
    }
}

impl Related<super::season::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Season.def()
    }
}

impl Related<super::watch_state::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WatchState.def()
    }
}

impl Related<super::file::Entity> for Entity {
    fn to() -> RelationDef {
        super::media_connection::Relation::File.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::media_connection::Relation::Media.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::media_connection::Entity")]
    MediaConnection,
    #[sea_orm(entity = "super::season::Entity")]
    Season,
    #[sea_orm(entity = "super::watch_state::Entity")]
    WatchState,
    #[sea_orm(entity = "super::file::Entity")]
    File,
}
