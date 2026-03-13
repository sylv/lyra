use async_graphql::{Enum, SimpleObject};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Enum, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum RootKind {
    Movie = 0,
    Series = 1,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, SimpleObject)]
#[sea_orm(table_name = "roots")]
#[graphql(name = "RootNode", complex)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub library_id: i64,
    pub kind: RootKind,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[graphql(skip)]
    #[sea_orm(column_type = "Blob", nullable)]
    pub match_candidates_json: Option<Vec<u8>>,
    pub last_added_at: i64,
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
    #[sea_orm(has_many = "super::seasons::Entity")]
    Seasons,
    #[sea_orm(has_many = "super::items::Entity")]
    Items,
    #[sea_orm(has_many = "super::root_metadata::Entity")]
    RootMetadata,
}

impl Related<super::libraries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Libraries.def()
    }
}

impl Related<super::seasons::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Seasons.def()
    }
}

impl Related<super::items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Items.def()
    }
}

impl Related<super::root_metadata::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RootMetadata.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
