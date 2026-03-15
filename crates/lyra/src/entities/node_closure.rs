use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "node_closure")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub ancestor_id: String,
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub descendant_id: String,
    pub depth: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::nodes::Entity",
        from = "Column::AncestorId",
        to = "super::nodes::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Ancestor,
    #[sea_orm(
        belongs_to = "super::nodes::Entity",
        from = "Column::DescendantId",
        to = "super::nodes::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Descendant,
}

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ancestor.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
