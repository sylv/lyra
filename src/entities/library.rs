use async_graphql::SimpleObject;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, SimpleObject)]
#[sea_orm(table_name = "library")]
#[graphql(name = "Library")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[sea_orm(column_type = "Text", unique)]
    pub path: String,
    pub last_scanned_at: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::file::Entity")]
    File,
    #[sea_orm(has_many = "super::library_user::Entity")]
    LibraryUser,
}

impl Related<super::file::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::File.def()
    }
}

impl Related<super::library_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LibraryUser.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        super::library_user::Relation::Users.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::library_user::Relation::Library.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::file::Entity")]
    File,
    #[sea_orm(entity = "super::library_user::Entity")]
    LibraryUser,
    #[sea_orm(entity = "super::users::Entity")]
    Users,
}
