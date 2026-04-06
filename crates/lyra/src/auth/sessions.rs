use crate::{
    auth::{AuthError, LAST_SEEN_UPDATE_INTERVAL_SECONDS, TOKEN_EXPIRY_DAYS},
    entities::user_sessions,
    ids, signer,
};
use chrono::Utc;
use cookie::{Cookie, SameSite};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, Set};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub async fn create_session_for_user(
    pool: &DatabaseConnection,
    user_id: &str,
    user_agent: Option<String>,
) -> Result<String, AuthError> {
    let session_id = ids::generate_ulid();
    let now = Utc::now().timestamp();

    user_sessions::Entity::insert(user_sessions::ActiveModel {
        id: Set(session_id.clone()),
        user_id: Set(user_id.to_string()),
        created_at: Set(now),
        user_agent: Set(user_agent),
        last_seen_at: Set(now),
    })
    .exec(pool)
    .await
    .map_err(|_| AuthError::InternalError)?;

    Ok(session_id)
}

pub fn get_set_cookie_headers_for_session(
    user_id: String,
    session_id: String,
) -> Result<String, AuthError> {
    let expiry = Duration::from_hours(24 * TOKEN_EXPIRY_DAYS);
    let payload = SessionTokenPayload {
        user_id,
        session_id,
    };

    let session_token = signer::sign(payload, expiry).map_err(|_| AuthError::InternalError)?;
    let cookie = Cookie::build(("session", session_token))
        .path("/api")
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(cookie::time::Duration::hours(
            (24 * TOKEN_EXPIRY_DAYS) as i64,
        ))
        .build()
        .to_string();

    Ok(cookie)
}

pub(super) async fn maybe_touch_last_seen(
    pool: &DatabaseConnection,
    session: &user_sessions::Model,
) -> Result<(), AuthError> {
    let now = Utc::now().timestamp();
    let should_update = now - session.last_seen_at >= LAST_SEEN_UPDATE_INTERVAL_SECONDS;
    if !should_update {
        return Ok(());
    }

    let mut session = session.clone().into_active_model();
    session.last_seen_at = Set(now);
    session
        .update(pool)
        .await
        .map_err(|_| AuthError::InternalError)?;

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub(super) struct SessionTokenPayload {
    pub(super) user_id: String,
    pub(super) session_id: String,
}
