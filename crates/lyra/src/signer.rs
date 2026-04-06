use crate::config::get_signing_key;
use anyhow::Result;
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use chrono::Utc;
use ed25519_dalek::{SIGNATURE_LENGTH, Signature, Signer};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct SignedPayload<T> {
    data: T,
    expires_at: i64,
}

pub fn sign<T: Serialize>(data: T, expires_in: Duration) -> Result<String> {
    let signing_key = get_signing_key();
    let expires_at = Utc::now()
        .checked_add_signed(chrono::Duration::from_std(expires_in)?)
        .expect("failed to calculate expiration time")
        .timestamp();

    let signed_payload = SignedPayload { data, expires_at };
    let mut payload = postcard::to_allocvec(&signed_payload).expect("failed to serialize payload");

    let signature = signing_key.sign(&payload);
    payload.extend_from_slice(&signature.to_bytes());

    Ok(BASE64_URL_SAFE_NO_PAD.encode(&payload))
}

pub fn verify<T: DeserializeOwned>(token: &str) -> Result<(Duration, T)> {
    let signing_key = get_signing_key();
    let payload = BASE64_URL_SAFE_NO_PAD.decode(token)?;
    if payload.len() < SIGNATURE_LENGTH {
        anyhow::bail!("invalid token");
    }

    let (payload_bytes, signature_bytes) = payload.split_at(payload.len() - SIGNATURE_LENGTH);

    let signature = Signature::from_bytes(signature_bytes.try_into()?);
    signing_key.verify(payload_bytes, &signature)?;

    let payload: SignedPayload<T> = postcard::from_bytes(payload_bytes)?;
    if Utc::now().timestamp() >= payload.expires_at {
        anyhow::bail!("token has expired");
    }

    let expires_in = Duration::from_secs((payload.expires_at - Utc::now().timestamp()) as u64);
    Ok((expires_in, payload.data))
}
