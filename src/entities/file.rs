use async_graphql::SimpleObject;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, SimpleObject)]
#[sea_orm(table_name = "file")]
#[graphql(name = "File")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub backend_name: String,
    pub key: String,
    pub pending_auto_match: i64,
    pub unavailable_since: Option<i64>,
    pub edition_name: Option<String>,
    pub resolution: Option<i64>,
    pub size_bytes: Option<i64>,
    pub scanned_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::media_connection::Entity")]
    MediaConnection,
}

impl Related<super::media_connection::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MediaConnection.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
