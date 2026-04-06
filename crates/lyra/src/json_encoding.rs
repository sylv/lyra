use anyhow::{Context, Result};
use serde::{Serialize, de::DeserializeOwned};
use std::io::Cursor;

const JSON_ZSTD_LEVEL: i32 = 12;

pub fn encode_json_zstd<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let json = serde_json::to_vec(value).context("failed to serialize cached JSON payload")?;
    zstd::encode_all(Cursor::new(json), JSON_ZSTD_LEVEL)
        .context("failed to zstd-compress cached JSON payload")
}

pub fn decode_json_zstd<T: DeserializeOwned>(payload: &[u8]) -> Result<T> {
    let decoded = match zstd::decode_all(Cursor::new(payload)) {
        Ok(bytes) => bytes,
        Err(_) => payload.to_vec(),
    };
    serde_json::from_slice(&decoded).context("failed to parse cached JSON payload")
}
