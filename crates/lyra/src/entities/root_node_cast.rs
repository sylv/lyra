use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "root_node_cast")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub root_node_id: String,
    pub person_id: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub character_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub department: Option<String>,
    pub position: i64,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::nodes::Entity",
        from = "Column::RootNodeId",
        to = "super::nodes::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Nodes,
    #[sea_orm(
        belongs_to = "super::people::Entity",
        from = "Column::PersonId",
        to = "super::people::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    People,
}

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nodes.def()
    }
}

impl Related<super::people::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::People.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
