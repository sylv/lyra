use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "node_metadata_images")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub node_metadata_id: String,
    pub asset_id: String,
    pub kind: NodeMetadataImageKind,
    pub position: i64,
    #[sea_orm(column_type = "Text", nullable)]
    pub language: Option<String>,
    pub vote_average: Option<f64>,
    pub vote_count: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub file_type: Option<String>,
    pub is_active: bool,
    pub created_at: i64,
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
        belongs_to = "super::node_metadata::Entity",
        from = "Column::NodeMetadataId",
        to = "super::node_metadata::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    NodeMetadata,
}

impl Related<super::assets::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Assets.def()
    }
}

impl Related<super::node_metadata::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NodeMetadata.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum NodeMetadataImageKind {
    Poster = 0,
    Thumbnail = 1,
    Backdrop = 2,
    Logo = 3,
}
