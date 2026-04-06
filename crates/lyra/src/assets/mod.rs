pub mod proxy;
pub mod service;
pub mod storage;

use crate::signer::sign;
pub use proxy::get_assets_router;
use serde::{Deserialize, Serialize};
pub use service::{create_local_asset_from_bytes, download_asset_to_local};
use std::time::Duration;

const ASSET_SIGNATURE_TTL: Duration = Duration::from_hours(1);

pub fn sign_asset_url(asset_id: &str) -> String {
    let payload = AssetPayload {
        asset_id: asset_id.to_string(),
    };
    let token = sign(payload, ASSET_SIGNATURE_TTL).expect("failed to sign asset URL");
    format!("/api/assets/{asset_id}/{token}")
}

#[derive(Serialize, Deserialize)]
pub struct AssetPayload {
    pub asset_id: String,
}
