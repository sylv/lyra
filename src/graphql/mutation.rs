use crate::RequestAuth;
use crate::auth::PermissionGuard;
use crate::entities::users::UserPerms;
use crate::entities::{library, users, watch_state};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use async_graphql::{Context, Object};
use chrono::Utc;
use sea_orm::Set;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
};

pub struct Mutation;

#[Object]
impl Mutation {
    #[graphql(guard = PermissionGuard::new(UserPerms::CREATE_USER))]
    async fn signup(
        &self,
        ctx: &Context<'_>,
        username: String,
        password: String,
        permissions: Option<u32>,
        invite_code: Option<String>,
    ) -> Result<users::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let password_hash = {
            let argon2 = Argon2::default();
            let salt = SaltString::generate(&mut OsRng);
            argon2
                .hash_password(password.as_bytes(), &salt)?
                .to_string()
        };

        if let Some(invite_code) = invite_code {
            // this handles accepting an invite for an existing user.
            let blank_user = users::Entity::find()
                .filter(users::Column::InviteCode.eq(invite_code))
                .one(pool)
                .await
                .map_err(|e| async_graphql::Error::new(e.to_string()))?;

            let Some(blank_user) = blank_user else {
                return Err(async_graphql::Error::new(
                    "Invite code already used".to_string(),
                ));
            };

            if permissions.is_some() {
                return Err(async_graphql::Error::new(
                    "Permissions cannot be set when accepting an invite".to_string(),
                ));
            }

            let mut blank_user = blank_user.into_active_model();
            blank_user.password_hash = Set(Some(password_hash));
            blank_user.invite_code = Set(None);
            let user = blank_user.insert(pool).await?;
            Ok(user)
        } else {
            // this handles creating a new user once a user already exists, aka inviting
            // people. invites are just users with no password yet, which allows you to setup
            // their account before they can sign in.
            let auth = ctx.data::<RequestAuth>()?;
            if !auth.has_permission(UserPerms::CREATE_USER) {
                return Err(async_graphql::Error::new(
                    "No invite code provided and users already exist".to_string(),
                ));
            }

            let id = ulid::Ulid::new().to_string();
            let permissions = permissions.unwrap_or_else(|| {
                if auth.is_setup() {
                    UserPerms::ADMIN.bits()
                } else {
                    UserPerms::empty().bits()
                }
            });

            let user = users::Entity::insert(users::ActiveModel {
                id: Set(id),
                username: Set(username),
                password_hash: Set(Some(password_hash)),
                permissions: Set(permissions),
                ..Default::default()
            })
            .exec_with_returning(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

            Ok(user)
        }
    }

    async fn update_watch_state(
        &self,
        ctx: &Context<'_>,
        media_id: i64,
        progress_percentage: f32,
        user_id: Option<String>,
    ) -> Result<watch_state::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;

        let user_id = if let Some(user_id) = user_id {
            if !auth.has_permission(UserPerms::EDIT_OTHERS_WATCH_STATE) {
                return Err(async_graphql::Error::new(
                    "Lacking permission to edit watch state for other users".to_string(),
                ));
            }

            user_id
        } else {
            let user = auth
                .get_user()
                .ok_or_else(|| async_graphql::Error::new("No user in context".to_string()))?;

            user.id.clone()
        };

        let watch_state = watch_state::Entity::insert(watch_state::ActiveModel {
            media_id: Set(media_id),
            user_id: Set(user_id),
            progress_percentage: Set(progress_percentage),
            updated_at: Set(Utc::now().timestamp()),
            ..Default::default()
        })
        .on_conflict(
            OnConflict::columns([watch_state::Column::MediaId, watch_state::Column::UserId])
                .update_columns([
                    watch_state::Column::ProgressPercentage,
                    watch_state::Column::UpdatedAt,
                ])
                .to_owned(),
        )
        .exec_with_returning(pool)
        .await
        .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(watch_state)
    }

    async fn create_library(
        &self,
        ctx: &Context<'_>,
        name: String,
        path: String,
    ) -> Result<library::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;

        if !auth.has_permission(UserPerms::ADMIN) {
            return Err(async_graphql::Error::new(
                "Lacking permission to create libraries".to_string(),
            ));
        }

        let library = library::Entity::insert(library::ActiveModel {
            name: Set(name),
            path: Set(path),
            ..Default::default()
        })
        .exec_with_returning(pool)
        .await
        .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(library)
    }
}
