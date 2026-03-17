use crate::config::get_config;
use async_graphql::{ComplexObject, SimpleObject};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, SimpleObject)]
#[sea_orm(table_name = "watch_progress")]
#[graphql(name = "WatchProgress", complex)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub user_id: String,
    pub node_id: String,
    pub file_id: String,
    #[graphql(skip)]
    pub progress_percent: f32,
    pub created_at: i64,
    pub updated_at: i64,
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
        belongs_to = "super::nodes::Entity",
        from = "Column::NodeId",
        to = "super::nodes::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Nodes,
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

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nodes.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

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

#[ComplexObject]
impl Model {
    async fn progress_percent(&self) -> f32 {
        normalize_progress_percent(self.progress_percent)
    }

    async fn completed(&self) -> bool {
        is_completed_progress(self.progress_percent)
    }
}
