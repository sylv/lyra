use crate::json_encoding;
use lyra_ffprobe::FfprobeOutput;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "file_probe")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub file_id: String,
    pub duration_s: Option<i64>,
    pub height: Option<i64>,
    pub width: Option<i64>,
    pub fps: Option<f64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub video_codec: Option<String>,
    pub video_bitrate: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub audio_codec: Option<String>,
    pub audio_bitrate: Option<i64>,
    pub audio_channels: Option<i64>,
    pub has_subtitles: i64,
    #[sea_orm(column_type = "Blob", nullable)]
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

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn decode_ffprobe_output(&self) -> anyhow::Result<FfprobeOutput> {
        let streams = self
            .streams
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("ffprobe payload missing"))?;
        json_encoding::decode_json_zstd(streams)
    }
}
