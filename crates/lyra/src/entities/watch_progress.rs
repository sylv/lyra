use async_graphql::{ComplexObject, SimpleObject};
use sea_orm::entity::prelude::*;

pub const DEFAULT_MINIMUM_PROGRESS_THRESHOLD: f32 = 0.05;
pub const DEFAULT_COMPLETED_PROGRESS_THRESHOLD: f32 = 0.8;

pub fn normalize_progress_percent(progress_percent: f32) -> f32 {
    let clamped = progress_percent.clamp(0.0, 1.0);

    if clamped <= DEFAULT_MINIMUM_PROGRESS_THRESHOLD {
        0.0
    } else if clamped >= DEFAULT_COMPLETED_PROGRESS_THRESHOLD {
        1.0
    } else {
        clamped
    }
}

pub fn is_completed_progress(progress_percent: f32) -> bool {
    normalize_progress_percent(progress_percent) >= 1.0
}

pub fn is_in_progress(progress_percent: f32) -> bool {
    let normalized = normalize_progress_percent(progress_percent);
    normalized > 0.0 && normalized < 1.0
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
        assert_eq!(normalize_progress_percent(0.03), 0.0);
        assert_eq!(normalize_progress_percent(0.05), 0.0);
        assert_eq!(normalize_progress_percent(0.2), 0.2);
        assert_eq!(normalize_progress_percent(0.8), 1.0);
        assert_eq!(normalize_progress_percent(0.95), 1.0);
        assert_eq!(normalize_progress_percent(1.5), 1.0);
    }
}
