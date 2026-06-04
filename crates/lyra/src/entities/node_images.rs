use super::images::{ImageKind, ImageSource};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "node_images")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    #[sea_orm(column_type = "Text")]
    pub node_id: String,
    #[sea_orm(column_type = "Text")]
    pub image_id: String,
    pub source: ImageSource,
    pub kind: ImageKind,
    #[sea_orm(column_type = "Text", nullable)]
    pub language: Option<String>,
    pub vote_average: Option<f64>,
    pub vote_count: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub file_type: Option<String>,
    pub is_active: bool,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::images::Entity",
        from = "Column::ImageId",
        to = "super::images::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Images,
    #[sea_orm(
        belongs_to = "super::nodes::Entity",
        from = "Column::NodeId",
        to = "super::nodes::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Nodes,
}

impl Related<super::images::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Images.def()
    }
}

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nodes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
