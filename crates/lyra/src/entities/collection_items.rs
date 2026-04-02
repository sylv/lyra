use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "collection_items")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub collection_id: String,
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub node_id: String,
    pub position: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::collections::Entity",
        from = "Column::CollectionId",
        to = "super::collections::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Collections,
    #[sea_orm(
        belongs_to = "super::nodes::Entity",
        from = "Column::NodeId",
        to = "super::nodes::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Nodes,
}

impl Related<super::collections::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Collections.def()
    }
}

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nodes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
