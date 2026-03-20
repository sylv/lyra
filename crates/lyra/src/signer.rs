use anyhow::{Context, bail};
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct Signer {
    private_key: Arc<[u8; 32]>,
}

impl Signer {
    pub fn new(private_key: &str) -> anyhow::Result<Self> {
        Ok(Self {
            private_key: Arc::new(decode_private_key(private_key)?),
        })
    }

    pub fn sign(&self, scope: &str, expires_in_seconds: i64, parts: &[&str]) -> String {
        let expires_at = Utc::now().timestamp() + expires_in_seconds;
        self.sign_with_expiry(scope, parts, expires_at)
    }

    pub fn sign_with_expiry(&self, scope: &str, parts: &[&str], expires_at: i64) -> String {
        let signature = self.signature_bytes(scope, parts, expires_at);
        format!("{expires_at}.{}", hex::encode(signature))
    }

    pub fn verify(&self, scope: &str, parts: &[&str], token: &str) -> bool {
        let Some((expires_at_raw, signature_raw)) = token.split_once('.') else {
            return false;
        };
        let Ok(expires_at) = expires_at_raw.parse::<i64>() else {
            return false;
        };
        if expires_at < Utc::now().timestamp() {
            return false;
        }

        let Ok(signature) = hex::decode(signature_raw) else {
            return false;
        };
        let Ok(mut mac) = HmacSha256::new_from_slice(self.private_key.as_ref()) else {
            return false;
        };
        mac.update(signature_payload(scope, parts, expires_at).as_bytes());
        mac.verify_slice(&signature).is_ok()
    }

    fn signature_bytes(&self, scope: &str, parts: &[&str], expires_at: i64) -> [u8; 32] {
        let mut mac = HmacSha256::new_from_slice(self.private_key.as_ref())
            .expect("signer hmac accepts fixed-size keys");
        mac.update(signature_payload(scope, parts, expires_at).as_bytes());

        let mut bytes = [0_u8; 32];
        bytes.copy_from_slice(&mac.finalize().into_bytes());
        bytes
    }
}

fn signature_payload(scope: &str, parts: &[&str], expires_at: i64) -> String {
    let mut payload = String::from(scope);
    for part in parts {
        payload.push('\n');
        payload.push_str(part);
    }
    payload.push('\n');
    payload.push_str(&expires_at.to_string());
    payload
}

fn decode_private_key(private_key: &str) -> anyhow::Result<[u8; 32]> {
    let decoded = hex::decode(private_key.trim()).context("failed to decode config.private_key")?;
    if decoded.len() != 32 {
        bail!(
            "config.private_key must decode to 32 bytes, got {}",
            decoded.len()
        );
    }

    let mut key = [0_u8; 32];
    key.copy_from_slice(&decoded);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::Signer;

    fn signer() -> Signer {
        Signer::new(&hex::encode([7_u8; 32])).expect("test signer key should be valid")
    }

    #[test]
    fn signature_verifies_for_matching_parts() {
        let signer = signer();
        let token = signer.sign_with_expiry("asset_url", &["user_1", "asset_1"], i64::MAX / 2);

        assert!(signer.verify("asset_url", &["user_1", "asset_1"], &token));
    }

    #[test]
    fn signature_rejects_other_scopes() {
        let signer = signer();
        let token = signer.sign_with_expiry("asset_url", &["user_1", "asset_1"], i64::MAX / 2);

        assert!(!signer.verify("hls", &["user_1", "asset_1"], &token));
    }

    #[test]
    fn signature_rejects_other_parts() {
        let signer = signer();
        let token = signer.sign_with_expiry("asset_url", &["user_1", "asset_1"], i64::MAX / 2);

        assert!(!signer.verify("asset_url", &["user_1", "asset_2"], &token));
    }

    #[test]
    fn signature_rejects_expired_tokens() {
        let signer = signer();
        let token = signer.sign_with_expiry("asset_url", &["user_1", "asset_1"], 0);

        assert!(!signer.verify("asset_url", &["user_1", "asset_1"], &token));
    }
}
