use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "file_images")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub file_id: String,
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub image_id: String,
    pub role: FileImageRole,
    pub start_ms: Option<i64>,
    pub end_ms: Option<i64>,
    pub sheet_frame_height: Option<i64>,
    pub sheet_frame_width: Option<i64>,
    pub sheet_gap_size: Option<i64>,
    pub sheet_interval: Option<i64>,
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
        belongs_to = "super::images::Entity",
        from = "Column::ImageId",
        to = "super::images::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Images,
}

impl Related<super::files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Files.def()
    }
}

impl Related<super::images::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Images.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum FileImageRole {
    Thumbnail = 0,
    TimelinePreviewSheet = 1,
}
