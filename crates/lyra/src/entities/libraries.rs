use async_graphql::SimpleObject;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, SimpleObject)]
#[sea_orm(table_name = "libraries")]
#[graphql(name = "Library")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub name: String,
    #[sea_orm(column_type = "Text", unique)]
    pub path: String,
    pub pinned: bool,
    pub last_scanned_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::files::Entity")]
    Files,
    #[sea_orm(has_many = "super::library_users::Entity")]
    LibraryUsers,
    #[sea_orm(has_many = "super::nodes::Entity")]
    Nodes,
}

impl Related<super::files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Files.def()
    }
}

impl Related<super::library_users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LibraryUsers.def()
    }
}

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nodes.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        super::library_users::Relation::Users.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::library_users::Relation::Libraries.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
