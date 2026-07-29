//! Hex + hashing helpers (T2).
//!
//! Pure functions with no dependency on `types`: hex encode/decode with
//! validation, and a `digest` helper over SHA-256 that produces the hex-encoded
//! `Digest` the rest of the chain uses.

use sha2::{Digest as _, Sha256};

/// Lowercase-hex encode bytes (no `0x` prefix).
pub fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Decode a hex string to bytes. Returns `None` if it isn't valid hex.
pub fn from_hex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

/// True if `s` is non-empty, even-length, all hex digits.
pub fn is_hex(s: &str) -> bool {
    !s.is_empty() && s.len() % 2 == 0 && s.bytes().all(|c| c.is_ascii_hexdigit())
}

/// SHA-256 of `bytes`, hex-encoded — the chain's `Digest`.
pub fn digest(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    to_hex(&h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrips() {
        let bytes = [0xde, 0xad, 0xbe, 0xef];
        assert_eq!(to_hex(&bytes), "deadbeef");
        assert_eq!(from_hex("deadbeef").unwrap(), bytes);
        assert!(is_hex("deadbeef"));
        assert!(!is_hex("nothex"));
    }

    #[test]
    fn digest_matches_sha256() {
        assert_eq!(
            digest(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }
}
