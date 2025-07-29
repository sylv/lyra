use async_graphql::SimpleObject;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, SimpleObject)]
#[sea_orm(table_name = "watch_state")]
#[graphql(name = "WatchState")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub media_id: i64,
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub user_id: String,
    pub progress_percentage: f32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::media::Entity",
        from = "Column::MediaId",
        to = "super::media::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Media,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Users,
}

impl Related<super::media::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Media.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
