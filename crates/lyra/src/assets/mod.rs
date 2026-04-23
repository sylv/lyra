mod job_cleanup;
mod job_download;
mod job_thumbhash;
mod proxy;
mod service;
mod storage;

use crate::signer::sign;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

pub(crate) use job_download::AssetDownloadJob;
pub(crate) use proxy::get_assets_router;
pub(crate) use service::{
    create_local_asset_from_bytes, create_local_file_asset_from_bytes, download_asset_to_local,
};
use std::time::Duration;
pub(crate) use storage::get_asset_output_path_from_mime_and_encoding;

const ASSET_SIGNATURE_TTL: Duration = Duration::from_hours(24);

pub(crate) fn sign_asset_url(asset_id: &str) -> String {
    let payload = AssetPayload {
        asset_id: asset_id.to_string(),
    };
    let token = sign(payload, ASSET_SIGNATURE_TTL).expect("failed to sign asset URL");
    format!("/api/assets/{asset_id}/{token}")
}

#[derive(Serialize, Deserialize)]
pub(crate) struct AssetPayload {
    pub asset_id: String,
}

pub(crate) fn register_jobs(
    jobs: &mut Vec<crate::jobs::RegisteredJob>,
    heavy_jobs: &mut Vec<Arc<dyn crate::jobs::HeavyJobRunner>>,
    pool: &DatabaseConnection,
    wake_signal: Arc<Notify>,
    startup_scans_complete: CancellationToken,
) {
    crate::jobs::register_job(
        Arc::new(job_cleanup::AssetCleanupJob),
        jobs,
        heavy_jobs,
        pool,
        wake_signal.clone(),
        startup_scans_complete.clone(),
    );
    crate::jobs::register_job(
        Arc::new(job_download::AssetDownloadJob),
        jobs,
        heavy_jobs,
        pool,
        wake_signal.clone(),
        startup_scans_complete.clone(),
    );
    crate::jobs::register_job(
        Arc::new(job_thumbhash::AssetThumbhashJob),
        jobs,
        heavy_jobs,
        pool,
        wake_signal,
        startup_scans_complete,
    );
}
