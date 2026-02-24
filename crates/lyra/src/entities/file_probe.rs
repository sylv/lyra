use crate::json_encoding;
use anyhow::{Context, Result};
use lyra_ffprobe::FfprobeOutput;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "file_probe")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub file_id: i64,
    pub duration_s: Option<i64>,
    pub height: Option<i64>,
    pub width: Option<i64>,
    pub fps: Option<f64>,
    pub video_codec: Option<String>,
    pub video_bitrate: Option<i64>,
    pub audio_codec: Option<String>,
    pub audio_bitrate: Option<i64>,
    pub audio_channels: Option<i64>,
    pub has_subtitles: bool,
    pub streams: Option<Vec<u8>>,
    pub generated_at: i64,
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
    pub fn decode_ffprobe_output(&self) -> Result<FfprobeOutput> {
        let payload = self
            .streams
            .as_deref()
            .context("file_probe row missing streams payload")?;
        json_encoding::decode_json_zstd::<FfprobeOutput>(payload)
    }
}

impl ActiveModelBehavior for ActiveModel {}
