use async_graphql::SimpleObject;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, SimpleObject)]
#[sea_orm(table_name = "invites")]
#[graphql(name = "Invite")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub code: String,
    pub permissions: u32,
    #[sea_orm(column_type = "Text")]
    pub created_by: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub used_at: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub used_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UsedBy",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Users2,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::CreatedBy",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Users1,
}

impl ActiveModelBehavior for ActiveModel {}
