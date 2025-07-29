use crate::{
    auth::RequestAuth,
    config::get_config,
    entities::users::{self},
    error::AppError,
    hls::{profiles::TranscodingProfile, segmenter::Segmenter},
};
use async_graphql::{Schema, http::GraphiQLSource};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Json, Router,
    extract::{FromRef, State},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait};
use serde::Serialize;
use serde_json::json;
use sqlx::sqlite::{
    SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous,
};
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, atomic::AtomicI64},
    time::Duration,
};
use tokio::sync::Mutex;

mod auth;
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
        .route("/api/login", post(auth::post_login))
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
