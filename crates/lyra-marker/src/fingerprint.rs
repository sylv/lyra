use anyhow::Context;

use crate::{
    AUDIO_FINGERPRINT_CACHE_MAGIC, AUDIO_FINGERPRINT_CACHE_SCHEMA_VERSION,
    AUDIO_FINGERPRINT_VERSION,
};

#[derive(Clone, Debug)]
pub struct Fingerprint {
    cache: Vec<u8>,
}

impl Fingerprint {
    pub fn from_bytes(cache: Vec<u8>) -> anyhow::Result<Self> {
        Self::decode_cache(&cache).context("invalid fingerprint cache")?;
        Ok(Self { cache })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.cache
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.cache
    }

    pub(crate) fn from_values(values: &[u32]) -> Self {
        let mut cache = Vec::with_capacity(16 + values.len().saturating_mul(4));
        cache.extend_from_slice(&AUDIO_FINGERPRINT_CACHE_MAGIC);
        cache.extend_from_slice(&AUDIO_FINGERPRINT_CACHE_SCHEMA_VERSION.to_le_bytes());
        cache.extend_from_slice(&AUDIO_FINGERPRINT_VERSION.to_le_bytes());
        cache.extend_from_slice(&(values.len() as u32).to_le_bytes());
        for value in values {
            cache.extend_from_slice(&value.to_le_bytes());
        }

        Self { cache }
    }

    pub(crate) fn decode(&self) -> anyhow::Result<Vec<u32>> {
        Self::decode_cache(&self.cache).context("invalid fingerprint cache")
    }

    fn decode_cache(cache: &[u8]) -> Option<Vec<u32>> {
        if cache.is_empty() || cache.len() < 16 {
            return None;
        }
        if cache[0..4] != AUDIO_FINGERPRINT_CACHE_MAGIC {
            return None;
        }

        let schema_version = read_u32_le(cache, 4)?;
        if schema_version != AUDIO_FINGERPRINT_CACHE_SCHEMA_VERSION {
            return None;
        }

        let fingerprint_version = read_u32_le(cache, 8)?;
        if fingerprint_version != AUDIO_FINGERPRINT_VERSION {
            return None;
        }

        let value_count = read_u32_le(cache, 12)? as usize;
        let payload = cache.get(16..)?;
        if payload.len() != value_count.saturating_mul(4) {
            return None;
        }

        let mut output = Vec::with_capacity(value_count);
        for chunk in payload.chunks_exact(4) {
            output.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
        Some(output)
    }
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let chunk = bytes.get(offset..end)?;
    Some(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
}
