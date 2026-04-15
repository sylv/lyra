use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "node_metadata_content_ratings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub node_metadata_id: String,
    pub country_code: String,
    pub rating: String,
    pub release_date: Option<i64>,
    pub release_type: Option<i64>,
    pub position: i64,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::node_metadata::Entity",
        from = "Column::NodeMetadataId",
        to = "super::node_metadata::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    NodeMetadata,
}

impl Related<super::node_metadata::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NodeMetadata.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
