use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "jobs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub job_kind: i64,
    #[sea_orm(column_type = "Text", unique)]
    pub subject_key: String,
    pub version_key: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub file_id: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub asset_id: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub node_id: Option<String>,
    pub run_after: Option<i64>,
    pub last_run_at: i64,
    #[sea_orm(column_type = "Text", nullable)]
    pub last_error_message: Option<String>,
    pub attempt_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::assets::Entity",
        from = "Column::AssetId",
        to = "super::assets::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Assets,
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
}

impl Related<super::assets::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Assets.def()
    }
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

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Hash)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum JobKind {
    #[sea_orm(num_value = 0)]
    FileGenerateTimelinePreview,
    #[sea_orm(num_value = 1)]
    FileGenerateThumbnail,
    #[sea_orm(num_value = 2)]
    FileExtractFfprobe,
    #[sea_orm(num_value = 3)]
    FileExtractKeyframes,
    #[sea_orm(num_value = 4)]
    AssetDownload,
    #[sea_orm(num_value = 5)]
    AssetGenerateThumbhash,
    #[sea_orm(num_value = 6)]
    NodeGenerateIntroSegments,
    #[sea_orm(num_value = 7)]
    NodeMatchMetadataRoot,
    #[sea_orm(num_value = 8)]
    NodeMatchMetadataGroups,
}

impl JobKind {
    pub const fn code(self) -> i64 {
        match self {
            JobKind::FileGenerateTimelinePreview => 0,
            JobKind::FileGenerateThumbnail => 1,
            JobKind::FileExtractFfprobe => 2,
            JobKind::FileExtractKeyframes => 3,
            JobKind::AssetDownload => 4,
            JobKind::AssetGenerateThumbhash => 5,
            JobKind::NodeGenerateIntroSegments => 6,
            JobKind::NodeMatchMetadataRoot => 7,
            JobKind::NodeMatchMetadataGroups => 8,
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            JobKind::FileGenerateTimelinePreview => "Timeline Preview Generation",
            JobKind::FileGenerateThumbnail => "Thumbnail Generation",
            JobKind::FileExtractFfprobe => "Probe Files",
            JobKind::FileExtractKeyframes => "Keyframe Extraction",
            JobKind::AssetDownload => "Asset Download",
            JobKind::AssetGenerateThumbhash => "Asset Preview Generation",
            JobKind::NodeGenerateIntroSegments => "Intro Detection",
            JobKind::NodeMatchMetadataRoot => "Match Root Metadata",
            JobKind::NodeMatchMetadataGroups => "Match Grouped Node Metadata",
        }
    }

    pub const fn subject_segment(self) -> &'static str {
        match self {
            JobKind::FileGenerateTimelinePreview => "timeline_preview",
            JobKind::FileGenerateThumbnail => "thumbnail",
            JobKind::FileExtractFfprobe => "ffprobe",
            JobKind::FileExtractKeyframes => "keyframes",
            JobKind::AssetDownload => "download",
            JobKind::AssetGenerateThumbhash => "thumbhash",
            JobKind::NodeGenerateIntroSegments => "intro_segments",
            JobKind::NodeMatchMetadataRoot => "metadata_match_root",
            JobKind::NodeMatchMetadataGroups => "metadata_match_groups",
        }
    }
}
