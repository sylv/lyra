use crate::entities::users::Permissions;
use crate::entities::{invites, users, watch_state};
use crate::{PermissionGuard, RequestAuth};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use async_graphql::{Context, Object};
use chrono::Utc;
use rand::RngCore;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait};
use sea_orm::{PaginatorTrait, Set};

pub struct Mutation;

#[Object]
impl Mutation {
    #[graphql(guard = PermissionGuard::new(Permissions::CREATE_USER))]
    async fn signup(
        &self,
        ctx: &Context<'_>,
        username: String,
        password: String,
        invite_code: Option<String>,
    ) -> Result<users::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let (invite, permissions) = if let Some(invite_code) = invite_code {
            let invite = invites::Entity::find_by_id(invite_code)
                .one(pool)
                .await
                .map_err(|e| async_graphql::Error::new(e.to_string()))?
                .ok_or_else(|| async_graphql::Error::new("Invalid invite code".to_string()))?;

            if invite.used_at.is_some() {
                return Err(async_graphql::Error::new(
                    "Invite code already used".to_string(),
                ));
            }

            if invite.expires_at < Utc::now().timestamp() {
                return Err(async_graphql::Error::new("Invite code expired".to_string()));
            }

            let permissions = invite.permissions;
            (Some(invite), permissions)
        } else {
            // if the user does not have an invite, we only allow user creation if no
            // users exist.
            let users = users::Entity::find().count(pool).await?;
            if users > 0 {
                return Err(async_graphql::Error::new(
                    "No invite code provided and users already exist".to_string(),
                ));
            }

            (None, Permissions::ADMIN.bits())
        };

        let password_hash = {
            let argon2 = Argon2::default();
            let salt = SaltString::generate(&mut OsRng);
            argon2
                .hash_password(password.as_bytes(), &salt)?
                .to_string()
        };

        let tx = pool.begin().await?;
        let id = ulid::Ulid::new().to_string();
        let user = users::Entity::insert(users::ActiveModel {
            id: Set(id),
            username: Set(username),
            password_hash: Set(password_hash),
            permissions: Set(permissions),
            ..Default::default()
        })
        .exec_with_returning(&tx)
        .await
        .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        if let Some(invite) = invite {
            invites::Entity::update(invites::ActiveModel {
                code: Set(invite.code),
                used_at: Set(Some(Utc::now().timestamp())),
                used_by: Set(Some(user.id.clone())),
                ..Default::default()
            })
            .filter(invites::Column::UsedBy.is_null())
            .exec(&tx)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        }

        tx.commit().await?;
        Ok(user)
    }

    #[graphql(guard = PermissionGuard::new(Permissions::CREATE_INVITE))]
    async fn create_invite(
        &self,
        ctx: &Context<'_>,
        permissions: u32,
    ) -> Result<invites::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let invite_code = {
            let mut bytes = [0u8; 16];
            rand::rng().fill_bytes(&mut bytes);
            hex::encode(bytes)
        };

        let user = ctx
            .data::<RequestAuth>()?
            .user
            .as_ref()
            .ok_or_else(|| async_graphql::Error::new("User not found".to_string()))?;

        let invite = invites::Entity::insert(invites::ActiveModel {
            code: Set(invite_code),
            permissions: Set(permissions),
            created_by: Set(user.id.clone()),
            created_at: Set(Utc::now().timestamp()),
            expires_at: Set(Utc::now().timestamp() + 30 * 24 * 60 * 60),
            ..Default::default()
        })
        .exec_with_returning(pool)
        .await
        .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(invite)
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
            if !auth.has_permission(Permissions::EDIT_OTHERS_WATCH_STATE) {
                return Err(async_graphql::Error::new(
                    "Lacking permission to edit watch state for other users".to_string(),
                ));
            }

            user_id
        } else {
            let user = auth
                .user
                .as_ref()
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
}
