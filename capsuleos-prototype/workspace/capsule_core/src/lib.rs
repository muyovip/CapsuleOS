//! Content-addressable hashing utilities for CapsuleOS

use sha2::{Sha256, Digest};

/// Compute content hash with prefix (e.g., "RenderV1", "AudioV1", "NodeV1")
pub fn compute_content_hash_token(prefix: &str, serialized: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(prefix.as_bytes());
    h.update(serialized);
    hex::encode(h.finalize())
}

/// Trait for canonical CBOR serialization
pub trait CanonicalSerialize {
    fn canonical_serialize(&self) -> Vec<u8>;
}
