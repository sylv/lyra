use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "people")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub provider_id: String,
    pub provider_person_id: String,
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub birthday: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub profile_asset_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::root_node_cast::Entity")]
    RootNodeCast,
    #[sea_orm(
        belongs_to = "super::assets::Entity",
        from = "Column::ProfileAssetId",
        to = "super::assets::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Assets,
}

impl Related<super::root_node_cast::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RootNodeCast.def()
    }
}

impl Related<super::assets::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Assets.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
