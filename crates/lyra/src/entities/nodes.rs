use async_graphql::{Enum, SimpleObject};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Enum, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum NodeKind {
    Movie = 0,
    Series = 1,
    Season = 2,
    Episode = 3,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, SimpleObject)]
#[sea_orm(table_name = "nodes")]
#[graphql(name = "Node", complex)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub root_id: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub parent_id: Option<String>,
    pub library_id: i64,
    pub file_id: Option<i64>,
    #[sea_orm(column_type = "Text")]
    pub relative_path: String,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    pub kind: NodeKind,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::files::Entity",
        from = "Column::FileId",
        to = "super::files::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Files,
    #[sea_orm(
        belongs_to = "super::libraries::Entity",
        from = "Column::LibraryId",
        to = "super::libraries::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Libraries,
    #[sea_orm(has_many = "super::node_metadata::Entity")]
    NodeMetadata,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Parent,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::RootId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Root,
    #[sea_orm(has_many = "super::watch_progress::Entity")]
    WatchProgress,
}

impl Related<super::files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Files.def()
    }
}

impl Related<super::libraries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Libraries.def()
    }
}

impl Related<super::node_metadata::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NodeMetadata.def()
    }
}

impl Related<super::watch_progress::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WatchProgress.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
