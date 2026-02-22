use async_graphql::SimpleObject;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, SimpleObject)]
#[sea_orm(table_name = "seasons")]
#[graphql(name = "SeasonNode", complex)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    #[sea_orm(column_type = "Text")]
    pub root_id: String,
    pub season_number: i64,
    pub order: i64,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    pub last_added_at: i64,
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
    #[sea_orm(has_many = "super::items::Entity")]
    Items,
    #[sea_orm(has_many = "super::season_metadata::Entity")]
    SeasonMetadata,
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

impl Related<super::season_metadata::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SeasonMetadata.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
