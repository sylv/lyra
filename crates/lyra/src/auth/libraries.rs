use crate::{
    auth::{AuthError, RequestAuth, extractors::get_user_or_auth_error},
    entities::{library_users, users::UserPerms},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};

// library visibility is either global via VIEW_ALL_LIBRARIES/admin or explicit via library_users.
pub async fn accessible_library_ids(
    pool: &DatabaseConnection,
    auth: &RequestAuth,
) -> Result<Option<Vec<String>>, AuthError> {
    if auth.has_permission(UserPerms::VIEW_ALL_LIBRARIES) {
        return Ok(None);
    }

    let user = get_user_or_auth_error(auth)?;
    let library_ids = library_users::Entity::find()
        .filter(library_users::Column::UserId.eq(user.id.clone()))
        .select_only()
        .column(library_users::Column::LibraryId)
        .into_tuple::<String>()
        .all(pool)
        .await
        .map_err(|_| AuthError::InternalError)?;

    Ok(Some(library_ids))
}

pub async fn ensure_library_access(
    pool: &DatabaseConnection,
    auth: &RequestAuth,
    library_id: &str,
) -> Result<(), AuthError> {
    let Some(library_ids) = accessible_library_ids(pool, auth).await? else {
        return Ok(());
    };

    if library_ids.iter().any(|candidate| candidate == library_id) {
        return Ok(());
    }

    Err(AuthError::InsufficientPermissions)
}
