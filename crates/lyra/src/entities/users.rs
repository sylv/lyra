use async_graphql::SimpleObject;
use bitflags::bitflags;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, SimpleObject)]
#[sea_orm(table_name = "users")]
#[graphql(name = "User")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    #[sea_orm(column_type = "Text", unique)]
    pub username: String,
    #[sea_orm(column_type = "Text", nullable)]
    #[graphql(skip)]
    pub password_hash: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    #[graphql(skip)]
    pub invite_code: Option<String>,
    pub permissions: i64,
    #[sea_orm(column_type = "Text", nullable)]
    pub default_subtitle_iso639_1: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub default_audio_iso639_1: Option<String>,
    pub subtitles_enabled: bool,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::library_users::Entity")]
    LibraryUsers,
    #[sea_orm(has_many = "super::user_sessions::Entity")]
    UserSessions,
    #[sea_orm(has_many = "super::watch_progress::Entity")]
    WatchProgress,
}

impl Related<super::library_users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LibraryUsers.def()
    }
}

impl Related<super::user_sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserSessions.def()
    }
}

impl Related<super::watch_progress::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WatchProgress.def()
    }
}

impl Related<super::libraries::Entity> for Entity {
    fn to() -> RelationDef {
        super::library_users::Relation::Libraries.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::library_users::Relation::Users.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct UserPerms: u32 {
        const ADMIN = 1 << 0;
        const CREATE_INVITE = 1 << 1;
        const CREATE_USER = 1 << 2;
        const EDIT_OTHERS_WATCH_STATE = 1 << 3;
        const VIEW_ALL_LIBRARIES = 1 << 4;
    }
}
