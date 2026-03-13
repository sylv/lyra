use crate::RequestAuth;
use crate::auth::{PermissionGuard, create_session_for_user};
use crate::entities::users::UserPerms;
use crate::entities::{files, item_files, libraries, users, watch_progress};
use crate::import::watch_state_import;
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use async_graphql::{Context, InputObject, Object, SimpleObject};
use chrono::Utc;
use sea_orm::Set;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    QuerySelect,
};

pub struct Mutation;

#[derive(Debug, Clone, InputObject)]
pub struct ImportWatchStatesInput {
    pub dry_run: bool,
    pub overwrite_conflicts: bool,
    pub rows: Vec<ImportWatchStateRowInput>,
}

#[derive(Debug, Clone, InputObject)]
pub struct ImportWatchStateRowInput {
    pub source: String,
    pub source_item_id: Option<String>,
    pub title: Option<String>,
    pub media_type: Option<String>,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
    pub progress_percent: f32,
    pub viewed_at: Option<i64>,
    pub file_path: Option<String>,
    pub file_basename: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<i64>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ImportWatchStateConflict {
    pub row_index: i32,
    pub source_item_id: Option<String>,
    pub title: Option<String>,
    pub item_id: String,
    pub existing_progress_percent: f32,
    pub imported_progress_percent: f32,
    pub reason: String,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ImportWatchStateUnmatched {
    pub row_index: i32,
    pub source_item_id: Option<String>,
    pub title: Option<String>,
    pub reason: String,
    pub ambiguous: bool,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ImportWatchStatesResult {
    pub dry_run: bool,
    pub total_rows: i32,
    pub matched_rows: i32,
    pub unmatched_rows: i32,
    pub conflict_rows: i32,
    pub will_insert: i32,
    pub will_overwrite: i32,
    pub imported: i32,
    pub skipped: i32,
    pub conflicts: Vec<ImportWatchStateConflict>,
    pub unmatched: Vec<ImportWatchStateUnmatched>,
}

impl From<watch_state_import::ImportWatchStateConflictData> for ImportWatchStateConflict {
    fn from(value: watch_state_import::ImportWatchStateConflictData) -> Self {
        Self {
            row_index: value.row_index,
            source_item_id: value.source_item_id,
            title: value.title,
            item_id: value.item_id,
            existing_progress_percent: value.existing_progress_percent,
            imported_progress_percent: value.imported_progress_percent,
            reason: value.reason,
        }
    }
}

impl From<watch_state_import::ImportWatchStateUnmatchedData> for ImportWatchStateUnmatched {
    fn from(value: watch_state_import::ImportWatchStateUnmatchedData) -> Self {
        Self {
            row_index: value.row_index,
            source_item_id: value.source_item_id,
            title: value.title,
            reason: value.reason,
            ambiguous: value.ambiguous,
        }
    }
}

impl From<watch_state_import::ImportWatchStatesResultData> for ImportWatchStatesResult {
    fn from(value: watch_state_import::ImportWatchStatesResultData) -> Self {
        Self {
            dry_run: value.dry_run,
            total_rows: value.total_rows,
            matched_rows: value.matched_rows,
            unmatched_rows: value.unmatched_rows,
            conflict_rows: value.conflict_rows,
            will_insert: value.will_insert,
            will_overwrite: value.will_overwrite,
            imported: value.imported,
            skipped: value.skipped,
            conflicts: value.conflicts.into_iter().map(Into::into).collect(),
            unmatched: value.unmatched.into_iter().map(Into::into).collect(),
        }
    }
}

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

            if auth.is_setup() {
                let cookie = create_session_for_user(pool, &user.id)
                    .await
                    .map_err(|e| -> async_graphql::Error { e.into() })?;
                ctx.insert_http_header("Set-Cookie", cookie);
            }

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

    async fn import_watch_states(
        &self,
        ctx: &Context<'_>,
        input: ImportWatchStatesInput,
    ) -> Result<ImportWatchStatesResult, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;

        let request = watch_state_import::ImportWatchStatesRequest {
            user_id: user.id.clone(),
            overwrite_conflicts: input.overwrite_conflicts,
            rows: input
                .rows
                .into_iter()
                .map(|row| watch_state_import::ImportWatchStateRow {
                    source: row.source,
                    source_item_id: row.source_item_id,
                    title: row.title,
                    media_type: row.media_type,
                    season_number: row.season_number,
                    episode_number: row.episode_number,
                    progress_percent: row.progress_percent,
                    viewed_at: row.viewed_at,
                    file_path: row.file_path,
                    file_basename: row.file_basename,
                    file_size_bytes: row.file_size_bytes,
                    imdb_id: row.imdb_id,
                    tmdb_id: row.tmdb_id,
                })
                .collect(),
        };

        let result = if input.dry_run {
            watch_state_import::dry_run(pool, request).await
        } else {
            watch_state_import::commit(pool, request).await
        }
        .map_err(|error| async_graphql::Error::new(error.to_string()))?;

        Ok(result.into())
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
