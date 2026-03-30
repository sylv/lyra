use crate::json_encoding;
use crate::segment_markers::StoredFileSegment;
use async_graphql::SimpleObject;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, SimpleObject)]
#[sea_orm(table_name = "files")]
#[graphql(name = "File", complex)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub library_id: String,
    pub relative_path: String,
    pub size_bytes: i64,
    pub height: Option<i64>,
    pub width: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub edition_name: Option<String>,
    #[graphql(skip)]
    pub audio_fingerprint: Option<Vec<u8>>,
    #[graphql(skip)]
    pub segments_json: Option<Vec<u8>>,
    #[graphql(skip)]
    pub keyframes_json: Option<Vec<u8>>,
    pub unavailable_at: Option<i64>,
    pub scanned_at: Option<i64>,
    pub discovered_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::file_probe::Entity")]
    FileProbe,
    #[sea_orm(
        belongs_to = "super::libraries::Entity",
        from = "Column::LibraryId",
        to = "super::libraries::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Libraries,
    #[sea_orm(has_many = "super::node_files::Entity")]
    NodeFiles,
    #[sea_orm(has_many = "super::watch_progress::Entity")]
    WatchProgress,
}

impl Related<super::file_probe::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FileProbe.def()
    }
}

impl Related<super::libraries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Libraries.def()
    }
}

impl Related<super::node_files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NodeFiles.def()
    }
}

impl Related<super::watch_progress::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WatchProgress.def()
    }
}

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        super::node_files::Relation::Nodes.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::node_files::Relation::Files.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn decode_keyframes(&self) -> anyhow::Result<Vec<i64>> {
        json_encoding::decode_json_zstd(
            self.keyframes_json
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("keyframes payload missing"))?,
        )
    }

    pub fn decode_segments(&self) -> anyhow::Result<Vec<StoredFileSegment>> {
        json_encoding::decode_json_zstd(
            self.segments_json
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("segments payload missing"))?,
        )
    }
}
