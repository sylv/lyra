use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "file_subtitles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub file_id: String,
    pub asset_id: String,
    pub derived_from_subtitle_id: Option<String>,
    pub kind: SubtitleKind,
    pub stream_index: i64,
    pub source: SubtitleSource,
    #[sea_orm(column_type = "Text", nullable)]
    pub language_bcp47: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub display_name: Option<String>,
    pub disposition_bits: i64,
    pub last_seen_at: i64,
    pub processed_at: Option<i64>,
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
        belongs_to = "super::assets::Entity",
        from = "Column::AssetId",
        to = "super::assets::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Assets,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::DerivedFromSubtitleId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    ParentSubtitle,
}

impl Related<super::files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Files.def()
    }
}

impl Related<super::assets::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Assets.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum SubtitleKind {
    Srt = 0,
    Vtt = 1,
    Ass = 2,
    MovText = 3,
    Text = 4,
    Ttml = 5,
    Pgs = 6,
    VobSub = 7,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum SubtitleSource {
    Extracted = 0,
    Converted = 1,
    Ocr = 2,
    Generated = 3,
}
