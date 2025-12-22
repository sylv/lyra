use crate::{
    get_keyframes::get_keyframes,
    hls::{SEGMENT_ROOT, TEST_FILE},
};
use async_graphql::{Schema, SimpleObject, http::GraphiQLSource};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Router,
    extract::{FromRef, State},
    response::{Html, IntoResponse},
    routing::get,
};
use std::{sync::Arc, time::Instant};
use tracing::info;

mod error;
mod ffprobe;
mod get_keyframes;
mod graphql;
mod hls;

type AppSchema =
    Schema<graphql::query::Query, graphql::mutation::Mutation, async_graphql::EmptySubscription>;

#[derive(Clone, FromRef)]
pub struct AppState {
    files: Arc<Vec<TestFile>>,
    schema: Arc<AppSchema>,
    keyframes: Vec<f64>,
}

#[derive(Clone, SimpleObject)]
pub struct TestFile {
    pub id: String,
    pub path: String,
}

async fn get_graphql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/api/graphql").finish())
}

async fn post_graphql(State(state): State<AppState>, req: GraphQLRequest) -> GraphQLResponse {
    state.schema.execute(req.into_inner()).await.into()
}

// const FILE_DIR: &str = "placeholder.mkv";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    if std::path::Path::new(SEGMENT_ROOT).exists() {
        tracing::info!("cleaned up existing segment root");
        std::fs::remove_dir_all(SEGMENT_ROOT).expect("failed to clean segment root");
    } else {
        tracing::info!("segment root does not exist");
    }

    let keyframes = get_keyframes(TEST_FILE).await.unwrap();
    info!("extracted keyframes: {:?}", &keyframes[..20]);

    let start = Instant::now();
    let files: Arc<Vec<TestFile>> = Arc::new(vec![]);
    info!("loaded {} files in {:?}", files.len(), start.elapsed());

    let schema: AppSchema = Schema::build(
        graphql::query::Query,
        graphql::mutation::Mutation,
        async_graphql::EmptySubscription,
    )
    .limit_depth(5)
    .limit_complexity(100)
    .limit_directives(5)
    .data(files.clone())
    .finish();

    let app = Router::new()
        .route("/api/graphql", get(get_graphql).post(post_graphql))
        .merge(hls::router())
        .with_state(AppState {
            files,
            schema: Arc::new(schema),
            keyframes,
        });

    let bind_addr =
        std::env::var("LYRA_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("bind http listener");

    info!("Server starting on {bind_addr}");
    info!("Press Ctrl+C to shutdown gracefully");

    axum::serve(listener, app).await.expect("server failure");
    info!("Server shutdown complete");
}
