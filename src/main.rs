use crate::{
    config::get_config,
    entities::{
        sessions,
        users::{self, Permissions},
    },
    error::AppError,
    hls::{profiles::TranscodingProfile, segmenter::Segmenter},
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use async_graphql::{Context, Guard, Schema, http::GraphiQLSource};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Json, Router, debug_handler,
    extract::{FromRef, FromRequestParts, OptionalFromRequestParts, State},
    http::HeaderMap,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use cookie::{Cookie, SameSite};
use rand::RngCore;
use reqwest::{StatusCode, header::SET_COOKIE};
use sea_orm::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::sqlite::{
    SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous,
};
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicI64, Ordering},
    },
    time::Duration,
};
use tokio::sync::Mutex;

mod config;
mod entities;
mod error;
mod ffmpeg;
mod graphql;
mod hls;
mod matcher;
mod proxy;
mod scanner;
mod tmdb;

type AppSchema =
    Schema<graphql::query::Query, graphql::mutation::Mutation, async_graphql::EmptySubscription>;

pub enum RequestAuthSource {
    Session,
    SetupToken,
}

struct RequestAuth {
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
}

#[derive(Clone, FromRef)]
struct AppState {
    segmenters: Arc<Mutex<HashMap<String, Arc<Segmenter>>>>,
    profiles: Vec<Arc<Box<dyn TranscodingProfile + Send + Sync>>>,
    pool: DatabaseConnection,
    schema: Arc<AppSchema>,
    setup_code: u32,
    last_setup_code_attempt: Arc<AtomicI64>,
}

async fn get_graphql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/api/graphql").finish())
}

async fn post_graphql(
    State(state): State<AppState>,
    auth: RequestAuth,
    req: GraphQLRequest,
) -> GraphQLResponse {
    state
        .schema
        .execute(req.into_inner().data(auth))
        .await
        .into()
}

#[derive(Deserialize)]
pub struct LoginInput {
    username: String,
    password: String,
}

#[debug_handler]
async fn post_login(
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

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InitState {
    Login,
    CreateFirstUser,
    Ready,
}

async fn get_init_state(
    State(state): State<AppState>,
    auth: Option<RequestAuth>,
) -> Result<impl IntoResponse, AppError> {
    if auth.is_none() {
        let user_count = users::Entity::find().count(&state.pool).await?;
        let has_first_user = user_count > 0;
        if has_first_user {
            // the user is not signed in and a first user does exist. the user has to sign in to
            // do anything.
            return Ok(Json(json!({
                "state": InitState::Login
            })));
        } else {
            return Ok(Json(json!({
                "state": InitState::CreateFirstUser
            })));
        }
    }

    // server is setup and user can do things.
    return Ok(Json(json!({
        "state": InitState::Ready
    })));
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    ffmpeg::ensure_ffmpeg().await.unwrap();

    let db_path = get_config().data_dir.join("data.db");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(300))
        .connect_with(
            // https://briandouglas.ie/sqlite-defaults/
            SqliteConnectOptions::from_str(db_path.to_string_lossy().as_ref())
                .expect("Failed to parse SQLite path")
                .journal_mode(SqliteJournalMode::Wal)
                .synchronous(SqliteSynchronous::Normal)
                .busy_timeout(Duration::from_secs(30))
                .foreign_keys(true)
                .auto_vacuum(SqliteAutoVacuum::Incremental)
                .pragma("cache_size", "-10000")
                .pragma("temp_store", "MEMORY")
                .create_if_missing(true)
                .page_size(8192),
        )
        .await
        .expect("Failed to connect to SQLite");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let pool = DatabaseConnection::from(pool);
    let scanner_pool = pool.clone();
    tokio::spawn(async move {
        scanner::start_scanner(scanner_pool).await;
    });

    let matcher_pool = pool.clone();
    tokio::spawn(async move {
        matcher::worker::start_matcher(matcher_pool)
            .await
            .expect("matcher failed");
    });

    let schema: AppSchema = Schema::build(
        graphql::query::Query,
        graphql::mutation::Mutation,
        async_graphql::EmptySubscription,
    )
    .limit_depth(5)
    .limit_complexity(100)
    .limit_directives(5)
    .data(pool.clone())
    .finish();

    // write the schema to a file in dev
    #[cfg(debug_assertions)]
    {
        let schema_str = schema.sdl();
        std::fs::write("schema.gql", schema_str).unwrap();
    }

    let setup_code = rand::random_range(100_000..=999_999);
    let user_count = users::Entity::find().count(&pool).await.unwrap();
    if user_count == 0 {
        let setup_code_str = format!("{:06}", setup_code);
        tracing::info!(
            "your setup code is '{}-{}'",
            &setup_code_str[..3],
            &setup_code_str[3..]
        );
    }

    #[allow(unused_mut)]
    let mut app = Router::new()
        .nest("/api/hls", hls::get_hls_router())
        .nest("/api/image-proxy", proxy::get_proxy_router())
        .route("/api/graphql", get(get_graphql).post(post_graphql))
        .route("/api/init", get(get_init_state))
        .route("/api/login", post(post_login))
        .with_state(AppState {
            segmenters: Arc::new(Mutex::new(HashMap::new())),
            profiles: hls::get_profiles(),
            pool: pool,
            schema: Arc::new(schema),
            setup_code,
            last_setup_code_attempt: Arc::new(AtomicI64::new(0)),
        });

    #[cfg(feature = "static")]
    {
        use tower_http::services::{ServeDir, ServeFile};

        let static_path = std::env::var("LYRA_STATIC_PATH")
            .expect("LYRA_STATIC_PATH not set with static feature");
        let index_path = static_path.clone() + "/index.html";
        let serve_dir = ServeDir::new(static_path)
            .not_found_service(ServeFile::new(index_path))
            .precompressed_gzip();

        app = app.fallback_service(serve_dir)
    }

    let config = get_config();
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
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
