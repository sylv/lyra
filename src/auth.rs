use crate::{
    AppState,
    entities::{
        sessions,
        users::{self, Permissions},
    },
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use async_graphql::{Context, Guard};
use axum::{
    Json, debug_handler,
    extract::{FromRef, FromRequestParts, OptionalFromRequestParts, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use cookie::{Cookie, SameSite};
use rand::RngCore;
use reqwest::{StatusCode, header::SET_COOKIE};
use sea_orm::Set;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use serde::Deserialize;
use serde_json::json;
use std::sync::atomic::Ordering;

pub struct RequestAuth {
    user: Option<users::Model>,
    permissions: Permissions,
}

impl RequestAuth {
    pub fn has_permission(&self, permission: Permissions) -> bool {
        self.permissions.contains(permission)
    }

    pub fn get_user_or_err(&self) -> Result<&users::Model, async_graphql::Error> {
        self.user
            .as_ref()
            .ok_or_else(|| async_graphql::Error::new("No user in context".to_string()))
    }

    pub fn get_user(&self) -> Option<&users::Model> {
        self.user.as_ref()
    }
}

impl<S> FromRequestParts<S> for RequestAuth
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);

        let setup_code = parts
            .headers
            .get("x-setup-code")
            .map(|h| h.to_str().unwrap())
            .and_then(|c| c.parse::<u32>().ok());

        if let Some(setup_code) = setup_code {
            let now = Utc::now().timestamp();
            let last_attempt = state.last_setup_code_attempt.load(Ordering::Relaxed);
            if now - last_attempt < 2 {
                return Err(AuthError::TooManyAttempts);
            }

            if setup_code != state.setup_code {
                tracing::warn!("setup code incorrect");
                return Err(AuthError::Unauthenticated);
            }

            let user_count = users::Entity::find()
                .count(&state.pool)
                .await
                .map_err(|_| AuthError::InternalError)?;
            if user_count > 0 {
                tracing::warn!("setup code attempted but user already exists");
                return Err(AuthError::Unauthenticated);
            }

            return Ok(RequestAuth {
                user: None,
                permissions: Permissions::CREATE_USER,
            });
        }

        let cookie_jar = CookieJar::from_headers(&parts.headers);
        let Some(session_id) = cookie_jar.get("session").map(|c| c.value()) else {
            return Err(AuthError::Unauthenticated);
        };

        let Some((session, Some(user))) = sessions::Entity::find_by_id(session_id)
            .find_also_related(users::Entity)
            .one(&state.pool)
            .await
            .map_err(|_| AuthError::InternalError)?
        else {
            return Err(AuthError::Unauthenticated);
        };

        if session.expires_at < Utc::now().timestamp() {
            return Err(AuthError::SessionExpired);
        }

        let permissions = Permissions::from_bits_truncate(user.permissions);
        Ok(RequestAuth {
            user: Some(user),
            permissions,
        })
    }
}

impl<S> OptionalFromRequestParts<S> for RequestAuth
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        match <RequestAuth as FromRequestParts<S>>::from_request_parts(parts, state).await {
            Ok(auth) => Ok(Some(auth)),
            Err(AuthError::InternalError) => Err(AuthError::InternalError),
            Err(_) => Ok(None),
        }
    }
}

pub enum AuthError {
    UnknownUserOrWrongCredentials,
    SessionExpired,
    InsufficientPermissions,
    Unauthenticated,
    TooManyAttempts,
    InternalError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::SessionExpired => (StatusCode::UNAUTHORIZED, "Session expired"),
            AuthError::Unauthenticated => (StatusCode::UNAUTHORIZED, "Unauthenticated"),
            AuthError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
            AuthError::UnknownUserOrWrongCredentials => (
                StatusCode::UNAUTHORIZED,
                "Unknown user or wrong credentials",
            ),
            AuthError::InsufficientPermissions => {
                (StatusCode::FORBIDDEN, "Insufficient permissions")
            }
            AuthError::TooManyAttempts => (StatusCode::TOO_MANY_REQUESTS, "Too many attempts"),
        };

        let body = Json(json!({
            "status_code": status.as_u16(),
            "error_message": error_message,
        }));

        (status, body).into_response()
    }
}

pub struct PermissionGuard(Permissions);

impl PermissionGuard {
    pub fn new(permissions: Permissions) -> Self {
        Self(permissions)
    }
}

impl Guard for PermissionGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<(), async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        if !auth.has_permission(self.0) {
            return Err(async_graphql::Error::new("Insufficient permissions"));
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct LoginInput {
    username: String,
    password: String,
}

#[debug_handler]
pub async fn post_login(
    State(state): State<AppState>,
    Json(body): Json<LoginInput>,
) -> Result<impl IntoResponse, AuthError> {
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(body.username))
        .one(&state.pool)
        .await
        .map_err(|_| AuthError::UnknownUserOrWrongCredentials)?
        .ok_or(AuthError::UnknownUserOrWrongCredentials)?;

    let parsed_hash = PasswordHash::new(&user.password_hash).unwrap();
    let argon2 = Argon2::default();
    let result = argon2.verify_password(body.password.as_bytes(), &parsed_hash);
    if result.is_err() {
        return Err(AuthError::UnknownUserOrWrongCredentials);
    }

    let session_expiry = 2 * 7 * 24 * 60 * 60; // 2 weeks
    let session_id = {
        let mut bytes = [0u8; 16];
        rand::rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    };

    sessions::Entity::insert(sessions::ActiveModel {
        id: Set(session_id.clone()),
        user_id: Set(user.id.clone()),
        created_at: Set(Utc::now().timestamp()),
        expires_at: Set(Utc::now().timestamp() + session_expiry),
        last_seen_at: Set(Utc::now().timestamp()),
    })
    .exec(&state.pool)
    .await
    .map_err(|_| AuthError::InternalError)?;

    // the session expiry is extended when its used, so we want the cookie
    // to last longer than the session expiry.
    let cookie = Cookie::build(("session", session_id))
        .path("/api")
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(cookie::time::Duration::days(365))
        .build();

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.to_string().parse().unwrap());

    Ok((headers, user.id.to_string()).into_response())
}
