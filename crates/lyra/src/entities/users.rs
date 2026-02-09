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
    pub permissions: u32,
    #[sea_orm(column_type = "Text", nullable)]
    pub default_subtitle_bcp47: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub default_audio_bcp47: Option<String>,
    pub subtitles_enabled: bool,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::library_user::Entity")]
    LibraryUser,
    #[sea_orm(has_many = "super::sessions::Entity")]
    Sessions,
    #[sea_orm(has_many = "super::watch_state::Entity")]
    WatchState,
}

impl Related<super::library_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LibraryUser.def()
    }
}

impl Related<super::sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sessions.def()
    }
}

impl Related<super::watch_state::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WatchState.def()
    }
}

impl Related<super::library::Entity> for Entity {
    fn to() -> RelationDef {
        super::library_user::Relation::Library.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::library_user::Relation::Users.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::library_user::Entity")]
    LibraryUser,
    #[sea_orm(entity = "super::sessions::Entity")]
    Sessions,
    #[sea_orm(entity = "super::watch_state::Entity")]
    WatchState,
    #[sea_orm(entity = "super::library::Entity")]
    Library,
}

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
