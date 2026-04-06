use crate::{
    AppState,
    auth::{
        AuthError, create_session_for_user, extractors::pending_user_by_invite_code,
        get_set_cookie_headers_for_session,
    },
    entities::users::{self},
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{Json, debug_handler, extract::State, http::HeaderMap, response::IntoResponse};
use reqwest::header::SET_COOKIE;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct LoginInput {
    username: String,
    password: String,
}

pub async fn find_pending_invite_user(
    pool: &DatabaseConnection,
    invite_code: &str,
) -> Result<Option<users::Model>, sea_orm::DbErr> {
    pending_user_by_invite_code(pool, invite_code).await
}

#[debug_handler]
pub async fn post_login(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(body): Json<LoginInput>,
) -> Result<impl IntoResponse, AuthError> {
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(body.username))
        .one(&state.pool)
        .await
        .map_err(|_| AuthError::UnknownUserOrWrongCredentials)?
        .ok_or(AuthError::UnknownUserOrWrongCredentials)?;

    let Some(password_hash) = user.password_hash else {
        // user is an invite user and does not have a password etc setup
        return Err(AuthError::UserStillPending);
    };

    let parsed_hash = PasswordHash::new(&password_hash).unwrap();
    let argon2 = Argon2::default();
    let result = argon2.verify_password(body.password.as_bytes(), &parsed_hash);
    if result.is_err() {
        return Err(AuthError::UnknownUserOrWrongCredentials);
    }

    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let session_id = create_session_for_user(&state.pool, &user.id, user_agent).await?;
    let cookie = get_set_cookie_headers_for_session(user.id.clone(), session_id)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        cookie.parse().map_err(|_| AuthError::InternalError)?,
    );

    Ok((headers, user.id.to_string()).into_response())
}
