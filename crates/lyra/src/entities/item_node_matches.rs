use super::node_match_status::NodeMatchStatus;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "item_node_matches")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_type = "Text")]
    pub root_id: String,
    #[sea_orm(column_type = "Text")]
    pub item_id: String,
    #[sea_orm(column_type = "Text")]
    pub provider_id: String,
    pub status: NodeMatchStatus,
    pub last_attempted_at: Option<i64>,
    pub last_added_at: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub last_error_message: Option<String>,
    pub retry_after: Option<i64>,
    pub attempts: i64,
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
        belongs_to = "super::items::Entity",
        from = "Column::ItemId",
        to = "super::items::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Items,
}

impl Related<super::roots::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Roots.def()
    }
}

impl Related<super::items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Items.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
