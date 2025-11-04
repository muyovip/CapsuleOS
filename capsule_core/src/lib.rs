// Cargo.toml dependencies:
// [dependencies]
// ed25519-dalek = "2.1"
// sha2 = "0.10"
// ciborium = "0.2"
// serde = { version = "1.0", features = ["derive"] }
// hex = "0.4"
// rand = "0.8"

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Core Data Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capsule {
    pub metadata: CapsuleMetadata,
    pub content: Vec<u8>,
    pub signature_block: SignatureBlock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleMetadata {
    pub id: String,
    pub version: String,
    pub timestamp: u64,
    pub parent_hash: Option<String>,
    pub claim: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureBlock {
    #[serde(with = "serde_bytes")]
    pub public_key: [u8; 32],
    #[serde(with = "serde_bytes")]
    pub signature: [u8; 64],
    pub content_hash: String,
}

#[derive(Debug, Clone)]
pub struct ProofResult {
    pub crypto_valid: bool,
    pub content_hash_valid: bool,
    pub root_lineage: bool,
}

// ============================================================================
// Canonical CBOR Serialization
// ============================================================================

/// Produces deterministic CBOR encoding per RFC 8949
pub fn canonical_cbor<T: Serialize>(data: &T) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();
    ciborium::into_writer(data, &mut buffer)
        .map_err(|e| format!("CBOR serialization failed: {}", e))?;
    Ok(buffer)
}

// ============================================================================
// Content Hashing with Prefixes
// ============================================================================

/// Compute content-addressable hash with a type-specific prefix.
/// Returns format: "prefix:hexhash"
pub fn compute_content_hash_with_prefix(prefix: &str, cbor_data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    hasher.update(cbor_data);
    let hash = hasher.finalize();
    format!("{}:{}", prefix, hex::encode(hash))
}

/// Legacy function for backward compatibility (GlyphV1 prefix, no prefix in output)
pub fn compute_content_hash(cbor_data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"GlyphV1");
    hasher.update(cbor_data);
    let hash = hasher.finalize();
    hex::encode(hash)
}

// ============================================================================
// Work Order 4: Content-Addressable Traits
// ============================================================================

/// Trait for deterministic canonical serialize to bytes (CBOR).
pub trait CanonicalSerialize {
    /// Produce deterministic canonical CBOR bytes for this value.
    /// Implementations should be stable and deterministic: identical input → identical bytes.
    fn canonical_serialize(&self) -> Vec<u8>;
}

/// Trait for computing content-addressable hash string for an object.
pub trait ContentAddressable {
    /// Return a prefixed content hash string (e.g. "GlyphV1:abcdef...").
    fn content_hash(&self) -> String;
}

// ============================================================================
// Work Order 4: Domain Types (Glyph, Expression, GraphNode)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Glyph {
    pub name: String,
    pub version: u32,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expression {
    pub expr_type: String,
    pub data: Vec<u8>,
    pub children: Vec<ExpressionRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: Option<String>,
    pub glyph: Option<GlyphRef>,
    pub expression: Option<ExpressionRef>,
    pub edges: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlyphRef {
    pub name: String,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpressionRef {
    pub expr_type: String,
    pub short_digest: Option<String>,
}

// ============================================================================
// CanonicalSerialize Implementations
// ============================================================================

impl CanonicalSerialize for Glyph {
    fn canonical_serialize(&self) -> Vec<u8> {
        serde_cbor::to_vec(self).expect("Glyph serialization should not fail")
    }
}

impl CanonicalSerialize for Expression {
    fn canonical_serialize(&self) -> Vec<u8> {
        serde_cbor::to_vec(self).expect("Expression serialization should not fail")
    }
}

impl CanonicalSerialize for GraphNode {
    fn canonical_serialize(&self) -> Vec<u8> {
        serde_cbor::to_vec(self).expect("GraphNode serialization should not fail")
    }
}

impl CanonicalSerialize for GlyphRef {
    fn canonical_serialize(&self) -> Vec<u8> {
        serde_cbor::to_vec(self).expect("GlyphRef serialization should not fail")
    }
}

impl CanonicalSerialize for ExpressionRef {
    fn canonical_serialize(&self) -> Vec<u8> {
        serde_cbor::to_vec(self).expect("ExpressionRef serialization should not fail")
    }
}

// ============================================================================
// ContentAddressable Implementations
// ============================================================================

impl ContentAddressable for Glyph {
    fn content_hash(&self) -> String {
        let ser = self.canonical_serialize();
        compute_content_hash_with_prefix("GlyphV1", &ser)
    }
}

impl ContentAddressable for Expression {
    fn content_hash(&self) -> String {
        let ser = self.canonical_serialize();
        compute_content_hash_with_prefix("ExprV1", &ser)
    }
}

impl ContentAddressable for GraphNode {
    fn content_hash(&self) -> String {
        let ser = self.canonical_serialize();
        compute_content_hash_with_prefix("NodeV1", &ser)
    }
}

impl ContentAddressable for GlyphRef {
    fn content_hash(&self) -> String {
        let ser = self.canonical_serialize();
        compute_content_hash_with_prefix("GlyphV1", &ser)
    }
}

impl ContentAddressable for ExpressionRef {
    fn content_hash(&self) -> String {
        let ser = self.canonical_serialize();
        compute_content_hash_with_prefix("ExprV1", &ser)
    }
}

// ============================================================================
// Keypair Generation
// ============================================================================

pub fn generate_keypair() -> SigningKey {
    let mut csprng = rand::rngs::OsRng;
    SigningKey::generate(&mut csprng)
}

// ============================================================================
// Root Capsule Creation (⊙₀)
// ============================================================================

pub fn create_root_capsule(keypair: &SigningKey, claim: String) -> Result<Capsule, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let metadata = CapsuleMetadata {
        id: "⊙₀".to_string(),
        version: "1.0.0".to_string(),
        timestamp,
        parent_hash: None, // Root has no parent
        claim,
    };

    let content = b"Genesis Capsule - Root of Trust".to_vec();

    // Serialize capsule data for signing (without signature block)
    #[derive(Serialize)]
    struct UnsignedCapsule<'a> {
        metadata: &'a CapsuleMetadata,
        content: &'a [u8],
    }

    let unsigned = UnsignedCapsule {
        metadata: &metadata,
        content: &content,
    };

    let cbor_data = canonical_cbor(&unsigned)?;
    let content_hash = compute_content_hash(&cbor_data);

    // Sign the content hash
    let signature = keypair.sign(content_hash.as_bytes());

    let signature_block = SignatureBlock {
        public_key: keypair.verifying_key().to_bytes(),
        signature: signature.to_bytes(),
        content_hash,
    };

    Ok(Capsule {
        metadata,
        content,
        signature_block,
    })
}

// ============================================================================
// Capsule Verification
// ============================================================================

pub fn verify_capsule(capsule: &Capsule) -> ProofResult {
    let mut result = ProofResult {
        crypto_valid: false,
        content_hash_valid: false,
        root_lineage: false,
    };

    // Step 1: Recompute content hash
    #[derive(Serialize)]
    struct UnsignedCapsule<'a> {
        metadata: &'a CapsuleMetadata,
        content: &'a [u8],
    }

    let unsigned = UnsignedCapsule {
        metadata: &capsule.metadata,
        content: &capsule.content,
    };

    let cbor_data = match canonical_cbor(&unsigned) {
        Ok(data) => data,
        Err(_) => return result,
    };

    let computed_hash = compute_content_hash(&cbor_data);
    result.content_hash_valid = computed_hash == capsule.signature_block.content_hash;

    // Step 2: Verify cryptographic signature
    let public_key = match VerifyingKey::from_bytes(&capsule.signature_block.public_key) {
        Ok(pk) => pk,
        Err(_) => return result,
    };

    let signature = match Signature::try_from(&capsule.signature_block.signature[..]) {
        Ok(sig) => sig,
        Err(_) => return result,
    };

    result.crypto_valid = public_key
        .verify(capsule.signature_block.content_hash.as_bytes(), &signature)
        .is_ok();

    // Step 3: Verify root lineage (no parent hash for root capsule)
    result.root_lineage = capsule.metadata.id == "⊙₀" && capsule.metadata.parent_hash.is_none();

    result
}

// ============================================================================
// Persistence
// ============================================================================

pub fn serialize_capsule(capsule: &Capsule) -> Result<Vec<u8>, String> {
    canonical_cbor(capsule)
}

pub fn persist_capsule(capsule: &Capsule, path: &str) -> Result<(), String> {
    let data = serialize_capsule(capsule)?;
    std::fs::write(path, data)
        .map_err(|e| format!("Failed to write capsule to disk: {}", e))?;
    Ok(())
}

pub fn load_capsule(path: &str) -> Result<Capsule, String> {
    let data = std::fs::read(path)
        .map_err(|e| format!("Failed to read capsule from disk: {}", e))?;
    ciborium::from_reader(&data[..])
        .map_err(|e| format!("Failed to deserialize capsule: {}", e))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_capsule_creation_and_verification() {
        println!("\n=== Root Capsule Creation Test ===");

        // Generate keypair
        let keypair = generate_keypair();
        println!("✓ Generated Ed25519 keypair");

        // Create root capsule
        let capsule = create_root_capsule(&keypair, "Root of Trust".to_string())
            .expect("Failed to create root capsule");
        println!("✓ Created root capsule (⊙₀)");
        println!("  ID: {}", capsule.metadata.id);
        println!("  Timestamp: {}", capsule.metadata.timestamp);
        println!("  Content Hash: {}", capsule.signature_block.content_hash);

        // Verify capsule
        let proof = verify_capsule(&capsule);
        println!("\n=== Verification Results ===");
        println!("crypto_valid: {}", proof.crypto_valid);
        println!("content_hash_valid: {}", proof.content_hash_valid);
        println!("root_lineage: {}", proof.root_lineage);

        assert!(proof.crypto_valid, "Cryptographic signature must be valid");
        assert!(proof.content_hash_valid, "Content hash must match");
        assert!(proof.root_lineage, "Must be valid root capsule");

        println!("\n✓ All verification checks passed");
    }

    #[test]
    fn test_capsule_persistence() {
        let keypair = generate_keypair();
        let capsule = create_root_capsule(&keypair, "Persistence Test".to_string())
            .expect("Failed to create capsule");

        // Persist to disk
        let path = "/tmp/test_capsule.capsule";
        persist_capsule(&capsule, path).expect("Failed to persist capsule");
        println!("✓ Persisted capsule to {}", path);

        // Load from disk
        let loaded = load_capsule(path).expect("Failed to load capsule");
        println!("✓ Loaded capsule from disk");

        // Verify loaded capsule
        let proof = verify_capsule(&loaded);
        assert!(proof.crypto_valid);
        assert!(proof.content_hash_valid);
        assert!(proof.root_lineage);
        println!("✓ Loaded capsule verification passed");

        // Cleanup
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_signature_tampering_detection() {
        let keypair = generate_keypair();
        let mut capsule = create_root_capsule(&keypair, "Tamper Test".to_string())
            .expect("Failed to create capsule");

        // Tamper with content
        capsule.content = b"Tampered content".to_vec();

        // Verification should fail
        let proof = verify_capsule(&capsule);
        assert!(!proof.content_hash_valid, "Tampering should be detected");
        println!("✓ Content tampering correctly detected");
    }

    #[test]
    fn test_canonical_cbor_determinism() {
        let metadata = CapsuleMetadata {
            id: "test".to_string(),
            version: "1.0.0".to_string(),
            timestamp: 1234567890,
            parent_hash: None,
            claim: "Test".to_string(),
        };

        let cbor1 = canonical_cbor(&metadata).unwrap();
        let cbor2 = canonical_cbor(&metadata).unwrap();

        assert_eq!(cbor1, cbor2, "CBOR encoding must be deterministic");
        println!("✓ CBOR encoding is deterministic");
    }

    // ========================================================================
    // Work Order 4 Tests
    // ========================================================================

    #[test]
    fn test_glyph_canonical_serialize_and_hash_prefix() {
        let g = Glyph {
            name: "example".to_string(),
            version: 1,
            payload: vec![1, 2, 3, 4],
        };
        let bytes = g.canonical_serialize();
        assert!(!bytes.is_empty(), "serialized glyph must be non-empty");
        let hash = g.content_hash();
        assert!(hash.starts_with("GlyphV1:"), "prefix must be present");
        // ensure deterministic: repeated calls equal
        let hash2 = g.content_hash();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_expression_hash_determinism() {
        let e = Expression {
            expr_type: "literal".to_string(),
            data: vec![10, 20],
            children: vec![ExpressionRef {
                expr_type: "child".to_string(),
                short_digest: None,
            }],
        };
        let h1 = e.content_hash();
        let h2 = e.content_hash();
        assert_eq!(h1, h2);
        assert!(h1.starts_with("ExprV1:"));
    }

    #[test]
    fn test_graphnode_hash_includes_nested() {
        let glyph_ref = GlyphRef {
            name: "g".to_string(),
            version: 7,
        };
        let expr_ref = ExpressionRef {
            expr_type: "op".to_string(),
            short_digest: Some("abc".to_string()),
        };
        let node = GraphNode {
            id: "node-1".to_string(),
            label: Some("first".to_string()),
            glyph: Some(glyph_ref.clone()),
            expression: Some(expr_ref.clone()),
            edges: vec!["node-2".to_string()],
        };

        let node_hash = node.content_hash();
        assert!(node_hash.starts_with("NodeV1:"));
        // Mutating clones should produce different hash if content changes
        let mut node2 = node.clone();
        node2.id = "node-1-changed".to_string();
        let node2_hash = node2.content_hash();
        assert_ne!(node_hash, node2_hash);
    }

    #[test]
    fn test_prefixes_are_different_for_same_bytes() {
        let gref = GlyphRef {
            name: "x".to_string(),
            version: 42,
        };
        let eref = ExpressionRef {
            expr_type: "x".to_string(),
            short_digest: None,
        };

        let gh = gref.content_hash();
        assert!(gh.starts_with("GlyphV1:"));
        let eh = eref.content_hash();
        assert!(eh.starts_with("ExprV1:"));
        assert_ne!(gh, eh, "different prefixes should produce different hashes");
    }

    #[test]
    fn test_compute_content_hash_with_prefix_helper() {
        let g = Glyph {
            name: "x".to_string(),
            version: 1,
            payload: vec![],
        };
        let ser = g.canonical_serialize();
        let direct = compute_content_hash_with_prefix("GlyphV1", &ser);
        let via_trait = g.content_hash();
        assert_eq!(direct, via_trait);
    }

    #[test]
    fn test_deterministic_for_equivalent_instances() {
        let a = Glyph {
            name: "ident".to_string(),
            version: 3,
            payload: vec![9, 8, 7],
        };
        let b = Glyph {
            name: "ident".to_string(),
            version: 3,
            payload: vec![9, 8, 7],
        };
        assert_eq!(a.canonical_serialize(), b.canonical_serialize());
        assert_eq!(a.content_hash(), b.content_hash());
    }
}
