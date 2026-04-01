use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "assets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub kind: AssetKind,
    #[sea_orm(column_type = "Text", nullable)]
    pub source_url: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub hash_sha256: Option<String>,
    pub size_bytes: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub mime_type: Option<String>,
    pub height: Option<i64>,
    pub width: Option<i64>,
    #[sea_orm(column_type = "Blob", nullable)]
    pub thumbhash: Option<Vec<u8>>,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::file_assets::Entity")]
    FileAssets,
}

impl Related<super::file_assets::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FileAssets.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum AssetKind {
    Poster = 0,
    Thumbnail = 1,
    Background = 2,
    TimelinePreviewSheet = 3,
}
