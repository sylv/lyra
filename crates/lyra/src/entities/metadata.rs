use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum MetadataKind {
    Movie = 0,
    Series = 1,
    Season = 2,
    Episode = 3,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "metadata")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub root_id: Option<i64>,
    pub parent_id: Option<i64>,
    #[sea_orm(column_type = "Text")]
    pub source: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub source_key: Option<String>,
    pub kind: MetadataKind,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub score_display: Option<String>,
    pub score_normalized: Option<i64>,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
    pub released_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub poster_asset_id: Option<i64>,
    pub thumbnail_asset_id: Option<i64>,
    pub background_asset_id: Option<i64>,
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
    BackgroundAsset,
    #[sea_orm(
        belongs_to = "super::assets::Entity",
        from = "Column::ThumbnailAssetId",
        to = "super::assets::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    ThumbnailAsset,
    #[sea_orm(
        belongs_to = "super::assets::Entity",
        from = "Column::PosterAssetId",
        to = "super::assets::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    PosterAsset,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Parent,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::RootId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Root,
    #[sea_orm(has_many = "super::node_metadata::Entity")]
    NodeMetadata,
}

impl Related<super::node_metadata::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NodeMetadata.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
