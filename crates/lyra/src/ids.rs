use rand::RngCore;
use sha2::{Digest, Sha256};

const CROCKFORD_BASE32: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

pub fn new_session_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    encode_crockford(&bytes)
}

pub fn new_invite_code() -> String {
    let mut bytes = [0_u8; 12];
    rand::rng().fill_bytes(&mut bytes);
    encode_crockford(&bytes)
}

pub fn generate_ulid() -> String {
    format!("u_{}", ulid::Ulid::new())
}

pub fn generate_hashid<'a>(parts: impl IntoIterator<Item = &'a str>) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.to_lowercase().as_bytes());
        hasher.update([0]);
    }

    let digest = hasher.finalize();
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    format!("h_{}", encode_crockford_u128(u128::from_be_bytes(bytes)))
}

fn encode_crockford(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    let mut buffer = 0_u16;
    let mut bits = 0_u8;

    for byte in bytes {
        buffer = (buffer << 8) | u16::from(*byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            let index = ((buffer >> bits) & 0x1f) as usize;
            output.push(CROCKFORD_BASE32[index] as char);
        }
    }

    if bits > 0 {
        let index = ((buffer << (5 - bits)) & 0x1f) as usize;
        output.push(CROCKFORD_BASE32[index] as char);
    }

    output
}

fn encode_crockford_u128(mut value: u128) -> String {
    let mut output = ['0'; 26];
    for idx in (0..26).rev() {
        output[idx] = CROCKFORD_BASE32[(value & 0x1f) as usize] as char;
        value >>= 5;
    }
    output.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::generate_hashid;

    #[test]
    fn hash_ids_are_ulid_shaped() {
        let id = generate_hashid(["example"]);
        assert_eq!(id.len(), 28);
        assert!(id.starts_with("h_"));
    }
}
