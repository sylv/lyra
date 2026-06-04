use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "images")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub kind: ImageKind,
    pub source: ImageSource,
    #[sea_orm(column_type = "Text", nullable)]
    pub source_url: Option<String>,
    pub size_bytes: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub mime_type: Option<String>,
    pub height: Option<i64>,
    pub width: Option<i64>,
    #[sea_orm(column_type = "Blob", nullable)]
    pub thumbhash: Option<Vec<u8>>,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::file_images::Entity")]
    FileImages,
    #[sea_orm(has_many = "super::node_images::Entity")]
    NodeImages,
}

impl Related<super::file_images::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FileImages.def()
    }
}

impl Related<super::node_images::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NodeImages.def()
    }
}

impl Related<super::files::Entity> for Entity {
    fn to() -> RelationDef {
        super::file_images::Relation::Files.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::file_images::Relation::Images.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum ImageKind {
    Poster = 0,
    Thumbnail = 1,
    Backdrop = 2,
    TimelinePreviewSheet = 3,
    Logo = 5,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum ImageSource {
    Local = 0,
    Remote = 1,
}
