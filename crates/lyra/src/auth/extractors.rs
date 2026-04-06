use crate::{
    AppState,
    auth::{
        AuthError, TOKEN_REFRESH_WITHIN_EXPIRY_DAYS,
        sessions::{
            SessionTokenPayload, get_set_cookie_headers_for_session, maybe_touch_last_seen,
        },
    },
    entities::{
        user_sessions,
        users::{self, UserPerms},
    },
    signer,
};
use axum::{
    extract::{FromRef, FromRequestParts, OptionalFromRequestParts},
    http::request::Parts,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use reqwest::header::SET_COOKIE;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};
use std::{sync::atomic::Ordering, time::Duration};

pub struct RequestAuth {
    user: Option<users::Model>,
    permissions: UserPerms,
    is_setup: bool,
}

impl RequestAuth {
    pub fn has_permission(&self, permission: UserPerms) -> bool {
        self.permissions.contains(UserPerms::ADMIN) || self.permissions.contains(permission)
    }

    pub fn get_user_or_err(&self) -> Result<&users::Model, async_graphql::Error> {
        self.user
            .as_ref()
            .ok_or_else(|| AuthError::Unauthenticated.into())
    }

    pub fn get_user(&self) -> Option<&users::Model> {
        self.user.as_ref()
    }

    pub fn is_setup(&self) -> bool {
        self.is_setup
    }
}

impl<S> FromRequestParts<S> for RequestAuth
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
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
                tracing::warn!("setup code attempted too soon after last attempt");
                return Err(AuthError::TooManyAttempts);
            }

            if setup_code != state.setup_code {
                tracing::warn!("setup code incorrect");
                state.last_setup_code_attempt.store(now, Ordering::Relaxed);
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

            return Ok(Self {
                user: None,
                permissions: UserPerms::ADMIN,
                is_setup: true,
            });
        }

        let cookie_jar = CookieJar::from_headers(&parts.headers);
        let Some(session_token) = cookie_jar.get("session").map(|c| c.value()) else {
            return Err(AuthError::Unauthenticated);
        };

        let (expires_in, payload) = signer::verify::<SessionTokenPayload>(session_token)
            .map_err(|_| AuthError::Unauthenticated)?;

        let Some((session, Some(user))) = user_sessions::Entity::find()
            .filter(user_sessions::Column::Id.eq(payload.session_id))
            .find_also_related(users::Entity)
            .one(&state.pool)
            .await
            .map_err(|_| AuthError::InternalError)?
        else {
            return Err(AuthError::Unauthenticated);
        };

        if expires_in <= Duration::from_hours(24 * TOKEN_REFRESH_WITHIN_EXPIRY_DAYS) {
            tracing::info!("refreshing session token for user {}", user.id);
            let cookie = get_set_cookie_headers_for_session(user.id.clone(), session.id.clone())?;
            parts.headers.insert(
                SET_COOKIE,
                cookie.parse().map_err(|_| AuthError::InternalError)?,
            );
        }

        maybe_touch_last_seen(&state.pool, &session).await?;

        let permissions = UserPerms::from_bits_truncate(user.permissions as u32);
        Ok(Self {
            user: Some(user),
            permissions,
            is_setup: false,
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
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        match <RequestAuth as FromRequestParts<S>>::from_request_parts(parts, state).await {
            Ok(auth) => Ok(Some(auth)),
            Err(AuthError::InternalError) => Err(AuthError::InternalError),
            Err(_) => Ok(None),
        }
    }
}

/// Unlike RequestAuth, only checks for a valid, non-expired session token.
/// It does not check that the session is still valid (ie, was not deleted) or do anything extra.
pub struct LazyRequestAuth;

impl<S> FromRequestParts<S> for LazyRequestAuth
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let cookie_jar = CookieJar::from_headers(&parts.headers);
        let Some(session_token) = cookie_jar.get("session").map(|c| c.value()) else {
            return Err(AuthError::Unauthenticated);
        };

        signer::verify::<SessionTokenPayload>(session_token)
            .map_err(|_| AuthError::Unauthenticated)?;

        Ok(Self)
    }
}

pub(super) fn get_user_or_auth_error(auth: &RequestAuth) -> Result<&users::Model, AuthError> {
    auth.user.as_ref().ok_or(AuthError::Unauthenticated)
}

pub(super) async fn pending_user_by_invite_code(
    pool: &DatabaseConnection,
    invite_code: &str,
) -> Result<Option<users::Model>, sea_orm::DbErr> {
    users::Entity::find()
        .filter(users::Column::InviteCode.eq(invite_code))
        .filter(users::Column::PasswordHash.is_null())
        .one(pool)
        .await
}
