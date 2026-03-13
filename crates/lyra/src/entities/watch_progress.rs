use crate::config::get_config;
use async_graphql::{ComplexObject, SimpleObject};
use sea_orm::entity::prelude::*;

pub fn minimum_progress_threshold() -> f32 {
    get_config().watch_progress_minimum_threshold
}

pub fn completed_progress_threshold() -> f32 {
    get_config().watch_progress_completed_threshold
}

pub fn normalize_progress_percent(progress_percent: f32) -> f32 {
    progress_percent.clamp(0.0, 1.0)
}

pub fn is_completed_progress(progress_percent: f32) -> bool {
    normalize_progress_percent(progress_percent) > completed_progress_threshold()
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
    #[graphql(skip)]
    pub progress_percent: f32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[ComplexObject]
impl Model {
    async fn progress_percent(&self) -> f32 {
        normalize_progress_percent(self.progress_percent)
    }

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

#[cfg(test)]
mod tests {
    use super::normalize_progress_percent;

    #[test]
    fn normalizes_threshold_edges() {
        assert_eq!(normalize_progress_percent(-0.5), 0.0);
        assert_eq!(normalize_progress_percent(0.03), 0.03);
        assert_eq!(normalize_progress_percent(0.05), 0.05);
        assert_eq!(normalize_progress_percent(0.2), 0.2);
        assert_eq!(normalize_progress_percent(0.8), 0.8);
        assert_eq!(normalize_progress_percent(0.95), 0.95);
        assert_eq!(normalize_progress_percent(1.5), 1.0);
    }
}
