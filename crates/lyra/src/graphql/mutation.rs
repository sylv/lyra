use crate::RequestAuth;
use crate::auth::PermissionGuard;
use crate::entities::users::UserPerms;
use crate::entities::{files, item_files, libraries, users, watch_progress};
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
    QuerySelect,
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
            let user = blank_user.update(pool).await?;
            Ok(user)
        } else {
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
                permissions: Set(permissions as i64),
                ..Default::default()
            })
            .exec_with_returning(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

            Ok(user)
        }
    }

    async fn update_watch_progress(
        &self,
        ctx: &Context<'_>,
        file_id: i64,
        progress_percent: f32,
        user_id: Option<String>,
    ) -> Result<Vec<watch_progress::Model>, async_graphql::Error> {
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

        let file = files::Entity::find_by_id(file_id)
            .filter(files::Column::UnavailableAt.is_null())
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("File not found".to_string()))?;

        let linked_item_ids: Vec<String> = item_files::Entity::find()
            .filter(item_files::Column::FileId.eq(file.id))
            .select_only()
            .column(item_files::Column::ItemId)
            .distinct()
            .into_tuple()
            .all(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        if linked_item_ids.is_empty() {
            return Err(async_graphql::Error::new(
                "No linked items found for file".to_string(),
            ));
        }

        let now = Utc::now().timestamp();
        let mut updated_rows = Vec::with_capacity(linked_item_ids.len());

        for item_id in linked_item_ids {
            let row = watch_progress::Entity::insert(watch_progress::ActiveModel {
                user_id: Set(user_id.clone()),
                item_id: Set(item_id),
                file_id: Set(file.id),
                progress_percent: Set(progress_percent),
                updated_at: Set(now),
                ..Default::default()
            })
            .on_conflict(
                OnConflict::columns([
                    watch_progress::Column::UserId,
                    watch_progress::Column::ItemId,
                ])
                .update_columns([
                    watch_progress::Column::FileId,
                    watch_progress::Column::ProgressPercent,
                    watch_progress::Column::UpdatedAt,
                ])
                .to_owned(),
            )
            .exec_with_returning(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

            updated_rows.push(row);
        }

        Ok(updated_rows)
    }

    async fn create_library(
        &self,
        ctx: &Context<'_>,
        name: String,
        path: String,
    ) -> Result<libraries::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;

        if !auth.has_permission(UserPerms::ADMIN) {
            return Err(async_graphql::Error::new(
                "Lacking permission to create libraries".to_string(),
            ));
        }

        let library = libraries::Entity::insert(libraries::ActiveModel {
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
