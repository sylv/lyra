use super::metadata_source::MetadataSource;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "root_metadata")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_type = "Text")]
    pub root_id: String,
    pub source: MetadataSource,
    #[sea_orm(column_type = "Text")]
    pub provider_id: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<i64>,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub score_display: Option<String>,
    pub score_normalized: Option<i64>,
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
        belongs_to = "super::roots::Entity",
        from = "Column::RootId",
        to = "super::roots::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Roots,
    #[sea_orm(
        belongs_to = "super::assets::Entity",
        from = "Column::PosterAssetId",
        to = "super::assets::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    PosterAsset,
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
        from = "Column::BackgroundAssetId",
        to = "super::assets::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    BackgroundAsset,
}

impl Related<super::roots::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Roots.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
