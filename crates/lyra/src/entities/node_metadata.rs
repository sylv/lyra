use crate::entities::metadata_source::MetadataSource;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "node_metadata")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub node_id: String,
    pub source: MetadataSource,
    pub provider_id: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<i64>,
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub score_display: Option<String>,
    pub score_normalized: Option<i64>,
    pub released_at: Option<i64>,
    pub ended_at: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub poster_asset_id: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub thumbnail_asset_id: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub background_asset_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::assets::Entity",
        from = "Column::BackgroundAssetId",
        to = "super::assets::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Assets3,
    #[sea_orm(
        belongs_to = "super::assets::Entity",
        from = "Column::ThumbnailAssetId",
        to = "super::assets::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Assets2,
    #[sea_orm(
        belongs_to = "super::assets::Entity",
        from = "Column::PosterAssetId",
        to = "super::assets::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Assets1,
    #[sea_orm(
        belongs_to = "super::nodes::Entity",
        from = "Column::NodeId",
        to = "super::nodes::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Nodes,
}

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nodes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
