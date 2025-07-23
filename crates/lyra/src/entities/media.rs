use async_graphql::{Enum, SimpleObject};
use sea_orm::entity::prelude::*;
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
    pub created_at: i64,
    pub updated_at: Option<i64>,
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
