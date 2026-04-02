use async_graphql::{Enum, SimpleObject};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, SimpleObject)]
#[sea_orm(table_name = "collections")]
#[graphql(name = "Collection", complex)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_by_id: Option<String>,
    pub visibility: CollectionVisibility,
    pub resolver_kind: CollectionResolverKind,
    #[graphql(skip)]
    pub kind: Option<i64>,
    #[graphql(skip)]
    pub filter_json: Option<Vec<u8>>,
    pub show_on_home: bool,
    pub home_position: i64,
    pub pinned: bool,
    pub pinned_position: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::collection_items::Entity")]
    CollectionItems,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::CreatedById",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Users,
}

impl Related<super::collection_items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CollectionItems.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        super::collection_items::Relation::Nodes.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::collection_items::Relation::Collections.def().rev())
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
pub enum CollectionVisibility {
    Public = 0,
    Private = 1,
}

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
pub enum CollectionResolverKind {
    Manual = 0,
    Filter = 1,
}

#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CollectionKind {
    ContinueWatching = 0,
}

impl CollectionKind {
    pub fn from_db(value: i64) -> Option<Self> {
        match value {
            0 => Some(Self::ContinueWatching),
            _ => None,
        }
    }

    pub fn as_db(self) -> i64 {
        self as i64
    }
}
