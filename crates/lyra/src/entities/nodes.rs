use async_graphql::{Enum, SimpleObject};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, SimpleObject)]
#[sea_orm(table_name = "nodes")]
#[graphql(name = "Node", complex)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub library_id: String,
    pub root_id: String,
    pub parent_id: Option<String>,
    pub kind: NodeKind,
    #[graphql(skip)]
    pub name: String,
    pub order: i64,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
    pub last_added_at: i64,
    pub last_fingerprint_version: Option<i64>,
    pub unavailable_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::libraries::Entity",
        from = "Column::LibraryId",
        to = "super::libraries::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Libraries,
    #[sea_orm(has_many = "super::node_files::Entity")]
    NodeFiles,
    #[sea_orm(has_many = "super::node_metadata::Entity")]
    NodeMetadata,
    #[sea_orm(has_many = "super::collection_items::Entity")]
    CollectionItems,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    SelfRef2,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::RootId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    SelfRef1,
    #[sea_orm(has_many = "super::watch_progress::Entity")]
    WatchProgress,
    #[sea_orm(has_many = "super::root_node_cast::Entity")]
    RootNodeCast,
}

impl Related<super::libraries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Libraries.def()
    }
}

impl Related<super::node_files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NodeFiles.def()
    }
}

impl Related<super::node_metadata::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NodeMetadata.def()
    }
}

impl Related<super::collection_items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CollectionItems.def()
    }
}

impl Related<super::watch_progress::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WatchProgress.def()
    }
}

impl Related<super::root_node_cast::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RootNodeCast.def()
    }
}

impl Related<super::files::Entity> for Entity {
    fn to() -> RelationDef {
        super::node_files::Relation::Files.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::node_files::Relation::Nodes.def().rev())
    }
}

impl Related<super::collections::Entity> for Entity {
    fn to() -> RelationDef {
        super::collection_items::Relation::Collections.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::collection_items::Relation::Nodes.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(
    Debug,
    Enum,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    EnumIter,
    DeriveActiveEnum,
)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum NodeKind {
    Movie = 0,
    Series = 1,
    Season = 2,
    Episode = 3,
}
