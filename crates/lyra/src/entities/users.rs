use async_graphql::SimpleObject;
use bitflags::bitflags;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, SimpleObject)]
#[sea_orm(table_name = "users")]
#[graphql(name = "User", complex)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub username: String,
    #[graphql(skip)]
    pub password_hash: Option<String>,
    pub invite_code: Option<String>,
    pub permissions: i64,
    pub created_at: i64,
    pub preferred_audio_language: Option<String>,
    pub preferred_audio_disposition: Option<String>,
    #[graphql(skip)]
    pub subtitle_mode: SubtitleMode,
    #[graphql(skip)]
    pub preferred_subtitle_languages: String,
    #[graphql(skip)]
    pub subtitle_variant_preference: SubtitleVariantPreference,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::library_users::Entity")]
    LibraryUsers,
    #[sea_orm(has_many = "super::collections::Entity")]
    Collections,
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

impl Related<super::collections::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Collections.def()
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum SubtitleMode {
    Off = 0,
    ForcedOnly = 1,
    On = 2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum SubtitleVariantPreference {
    Auto = 0,
    Forced = 1,
    Normal = 2,
    Sdh = 3,
    Commentary = 4,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct UserPerms: u32 {
        const ADMIN = 1 << 0;
        const CREATE_INVITE = 1 << 1;
        const EDIT_OTHERS_WATCH_STATE = 1 << 3;
        const VIEW_ALL_LIBRARIES = 1 << 4;
    }
}
