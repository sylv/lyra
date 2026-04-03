use crate::{
    auth::RequestAuth,
    cleanup::start_cleanup_worker,
    collections::reconcile_system_collections,
    config::get_config,
    content_update::CONTENT_UPDATE,
    entities::{
        jobs as jobs_entity, libraries,
        users::{self},
    },
    error::AppError,
    jobs::{JobSemaphore, load_registered_jobs},
    watch_session::WatchSessionRegistry,
};
use anyhow::Context;
use async_graphql::Data;
use async_graphql::dataloader::DataLoader;
use async_graphql::{Schema, http::GraphiQLSource};
use async_graphql_axum::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};
use axum::{
    Json, Router,
    extract::ws::WebSocketUpgrade,
    extract::{FromRef, Query as AxumQuery, State},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use lyra_packager::Package as PackagerState;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};
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
use tokio::sync::{Mutex, Notify};
use tokio::{signal, task::JoinSet};
use tokio_util::sync::CancellationToken;

mod activity;
mod assets;
mod auth;
mod cleanup;
mod collections;
mod config;
mod content_update;
mod entities;
mod error;
mod file_analysis;
mod graphql;
mod hls;
mod ids;
mod import;
mod jobs;
mod json_encoding;
mod metadata;
mod scanner;
mod segment_markers;
mod signer;
mod watch_session;

type AppSchema = Schema<
    graphql::query::Query,
    graphql::mutation::Mutation,
    graphql::subscription::SubscriptionRoot,
>;

#[derive(Clone, FromRef)]
struct AppState {
    signer: signer::Signer,
    pool: DatabaseConnection,
    schema: Arc<AppSchema>,
    job_wake_signal: Arc<Notify>,
    job_semaphore: Arc<JobSemaphore>,
    setup_code: u32,
    last_setup_code_attempt: Arc<AtomicI64>,
    packager_states: Arc<Mutex<HashMap<String, Arc<PackagerState>>>>,
}

async fn get_graphql(_auth: RequestAuth) -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .endpoint("/api/graphql")
            .subscription_endpoint("/api/graphql/ws")
            .finish(),
    )
}

async fn get_graphql_ws(
    ws: WebSocketUpgrade,
    protocol: GraphQLProtocol,
    auth: RequestAuth,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let schema = state.schema.as_ref().clone();
    let mut data = Data::default();
    data.insert(auth);

    ws.protocols(async_graphql::http::ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, schema, protocol)
                .with_data(data)
                .serve()
        })
}

async fn post_graphql(
    State(state): State<AppState>,
    auth: Option<RequestAuth>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let req = req.into_inner();

    match auth {
        Some(auth) => state.schema.execute(req.data(auth)).await.into(),
        None => state.schema.execute(req).await.into(),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InitState {
    Login,
    CreateFirstUser,
    CreateInvitedUser,
    CreateFirstLibrary,
    Ready,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitQuery {
    invite_code: Option<String>,
}

async fn get_init_state(
    State(state): State<AppState>,
    AxumQuery(query): AxumQuery<InitQuery>,
    auth: Option<RequestAuth>,
) -> Result<impl IntoResponse, AppError> {
    if auth.is_none() {
        if let Some(invite_code) = query
            .invite_code
            .as_deref()
            .map(str::trim)
            .filter(|invite_code| !invite_code.is_empty())
        {
            if let Some(user) = auth::find_pending_invite_user(&state.pool, invite_code).await? {
                return Ok(Json(json!({
                    "state": InitState::CreateInvitedUser,
                    "invite_code": invite_code,
                    "username": user.username,
                })));
            }
        }

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

    let library_count = libraries::Entity::find().count(&state.pool).await?;
    if library_count == 0 {
        return Ok(Json(json!({
            "state": InitState::CreateFirstLibrary
        })));
    }

    // server is setup and user can do things.
    return Ok(Json(json!({
        "state": InitState::Ready
    })));
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    lyra_ffprobe::paths::init_ffmpeg().unwrap();
    let config = get_config();
    let private_key = config
        .get_private_key()
        .expect("failed to load private key");
    let signer = signer::Signer::new(&private_key).expect("failed to load signer");

    let db_path = config.data_dir.join("data.db");
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
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

    tracing::info!("Running database migrations");
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    tracing::info!("Database migrations complete");

    let pool = DatabaseConnection::from(pool);
    reconcile_system_collections(&pool)
        .await
        .expect("Failed to reconcile system collections");
    let job_wake_signal = Arc::new(Notify::new());
    let job_semaphore = Arc::new(JobSemaphore::new());
    let startup_scans_complete = CancellationToken::new();

    CONTENT_UPDATE.start();

    let watch_session_registry = WatchSessionRegistry::new(pool.clone());
    watch_session_registry.start();

    clear_locked_jobs(&pool)
        .await
        .expect("Failed to clear stale job locks");

    let mut background_workers: JoinSet<anyhow::Result<()>> = JoinSet::new();

    let scanner_pool = pool.clone();
    let scanner_wake_signal = job_wake_signal.clone();
    let scanner_startup_scans_complete = startup_scans_complete.clone();
    background_workers.spawn(async move {
        scanner::start_scanner(
            scanner_pool,
            scanner_wake_signal,
            scanner_startup_scans_complete,
        )
        .await
    });

    let cleanup_pool = pool.clone();
    let cleanup_startup_scans_complete = startup_scans_complete.clone();
    background_workers.spawn(async move {
        start_cleanup_worker(cleanup_pool, cleanup_startup_scans_complete).await
    });

    let registered_jobs = load_registered_jobs(
        &pool,
        job_wake_signal.clone(),
        job_semaphore.clone(),
        startup_scans_complete.clone(),
    );
    for job in registered_jobs {
        let job_kind = job.job_kind;
        background_workers.spawn(async move {
            job.task
                .await
                .with_context(|| format!("job worker '{job_kind:?}' exited"))
        });
    }

    let schema: AppSchema = Schema::build(
        graphql::query::Query,
        graphql::mutation::Mutation,
        graphql::subscription::SubscriptionRoot,
    )
    .limit_depth(8)
    .limit_complexity(100)
    .limit_directives(5)
    .data(pool.clone())
    .data(DataLoader::new(
        graphql::dataloaders::node_metadata::NodeMetadataLoader::new(pool.clone()),
        tokio::spawn,
    ))
    .data(DataLoader::new(
        graphql::dataloaders::node_counts::NodeCountsLoader::new(pool.clone()),
        tokio::spawn,
    ))
    .data(signer.clone())
    .data(watch_session_registry)
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
        .nest("/api/assets", assets::get_assets_router())
        .route("/api/graphql", get(get_graphql).post(post_graphql))
        .route("/api/graphql/ws", get(get_graphql_ws))
        .route("/api/init", get(get_init_state))
        .route("/api/login", post(auth::post_login))
        .with_state(AppState {
            signer,
            packager_states: Arc::new(Mutex::new(HashMap::new())),
            pool: pool.clone(),
            schema: Arc::new(schema),
            job_wake_signal,
            job_semaphore,
            setup_code,
            last_setup_code_attempt: Arc::new(AtomicI64::new(0)),
        });

    #[cfg(all(feature = "static", not(debug_assertions)))]
    {
        use tower_http::services::{ServeDir, ServeFile};

        let static_path = std::env::var("LYRA_STATIC_PATH")
            .expect("LYRA_STATIC_PATH must be set for release builds with static feature");
        let index_path = static_path.clone() + "/index.html";
        let serve_dir = ServeDir::new(static_path)
            .not_found_service(ServeFile::new(index_path))
            .precompressed_gzip();

        app = app.fallback_service(serve_dir)
    }

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .unwrap();

    tracing::info!("Server starting on {}:{}", config.host, config.port);
    tracing::info!("Press Ctrl+C to shutdown gracefully");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(background_workers))
        .await
        .unwrap();

    tracing::info!("Server shutdown complete");
}

async fn shutdown_signal(mut background_workers: JoinSet<anyhow::Result<()>>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        worker = background_workers.join_next() => {
            match worker {
                Some(Ok(Ok(()))) => {
                    tracing::error!("a background worker exited unexpectedly");
                }
                Some(Ok(Err(error))) => {
                    tracing::error!(error = ?error, "a background worker failed");
                }
                Some(Err(error)) => {
                    tracing::error!(error = ?error, "a background worker panicked");
                }
                None => {
                    tracing::error!("all background workers exited");
                }
            }
        },
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C");
        },
        _ = terminate => {
            tracing::info!("Received termination signal");
        },
    }

    background_workers.abort_all();
}

async fn clear_locked_jobs(database: &DatabaseConnection) -> anyhow::Result<()> {
    jobs_entity::Entity::delete_many()
        .filter(jobs_entity::Column::LockedAt.is_not_null())
        .exec(database)
        .await?;

    Ok(())
}
