use async_graphql::SimpleObject;
use bitflags::bitflags;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, SimpleObject)]
#[sea_orm(table_name = "users")]
#[graphql(name = "User", complex)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    #[sea_orm(column_type = "Text", unique)]
    pub username: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub password_hash: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub invite_code: Option<String>,
    pub permissions: i64,
    pub created_at: i64,
    #[sea_orm(column_type = "Text", nullable)]
    pub preferred_audio_language: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub preferred_audio_disposition: Option<String>,
    #[graphql(skip)]
    pub subtitle_mode: SubtitleMode,
    #[sea_orm(column_type = "Text")]
    pub preferred_subtitle_languages: String,
    #[graphql(skip)]
    pub subtitle_variant_preference: SubtitleVariantPreference,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_sessions::Entity")]
    UserSessions,
    #[sea_orm(has_many = "super::watch_progress::Entity")]
    WatchProgress,
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
