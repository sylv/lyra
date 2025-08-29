use sea_orm::entity::prelude::*;
use async_graphql::SimpleObject;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, SimpleObject)]
#[sea_orm(table_name = "file")]
#[graphql(name = "File")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub library_id: i64,
    #[sea_orm(column_type = "Text")]
    pub relative_path: String,
    pub pending_auto_match: i64,
    #[sea_orm(column_type = "Text", nullable)]
    pub edition_name: Option<String>,
    pub resolution: Option<i64>,
    pub size_bytes: Option<i64>,
    pub scanned_at: i64,
    pub unavailable_at: Option<i64>,
    pub corrupted_at: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::library::Entity",
        from = "Column::LibraryId",
        to = "super::library::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Library,
    #[sea_orm(has_many = "super::media_connection::Entity")]
    MediaConnection,
}

impl Related<super::library::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Library.def()
    }
}

impl Related<super::media_connection::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MediaConnection.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::library::Entity")]
    Library,
    #[sea_orm(entity = "super::media_connection::Entity")]
    MediaConnection,
}
