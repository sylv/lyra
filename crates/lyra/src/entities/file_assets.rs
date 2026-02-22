use async_graphql::Enum;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "file_assets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub file_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub asset_id: i64,
    pub role: FileAssetRole,
    pub chapter_number: Option<i64>,
    pub position_ms: Option<i64>,
    pub end_ms: Option<i64>,
    pub sheet_frame_height: Option<i64>,
    pub sheet_frame_width: Option<i64>,
    pub sheet_gap_size: Option<i64>,
    pub sheet_interval: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(
    Debug, Enum, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum FileAssetRole {
    TimelinePreviewSheet = 0,
    Thumbnail = 1,
}
