use crate::metadata::matcher::{process_items, process_roots};
use lyra_metadata::MetadataProvider;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::time::{Duration, sleep};

const WORKER_INTERVAL: Duration = Duration::from_secs(60);

pub async fn start_metadata_worker(
    pool: DatabaseConnection,
    providers: Vec<Arc<dyn MetadataProvider>>,
) -> anyhow::Result<()> {
    tracing::info!(
        provider_count = providers.len(),
        interval_secs = WORKER_INTERVAL.as_secs(),
        "metadata worker started"
    );

    loop {
        let now = chrono::Utc::now().timestamp();
        for provider in &providers {
            if let Err(error) = run_provider_tick(&pool, provider.as_ref(), now).await {
                tracing::error!(
                    provider_id = provider.id(),
                    error = ?error,
                    "metadata provider tick failed"
                );
            }
        }

        sleep(WORKER_INTERVAL).await;
    }
}

async fn run_provider_tick(
    pool: &DatabaseConnection,
    provider: &dyn MetadataProvider,
    now: i64,
) -> anyhow::Result<()> {
    process_roots(pool, provider, now).await?;
    process_items(pool, provider, now).await?;
    Ok(())
}
