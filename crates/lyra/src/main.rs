use crate::{
    config::get_config,
    hls::{profiles::TranscodingProfile, segmenter::Segmenter},
};
use async_graphql::{Schema, http::GraphiQLSource};
use async_graphql_axum::GraphQL;
use axum::{
    Router,
    response::{Html, IntoResponse},
    routing::get,
};
use sea_orm::DatabaseConnection;
use sqlx::sqlite::{
    SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous,
};
use std::{collections::HashMap, str::FromStr, sync::Arc, time::Duration};
use tokio::sync::Mutex;

mod api;
mod config;
mod entities;
mod error;
mod ffmpeg;
mod hls;
mod matcher;
mod proxy;
mod scanner;
mod tmdb;

#[derive(Clone)]
struct AppState {
    segmenters: Arc<Mutex<HashMap<String, Arc<Segmenter>>>>,
    profiles: Vec<Arc<Box<dyn TranscodingProfile + Send + Sync>>>,
    pool: DatabaseConnection,
}

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/api/graphql").finish())
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

    sqlx::migrate!("../../migrations")
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
        matcher::start_matcher(matcher_pool)
            .await
            .expect("matcher failed");
    });

    let graphql = {
        let schema = Schema::build(
            api::Query,
            async_graphql::EmptyMutation,
            async_graphql::EmptySubscription,
        )
        .data(pool.clone())
        .finish();

        // write the schema to a file in dev
        #[cfg(debug_assertions)]
        {
            use std::path::PathBuf;
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let schema_path = manifest_dir.join("schema.gql");
            let schema_str = schema.sdl();
            std::fs::write(schema_path, schema_str).unwrap();
        }

        Router::new().route("/", get(graphiql).post_service(GraphQL::new(schema)))
    };

    let app = Router::new()
        .nest("/api/hls", hls::get_hls_router())
        .nest("/api/image-proxy", proxy::get_proxy_router())
        .nest("/api/graphql", graphql)
        .with_state(AppState {
            segmenters: Arc::new(Mutex::new(HashMap::new())),
            profiles: hls::get_profiles(),
            pool: pool,
        });

    let config = get_config();
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
