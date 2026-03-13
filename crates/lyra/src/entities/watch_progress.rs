use async_graphql::{ComplexObject, SimpleObject};
use sea_orm::entity::prelude::*;

pub const DEFAULT_COMPLETED_PROGRESS_THRESHOLD: f32 = 0.8;

pub fn is_completed_progress(progress_percent: f32) -> bool {
    progress_percent >= DEFAULT_COMPLETED_PROGRESS_THRESHOLD
}

pub fn is_in_progress(progress_percent: f32) -> bool {
    progress_percent > 0.05 && !is_completed_progress(progress_percent)
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, SimpleObject)]
#[sea_orm(table_name = "watch_progress")]
#[graphql(name = "WatchProgress", complex)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_type = "Text")]
    pub user_id: String,
    #[sea_orm(column_type = "Text")]
    pub item_id: String,
    pub file_id: i64,
    pub progress_percent: f32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[ComplexObject]
impl Model {
    async fn completed(&self) -> bool {
        is_completed_progress(self.progress_percent)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::files::Entity",
        from = "Column::FileId",
        to = "super::files::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Files,
    #[sea_orm(
        belongs_to = "super::items::Entity",
        from = "Column::ItemId",
        to = "super::items::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Items,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Users,
}

impl Related<super::files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Files.def()
    }
}

impl Related<super::items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Items.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
