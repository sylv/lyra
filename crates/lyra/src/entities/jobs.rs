use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "jobs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub job_kind: i64,
    #[sea_orm(column_type = "Text")]
    pub target_id: String,
    pub state: i64,
    pub locked_at: Option<i64>,
    pub retry_after: Option<i64>,
    pub last_run_at: i64,
    #[sea_orm(column_type = "Text", nullable)]
    pub last_error_message: Option<String>,
    pub attempt_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Hash)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum JobKind {
    #[sea_orm(num_value = 0)]
    FileGenerateTimelinePreview,
    #[sea_orm(num_value = 1)]
    FileGenerateThumbnail,
    #[sea_orm(num_value = 2)]
    FileProbe,
    #[sea_orm(num_value = 4)]
    AssetDownload,
    #[sea_orm(num_value = 5)]
    AssetGenerateThumbhash,
    #[sea_orm(num_value = 6)]
    NodeGenerateIntroSegments,
    #[sea_orm(num_value = 7)]
    NodeSyncMetadataRoot,
    #[sea_orm(num_value = 8)]
    FileExtractSubtitles,
    #[sea_orm(num_value = 9)]
    FileProcessSubtitle,
    #[sea_orm(num_value = 10)]
    AssetCleanup,
}

impl JobKind {
    pub const fn code(self) -> i64 {
        match self {
            JobKind::FileGenerateTimelinePreview => 0,
            JobKind::FileGenerateThumbnail => 1,
            JobKind::FileProbe => 2,
            JobKind::AssetDownload => 4,
            JobKind::AssetGenerateThumbhash => 5,
            JobKind::NodeGenerateIntroSegments => 6,
            JobKind::NodeSyncMetadataRoot => 7,
            JobKind::FileExtractSubtitles => 8,
            JobKind::FileProcessSubtitle => 9,
            JobKind::AssetCleanup => 10,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum JobState {
    #[sea_orm(num_value = 0)]
    Running,
    #[sea_orm(num_value = 1)]
    Errored,
}
