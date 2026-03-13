use async_graphql::{Enum, SimpleObject};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Enum, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum ItemKind {
    Movie = 0,
    Episode = 1,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, SimpleObject)]
#[sea_orm(table_name = "items")]
#[graphql(name = "ItemNode", complex)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    #[sea_orm(column_type = "Text")]
    pub root_id: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub season_id: Option<String>,
    pub kind: ItemKind,
    pub episode_number: Option<i64>,
    pub order: i64,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    pub last_added_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::roots::Entity",
        from = "Column::RootId",
        to = "super::roots::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Roots,
    #[sea_orm(
        belongs_to = "super::seasons::Entity",
        from = "Column::SeasonId",
        to = "super::seasons::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Seasons,
    #[sea_orm(has_many = "super::item_files::Entity")]
    ItemFiles,
    #[sea_orm(has_many = "super::item_metadata::Entity")]
    ItemMetadata,
    #[sea_orm(has_many = "super::watch_progress::Entity")]
    WatchProgress,
}

impl Related<super::roots::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Roots.def()
    }
}

impl Related<super::seasons::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Seasons.def()
    }
}

impl Related<super::item_files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ItemFiles.def()
    }
}

impl Related<super::item_metadata::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ItemMetadata.def()
    }
}

impl Related<super::watch_progress::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WatchProgress.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
