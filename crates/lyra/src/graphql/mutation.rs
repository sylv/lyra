use crate::RequestAuth;
use crate::auth::{
    AuthenticatedGuard, PermissionGuard, accessible_library_ids, create_session_for_user,
    ensure_library_access, find_pending_invite_user,
};
use crate::content_update::CONTENT_UPDATE;
use crate::entities::collections::{CollectionResolverKind, CollectionVisibility};
use crate::entities::users::UserPerms;
use crate::entities::{
    collection_items, collections, files, libraries, library_users, node_files, nodes,
    user_sessions, users, watch_progress,
};
use crate::graphql::properties::TrackDispositionPreference;
use crate::graphql::query::{NodeFilter, collection_editable_by_user, is_watchlist_collection};
use crate::ids::{self, new_invite_code};
use crate::import::watch_state_import;
use crate::watch_session::{
    WatchSessionActionInput, WatchSessionBeacon, WatchSessionHeartbeatInput, WatchSessionRegistry,
};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use async_graphql::{Context, InputObject, Object, SimpleObject};
use chrono::Utc;
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

fn normalize_library_ids(mut library_ids: Vec<String>) -> Vec<String> {
    library_ids.sort();
    library_ids.dedup();
    library_ids
}

fn normalize_collection_name(name: String) -> Result<String, async_graphql::Error> {
    let name = name.trim();
    if name.is_empty() {
        return Err(async_graphql::Error::new("Collection name is required"));
    }

    Ok(name.to_string())
}

fn ensure_collection_visibility_allowed(
    auth: &RequestAuth,
    visibility: CollectionVisibility,
) -> Result<(), async_graphql::Error> {
    if visibility == CollectionVisibility::Public && !auth.has_permission(UserPerms::ADMIN) {
        return Err(async_graphql::Error::new(
            "Only admins can create or publish public collections",
        ));
    }

    Ok(())
}

fn ensure_collection_is_user_editable(
    collection: &collections::Model,
    auth: &RequestAuth,
) -> Result<(), async_graphql::Error> {
    let user = auth.get_user_or_err()?;
    if !collection_editable_by_user(collection, &user.id, auth.has_permission(UserPerms::ADMIN)) {
        return Err(async_graphql::Error::new("Collection is not editable"));
    }

    Ok(())
}

async fn ensure_node_accessible(
    pool: &DatabaseConnection,
    auth: &RequestAuth,
    node_id: &str,
) -> Result<nodes::Model, async_graphql::Error> {
    let mut query = nodes::Entity::find().filter(nodes::Column::Id.eq(node_id.to_string()));
    if let Some(visible_library_ids) = accessible_library_ids(pool, auth)
        .await
        .map_err(async_graphql::Error::from)?
    {
        if visible_library_ids.is_empty() {
            return Err(async_graphql::Error::new("Node not found"));
        }
        query = query.filter(nodes::Column::LibraryId.is_in(visible_library_ids));
    }

    query
        .one(pool)
        .await?
        .ok_or_else(|| async_graphql::Error::new("Node not found"))
}

async fn ensure_watchlist_collection(
    pool: &DatabaseConnection,
    user_id: &str,
) -> Result<collections::Model, async_graphql::Error> {
    if let Some(collection) = collections::Entity::find_by_id(user_id.to_string())
        .one(pool)
        .await?
    {
        let mut active = collection.into_active_model();
        active.name = Set("Watchlist".to_string());
        active.description = Set(Some("Your saved movies, series, and episodes".to_string()));
        active.created_by_id = Set(Some(user_id.to_string()));
        active.visibility = Set(CollectionVisibility::Private);
        active.resolver_kind = Set(CollectionResolverKind::Manual);
        active.kind = Set(None);
        active.filter_json = Set(None);
        active.show_on_home = Set(false);
        active.home_position = Set(0);
        active.pinned = Set(false);
        active.pinned_position = Set(0);
        return Ok(active.update(pool).await?);
    }

    Ok(collections::Entity::insert(collections::ActiveModel {
        id: Set(user_id.to_string()),
        name: Set("Watchlist".to_string()),
        description: Set(Some("Your saved movies, series, and episodes".to_string())),
        created_by_id: Set(Some(user_id.to_string())),
        visibility: Set(CollectionVisibility::Private),
        resolver_kind: Set(CollectionResolverKind::Manual),
        kind: Set(None),
        filter_json: Set(None),
        show_on_home: Set(false),
        home_position: Set(0),
        pinned: Set(false),
        pinned_position: Set(0),
        ..Default::default()
    })
    .exec_with_returning(pool)
    .await?)
}

// keep user updates atomic so permission flips and explicit library assignments can't drift apart.
async fn sync_user_library_access<C>(
    db: &C,
    user_id: &str,
    library_ids: &[String],
) -> Result<(), async_graphql::Error>
where
    C: sea_orm::ConnectionTrait,
{
    if !library_ids.is_empty() {
        let found_library_count = libraries::Entity::find()
            .filter(libraries::Column::Id.is_in(library_ids.iter().cloned()))
            .count(db)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        if found_library_count != library_ids.len() as u64 {
            return Err(async_graphql::Error::new(
                "One or more libraries were not found",
            ));
        }
    }

    library_users::Entity::delete_many()
        .filter(library_users::Column::UserId.eq(user_id.to_string()))
        .exec(db)
        .await
        .map_err(|e| async_graphql::Error::new(e.to_string()))?;

    if library_ids.is_empty() {
        return Ok(());
    }

    library_users::Entity::insert_many(library_ids.iter().cloned().map(|library_id| {
        library_users::ActiveModel {
            library_id: Set(library_id),
            user_id: Set(user_id.to_string()),
        }
    }))
    .exec(db)
    .await
    .map_err(|e| async_graphql::Error::new(e.to_string()))?;

    Ok(())
}

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

        if let Some(invite_code) = invite_code
            .as_deref()
            .map(str::trim)
            .filter(|invite_code| !invite_code.is_empty())
        {
            let blank_user = find_pending_invite_user(pool, invite_code)
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
            let cookie = create_session_for_user(pool, &user.id)
                .await
                .map_err(|e| -> async_graphql::Error { e.into() })?;
            ctx.insert_http_header("Set-Cookie", cookie);
            CONTENT_UPDATE.emit();
            Ok(user)
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
                let cookie = create_session_for_user(pool, &user.id)
                    .await
                    .map_err(|e| -> async_graphql::Error { e.into() })?;
                ctx.insert_http_header("Set-Cookie", cookie);
            }

            CONTENT_UPDATE.emit();
            Ok(user)
        }
    }

    #[graphql(guard = PermissionGuard::new(UserPerms::ADMIN))]
    pub async fn create_user_invite(
        &self,
        ctx: &Context<'_>,
        username: String,
        permissions: u32,
        library_ids: Vec<String>,
    ) -> Result<users::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let username = normalize_username(username)?;
        let invite_code = new_invite_code();
        let library_ids = normalize_library_ids(library_ids);
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

        sync_user_library_access(&txn, &user.id, &library_ids).await?;

        txn.commit()
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        CONTENT_UPDATE.emit();
        Ok(user)
    }

    #[graphql(guard = PermissionGuard::new(UserPerms::ADMIN))]
    pub async fn update_user(
        &self,
        ctx: &Context<'_>,
        user_id: String,
        username: String,
        permissions: u32,
        library_ids: Vec<String>,
    ) -> Result<users::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let username = normalize_username(username)?;
        let library_ids = normalize_library_ids(library_ids);
        let existing_user = users::Entity::find_by_id(user_id)
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("User not found".to_string()))?;
        let existing_library_ids = normalize_library_ids(
            library_users::Entity::find()
                .filter(library_users::Column::UserId.eq(existing_user.id.clone()))
                .select_only()
                .column(library_users::Column::LibraryId)
                .into_tuple::<String>()
                .all(pool)
                .await
                .map_err(|e| async_graphql::Error::new(e.to_string()))?,
        );

        if auth
            .get_user()
            .is_some_and(|current_user| current_user.id == existing_user.id)
            && (existing_user.permissions != permissions as i64
                || existing_library_ids != library_ids)
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
        sync_user_library_access(&txn, &user.id, &library_ids).await?;
        txn.commit()
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        CONTENT_UPDATE.emit();
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

        CONTENT_UPDATE.emit();
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

        CONTENT_UPDATE.emit();
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
        ensure_library_access(pool, auth, &file.library_id)
            .await
            .map_err(|_| async_graphql::Error::new("File not found"))?;

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

        CONTENT_UPDATE.emit();
        Ok(updated_rows)
    }

    pub async fn import_watch_states(
        &self,
        ctx: &Context<'_>,
        input: ImportWatchStatesInput,
    ) -> Result<ImportWatchStatesResult, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;

        let request = watch_state_import::ImportWatchStatesRequest {
            user_id: user.id.clone(),
            accessible_library_ids: accessible_library_ids(pool, auth)
                .await
                .map_err(|error| -> async_graphql::Error { error.into() })?,
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

        if result.imported > 0 {
            CONTENT_UPDATE.emit();
        }

        Ok(result.into())
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

        CONTENT_UPDATE.emit();
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

        CONTENT_UPDATE.emit();
        Ok(library)
    }

    #[graphql(guard = AuthenticatedGuard::new())]
    pub async fn create_collection(
        &self,
        ctx: &Context<'_>,
        name: String,
        description: Option<String>,
        visibility: CollectionVisibility,
        resolver_kind: CollectionResolverKind,
        filter: Option<NodeFilter>,
        show_on_home: Option<bool>,
        home_position: Option<i64>,
        pinned: Option<bool>,
        pinned_position: Option<i64>,
    ) -> Result<collections::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;
        ensure_collection_visibility_allowed(auth, visibility)?;

        if resolver_kind == CollectionResolverKind::Filter && filter.is_none() {
            return Err(async_graphql::Error::new(
                "Filter collections require a filter definition",
            ));
        }

        let collection = collections::Entity::insert(collections::ActiveModel {
            id: Set(ids::generate_ulid()),
            name: Set(normalize_collection_name(name)?),
            description: Set(description.and_then(|value| {
                let trimmed = value.trim().to_string();
                (!trimmed.is_empty()).then_some(trimmed)
            })),
            created_by_id: Set(Some(user.id.clone())),
            visibility: Set(visibility),
            resolver_kind: Set(resolver_kind),
            kind: Set(None),
            filter_json: Set(filter.as_ref().map(serde_json::to_vec).transpose()?),
            show_on_home: Set(show_on_home.unwrap_or(false)),
            home_position: Set(home_position.unwrap_or(0)),
            pinned: Set(pinned.unwrap_or(false)),
            pinned_position: Set(pinned_position.unwrap_or(0)),
            ..Default::default()
        })
        .exec_with_returning(pool)
        .await?;

        CONTENT_UPDATE.emit();
        Ok(collection)
    }

    #[graphql(guard = AuthenticatedGuard::new())]
    pub async fn update_collection(
        &self,
        ctx: &Context<'_>,
        collection_id: String,
        name: String,
        description: Option<String>,
        visibility: CollectionVisibility,
        resolver_kind: CollectionResolverKind,
        filter: Option<NodeFilter>,
        show_on_home: bool,
        home_position: i64,
        pinned: bool,
        pinned_position: i64,
    ) -> Result<collections::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        ensure_collection_visibility_allowed(auth, visibility)?;

        let existing = collections::Entity::find_by_id(collection_id)
            .one(pool)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Collection not found"))?;
        ensure_collection_is_user_editable(&existing, auth)?;

        if resolver_kind == CollectionResolverKind::Filter && filter.is_none() {
            return Err(async_graphql::Error::new(
                "Filter collections require a filter definition",
            ));
        }

        let mut active = existing.into_active_model();
        active.name = Set(normalize_collection_name(name)?);
        active.description = Set(description.and_then(|value| {
            let trimmed = value.trim().to_string();
            (!trimmed.is_empty()).then_some(trimmed)
        }));
        active.visibility = Set(visibility);
        active.resolver_kind = Set(resolver_kind);
        active.filter_json = Set(filter.as_ref().map(serde_json::to_vec).transpose()?);
        active.show_on_home = Set(show_on_home);
        active.home_position = Set(home_position);
        active.pinned = Set(pinned);
        active.pinned_position = Set(pinned_position);

        let collection = active.update(pool).await?;
        CONTENT_UPDATE.emit();
        Ok(collection)
    }

    #[graphql(guard = AuthenticatedGuard::new())]
    pub async fn delete_collection(
        &self,
        ctx: &Context<'_>,
        collection_id: String,
    ) -> Result<bool, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let Some(collection) = collections::Entity::find_by_id(collection_id)
            .one(pool)
            .await?
        else {
            return Ok(false);
        };
        ensure_collection_is_user_editable(&collection, auth)?;
        collections::Entity::delete_by_id(collection.id)
            .exec(pool)
            .await?;
        CONTENT_UPDATE.emit();
        Ok(true)
    }

    #[graphql(guard = AuthenticatedGuard::new())]
    pub async fn add_node_to_collection(
        &self,
        ctx: &Context<'_>,
        collection_id: String,
        node_id: String,
    ) -> Result<collections::Model, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let collection = collections::Entity::find_by_id(collection_id)
            .one(pool)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Collection not found"))?;
        ensure_collection_is_user_editable(&collection, auth)?;

        if collection.resolver_kind != CollectionResolverKind::Manual {
            return Err(async_graphql::Error::new(
                "Only manual collections can accept direct node additions",
            ));
        }

        if is_watchlist_collection(&collection) {
            return Err(async_graphql::Error::new(
                "Use the watchlist actions to manage your watchlist",
            ));
        }

        let _node = ensure_node_accessible(pool, auth, &node_id).await?;
        let next_position = collection_items::Entity::find()
            .filter(collection_items::Column::CollectionId.eq(collection.id.clone()))
            .count(pool)
            .await? as i64;

        collection_items::Entity::insert(collection_items::ActiveModel {
            collection_id: Set(collection.id.clone()),
            node_id: Set(node_id),
            position: Set(next_position),
            ..Default::default()
        })
        .on_conflict(
            OnConflict::columns([
                collection_items::Column::CollectionId,
                collection_items::Column::NodeId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(pool)
        .await?;

        CONTENT_UPDATE.emit();
        Ok(collection)
    }

    #[graphql(guard = AuthenticatedGuard::new())]
    pub async fn add_node_to_watchlist(
        &self,
        ctx: &Context<'_>,
        node_id: String,
    ) -> Result<bool, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;
        let collection = ensure_watchlist_collection(pool, &user.id).await?;
        let _node = ensure_node_accessible(pool, auth, &node_id).await?;
        let next_position = collection_items::Entity::find()
            .filter(collection_items::Column::CollectionId.eq(collection.id.clone()))
            .count(pool)
            .await? as i64;

        collection_items::Entity::insert(collection_items::ActiveModel {
            collection_id: Set(collection.id),
            node_id: Set(node_id),
            position: Set(next_position),
            ..Default::default()
        })
        .on_conflict(
            OnConflict::columns([
                collection_items::Column::CollectionId,
                collection_items::Column::NodeId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(pool)
        .await?;

        CONTENT_UPDATE.emit();
        Ok(true)
    }

    #[graphql(guard = AuthenticatedGuard::new())]
    pub async fn remove_node_from_watchlist(
        &self,
        ctx: &Context<'_>,
        node_id: String,
    ) -> Result<bool, async_graphql::Error> {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;

        collection_items::Entity::delete_many()
            .filter(collection_items::Column::CollectionId.eq(user.id.clone()))
            .filter(collection_items::Column::NodeId.eq(node_id))
            .exec(pool)
            .await?;

        CONTENT_UPDATE.emit();
        Ok(true)
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

    pub async fn set_preferred_subtitle(
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
        active.preferred_subtitle_language = Set(language);
        active.preferred_subtitle_disposition = Set(disposition.map(|d| d.as_str().to_string()));

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

        CONTENT_UPDATE.emit();
        Ok(true)
    }

    #[graphql(guard = AuthenticatedGuard::new())]
    pub async fn leave_watch_session(
        &self,
        ctx: &Context<'_>,
        session_id: String,
        player_id: String,
    ) -> Result<bool, async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        let registry = ctx.data::<WatchSessionRegistry>()?;
        registry.leave_session(auth, &session_id, &player_id).await
    }

    #[graphql(guard = AuthenticatedGuard::new())]
    pub async fn watch_session_heartbeat(
        &self,
        ctx: &Context<'_>,
        input: WatchSessionHeartbeatInput,
    ) -> Result<WatchSessionBeacon, async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        let registry = ctx.data::<WatchSessionRegistry>()?;
        registry.heartbeat(auth, input).await
    }

    #[graphql(guard = AuthenticatedGuard::new())]
    pub async fn watch_session_action(
        &self,
        ctx: &Context<'_>,
        input: WatchSessionActionInput,
    ) -> Result<WatchSessionBeacon, async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        let registry = ctx.data::<WatchSessionRegistry>()?;
        registry.apply_action(auth, input).await
    }
}
