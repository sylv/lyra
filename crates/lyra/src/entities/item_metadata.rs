use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "item_metadata")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_type = "Text")]
    pub item_id: String,
    #[sea_orm(column_type = "Text")]
    pub source: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub source_key: Option<String>,
    pub is_primary: bool,
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
        belongs_to = "super::items::Entity",
        from = "Column::ItemId",
        to = "super::items::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Items,
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

impl Related<super::items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Items.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
