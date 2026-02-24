use async_graphql::SimpleObject;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, SimpleObject)]
#[sea_orm(table_name = "files")]
#[graphql(name = "File", complex)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub library_id: i64,
    #[sea_orm(column_type = "Text")]
    pub relative_path: String,
    pub size_bytes: i64,
    pub height: Option<i64>,
    pub width: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub edition_name: Option<String>,
    pub unavailable_at: Option<i64>,
    pub corrupted_at: Option<i64>,
    pub scanned_at: Option<i64>,
    pub discovered_at: i64,
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
    #[sea_orm(has_many = "super::item_files::Entity")]
    ItemFiles,
    #[sea_orm(has_many = "super::jobs::Entity")]
    Jobs,
    #[sea_orm(has_many = "super::items::Entity")]
    PrimaryItems,
    #[sea_orm(has_many = "super::watch_progress::Entity")]
    WatchProgress,
}

impl Related<super::libraries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Libraries.def()
    }
}

impl Related<super::item_files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ItemFiles.def()
    }
}

impl Related<super::jobs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Jobs.def()
    }
}

impl Related<super::items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PrimaryItems.def()
    }
}

impl Related<super::watch_progress::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WatchProgress.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
