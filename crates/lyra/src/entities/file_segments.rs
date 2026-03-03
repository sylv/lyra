use crate::{json_encoding, segment_markers::StoredFileSegment};
use anyhow::Result;
use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum FileSegmentsStatus {
    #[sea_orm(num_value = 0)]
    Ready,
    #[sea_orm(num_value = 1)]
    Error,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "file_segments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub file_id: i64,
    pub segment_list: Vec<u8>,
    pub status: FileSegmentsStatus,
    pub attempts: i64,
    pub last_attempted_at: Option<i64>,
    pub retry_after: Option<i64>,
    pub last_error_message: Option<String>,
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
}

impl Related<super::files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Files.def()
    }
}

impl Model {
    pub fn decode_segments(&self) -> Result<Vec<StoredFileSegment>> {
        json_encoding::decode_json_zstd::<Vec<StoredFileSegment>>(&self.segment_list)
    }
}

impl ActiveModelBehavior for ActiveModel {}
