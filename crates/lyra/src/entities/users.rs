use async_graphql::SimpleObject;
use bitflags::bitflags;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, SimpleObject)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub username: String,
    #[graphql(skip)]
    pub password_hash: String,
    pub permissions: u32,
    pub default_subtitle_bcp47: Option<String>,
    pub default_audio_bcp47: Option<String>,
    pub subtitles_enabled: i64,
    pub created_at: i64,
    pub last_login_at: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::sessions::Entity")]
    Sessions,
}

impl Related<super::sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sessions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Permissions: u32 {
        const ADMIN = 1 << 0;
        const CREATE_INVITE = 1 << 1;
        const CREATE_USER = 1 << 2;
    }
}
