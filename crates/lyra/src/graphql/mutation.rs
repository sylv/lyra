use crate::auth::{
    AuthenticatedGuard, PermissionGuard, create_session_for_user, find_pending_invite_user,
    get_set_cookie_headers_for_session,
};
use crate::entities::users::SubtitleMode;
use crate::entities::users::UserPerms;
use crate::entities::{files, libraries, node_files, user_sessions, users, watch_progress};
use crate::graphql::properties::TrackDispositionPreference;
use crate::graphql::types::file::parse_source_track_id;
use crate::hls;
use crate::ids::{self, new_invite_code};
use crate::subtitles::language::SubtitleTrackVariant;
use crate::{RequestAuth, UserAgent};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use async_graphql::{Context, InputObject, Object};
use chrono::Utc;
use reqwest::header::SET_COOKIE;
use sea_orm::Set;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter, QuerySelect, TransactionTrait,
};
pub struct Mutation;

fn normalize_username(username: String) -> Result<String, async_graphql::Error> {
    let username = username.trim();
    if username.is_empty() {
        return Err(async_graphql::Error::new("Username is required"));
    }

    Ok(username.to_string())
}

fn hash_password(password: &str) -> Result<String, async_graphql::Error> {
    if password.is_empty() {
        return Err(async_graphql::Error::new("Password is required"));
    }

    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

fn subtitle_track_variant_for_stream(stream: &lyra_probe::Stream) -> SubtitleTrackVariant {
    if stream.is_commentary() {
        SubtitleTrackVariant::Commentary
    } else if stream.is_forced() {
        SubtitleTrackVariant::Forced
    } else if stream.is_hearing_impaired() {
        SubtitleTrackVariant::Sdh
    } else {
        SubtitleTrackVariant::Normal
    }
}

async fn update_subtitle_preferences_for_disable(
    pool: &DatabaseConnection,
    user_id: &str,
    variant: SubtitleTrackVariant,
) -> Result<(), async_graphql::Error> {
    let user = users::Entity::find_by_id(user_id.to_string())
        .one(pool)
        .await?
        .ok_or_else(|| async_graphql::Error::new("User not found"))?;
    let current_mode = user.subtitle_mode;
    let mut active: users::ActiveModel = user.into();
    active.subtitle_mode = Set(match (current_mode, variant) {
        (SubtitleMode::ForcedOnly, _) => SubtitleMode::Off,
        (SubtitleMode::On, SubtitleTrackVariant::Forced) => SubtitleMode::Off,
        (SubtitleMode::On, _) => SubtitleMode::ForcedOnly,
        (SubtitleMode::Off, _) => SubtitleMode::Off,
    });
    active.update(pool).await?;
    Ok(())
}

#[derive(Debug, Clone, InputObject)]
pub struct DisabledSubtitlesHintInput {
    pub file_id: String,
    pub source_track_id: String,
}

#[Object]
impl Mutation {
    pub async fn signup(
        &self,
        ctx: &Context<'_>,
        username: String,
        password: String,
        permissions: Option<u32>,
        invite_code: Option<String>,
    ) -> Result<users::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let username = normalize_username(username)?;
        let password_hash = hash_password(&password)?;

        let mut auto_signin = false;
        let user = if let Some(invite_code) = invite_code {
            let blank_user = find_pending_invite_user(pool, &invite_code)
                .await
                .map_err(|e| async_graphql::Error::new(e.to_string()))?;

            let Some(blank_user) = blank_user else {
                return Err(async_graphql::Error::new(
                    "Invite code is invalid or already used".to_string(),
                ));
            };

            if permissions.is_some() {
                return Err(async_graphql::Error::new(
                    "Permissions cannot be set when accepting an invite".to_string(),
                ));
            }

            let mut blank_user = blank_user.into_active_model();
            blank_user.username = Set(username);
            blank_user.password_hash = Set(Some(password_hash));
            blank_user.invite_code = Set(None);
            let user = blank_user.update(pool).await?;
            auto_signin = true;
            user
        } else {
            let auth = ctx.data_opt::<RequestAuth>().ok_or_else(|| {
                async_graphql::Error::new("No invite code provided and users already exist")
            })?;

            if !auth.has_permission(UserPerms::ADMIN) {
                return Err(async_graphql::Error::new(
                    "No invite code provided and users already exist".to_string(),
                ));
            }

            let id = ids::generate_ulid();
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
                auto_signin = true;
            }

            user
        };

        if auto_signin {
            let ua = ctx.data::<UserAgent>()?.0.clone();
            let session_id = create_session_for_user(pool, &user.id, ua)
                .await
                .map_err(|e| -> async_graphql::Error { e.into() })?;
            let cookie = get_set_cookie_headers_for_session(user.id.clone(), session_id)?;
            ctx.insert_http_header(SET_COOKIE, cookie);
        }

        Ok(user)
    }

    #[graphql(guard = PermissionGuard::new(UserPerms::ADMIN))]
    pub async fn create_user_invite(
        &self,
        ctx: &Context<'_>,
        username: String,
        permissions: u32,
    ) -> Result<users::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let username = normalize_username(username)?;
        let invite_code = new_invite_code();
        let txn = pool
            .begin()
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let user = users::Entity::insert(users::ActiveModel {
            id: Set(ids::generate_ulid()),
            username: Set(username),
            password_hash: Set(None),
            invite_code: Set(Some(invite_code)),
            permissions: Set(permissions as i64),
            ..Default::default()
        })
        .exec_with_returning(&txn)
        .await
        .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        txn.commit()
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(user)
    }

    #[graphql(guard = PermissionGuard::new(UserPerms::ADMIN))]
    pub async fn update_user(
        &self,
        ctx: &Context<'_>,
        user_id: String,
        username: String,
        permissions: u32,
    ) -> Result<users::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let username = normalize_username(username)?;
        let existing_user = users::Entity::find_by_id(user_id)
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("User not found".to_string()))?;
        if auth
            .get_user()
            .is_some_and(|current_user| current_user.id == existing_user.id)
            && existing_user.permissions != permissions as i64
        {
            return Err(async_graphql::Error::new(
                "You cannot edit your current account permissions or library access",
            ));
        }

        let mut user = existing_user.into_active_model();
        user.username = Set(username);
        user.permissions = Set(permissions as i64);

        let txn = pool
            .begin()
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        let user = user
            .update(&txn)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        txn.commit()
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(user)
    }

    #[graphql(guard = PermissionGuard::new(UserPerms::ADMIN))]
    pub async fn reset_user_invite(
        &self,
        ctx: &Context<'_>,
        user_id: String,
    ) -> Result<users::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let existing_user = users::Entity::find_by_id(user_id.clone())
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("User not found".to_string()))?;

        if auth
            .get_user()
            .is_some_and(|current_user| current_user.id == existing_user.id)
        {
            return Err(async_graphql::Error::new(
                "You cannot reset your current account",
            ));
        }

        let user_count = users::Entity::find().count(pool).await?;
        if user_count <= 1 {
            return Err(async_graphql::Error::new(
                "The last remaining account cannot be reset",
            ));
        }
        let invite_code = new_invite_code();
        let txn = pool
            .begin()
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let mut user = existing_user.into_active_model();
        user.password_hash = Set(None);
        user.invite_code = Set(Some(invite_code));
        let user = user
            .update(&txn)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        user_sessions::Entity::delete_many()
            .filter(user_sessions::Column::UserId.eq(user_id))
            .exec(&txn)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        txn.commit()
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(user)
    }

    #[graphql(guard = PermissionGuard::new(UserPerms::ADMIN))]
    pub async fn delete_user(
        &self,
        ctx: &Context<'_>,
        user_id: String,
    ) -> Result<bool, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let existing_user = users::Entity::find_by_id(user_id.clone())
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("User not found".to_string()))?;

        if auth
            .get_user()
            .is_some_and(|current_user| current_user.id == existing_user.id)
        {
            return Err(async_graphql::Error::new(
                "You cannot delete your current account",
            ));
        }

        let user_count = users::Entity::find().count(pool).await?;
        if user_count <= 1 {
            return Err(async_graphql::Error::new(
                "The last remaining account cannot be deleted",
            ));
        }

        users::Entity::delete_by_id(user_id)
            .exec(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(true)
    }

    pub async fn update_watch_progress(
        &self,
        ctx: &Context<'_>,
        file_id: String,
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

        let linked_node_ids: Vec<String> = node_files::Entity::find()
            .filter(node_files::Column::FileId.eq(file.id.clone()))
            .select_only()
            .column(node_files::Column::NodeId)
            .distinct()
            .into_tuple()
            .all(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        if linked_node_ids.is_empty() {
            return Err(async_graphql::Error::new(
                "No linked nodes found for file".to_string(),
            ));
        }

        let normalized_progress_percent =
            watch_progress::normalize_progress_percent(progress_percent);
        let now = Utc::now().timestamp();
        let mut updated_rows = Vec::with_capacity(linked_node_ids.len());

        for node_id in linked_node_ids {
            let row = watch_progress::Entity::insert(watch_progress::ActiveModel {
                id: Set(ids::generate_ulid()),
                user_id: Set(user_id.clone()),
                node_id: Set(node_id),
                file_id: Set(file.id.clone()),
                progress_percent: Set(normalized_progress_percent),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            })
            .on_conflict(
                OnConflict::columns([
                    watch_progress::Column::UserId,
                    watch_progress::Column::NodeId,
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

    pub async fn create_library(
        &self,
        ctx: &Context<'_>,
        name: String,
        path: String,
        pinned: Option<bool>,
    ) -> Result<libraries::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;

        if !auth.has_permission(UserPerms::ADMIN) {
            return Err(async_graphql::Error::new(
                "Lacking permission to create libraries".to_string(),
            ));
        }

        let library = libraries::Entity::insert(libraries::ActiveModel {
            id: Set(ids::generate_ulid()),
            name: Set(name),
            path: Set(path),
            pinned: Set(pinned.unwrap_or(true)),
            ..Default::default()
        })
        .exec_with_returning(pool)
        .await
        .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(library)
    }

    pub async fn update_library(
        &self,
        ctx: &Context<'_>,
        library_id: String,
        name: String,
        path: String,
        pinned: bool,
    ) -> Result<libraries::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;

        if !auth.has_permission(UserPerms::ADMIN) {
            return Err(async_graphql::Error::new(
                "Lacking permission to update libraries".to_string(),
            ));
        }

        let existing_library = libraries::Entity::find_by_id(library_id)
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("Library not found".to_string()))?;
        let path_changed = existing_library.path != path;
        let mut library = existing_library.into_active_model();
        library.name = Set(name);
        library.path = Set(path);
        library.pinned = Set(pinned);

        // force the scheduler to rescan quickly when the root changes
        // instead of leaving the moved library on the previous scan cadence.
        if path_changed {
            library.last_scanned_at = Set(None);
        }

        let library = library
            .update(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(library)
    }

    pub async fn set_preferred_audio(
        &self,
        ctx: &Context<'_>,
        language: Option<String>,
        disposition: Option<TrackDispositionPreference>,
    ) -> Result<users::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;

        let existing = users::Entity::find_by_id(user.id.clone())
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("User not found"))?;

        let mut active = existing.into_active_model();
        active.preferred_audio_language = Set(language);
        active.preferred_audio_disposition = Set(disposition.map(|d| d.as_str().to_string()));

        let updated = active
            .update(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(updated)
    }

    pub async fn delete_library(
        &self,
        ctx: &Context<'_>,
        library_id: String,
    ) -> Result<bool, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;

        if !auth.has_permission(UserPerms::ADMIN) {
            return Err(async_graphql::Error::new(
                "Lacking permission to delete libraries".to_string(),
            ));
        }

        libraries::Entity::find_by_id(library_id.clone())
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("Library not found".to_string()))?;

        libraries::Entity::delete_by_id(library_id)
            .exec(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(true)
    }

    #[graphql(guard = AuthenticatedGuard::new())]
    pub async fn disabled_subtitles_hint(
        &self,
        ctx: &Context<'_>,
        input: DisabledSubtitlesHintInput,
    ) -> Result<bool, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;
        let file = files::Entity::find_by_id(input.file_id.clone())
            .filter(files::Column::UnavailableAt.is_null())
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("File not found".to_string()))?;
        let stream_index = parse_source_track_id(&input.source_track_id)
            .ok_or_else(|| async_graphql::Error::new("Invalid subtitle track"))?;

        let (probe, _) = hls::load_probe_data_for_playback_options(pool, &file.id)
            .await
            .map_err(|error| async_graphql::Error::new(error.to_string()))?;
        let stream = probe
            .stream(stream_index)
            .ok_or_else(|| async_graphql::Error::new("Subtitle not found"))?;
        if stream.kind() != lyra_probe::StreamKind::Subtitle {
            return Err(async_graphql::Error::new("Subtitle not found"));
        }

        update_subtitle_preferences_for_disable(
            pool,
            &user.id,
            subtitle_track_variant_for_stream(stream),
        )
        .await?;
        Ok(true)
    }
}
