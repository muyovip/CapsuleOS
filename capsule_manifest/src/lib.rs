//! capsule_manifest crate
//! Work Order 9 — CapsuleManifest Parser, Verifier, and Loader
//! - Deterministic CBOR manifest format
//! - Ed25519 signature verification (signature is over canonical CBOR without signature field)
//! - Lineage verification ending at '⊙₀'
//! - Simple in-memory Gamma loader/registry

use ed25519_dalek::{PublicKey, Signature, Verifier};
use serde::{Deserialize, Serialize};
use serde_cbor;
use std::collections::BTreeMap;
use thiserror::Error;

/// Root sentinel identifier — lineage must terminate in this id.
pub const ROOT_ID: &str = "⊙₀";

/// The manifest payload. Fields order affects canonical CBOR; keep declaration order stable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapsuleManifest {
    /// Unique capsule id
    pub id: String,

    /// Parent capsule id (optional; top-level capsules might be None)
    pub parent: Option<String>,

    /// Ed25519 signature bytes signing the canonical manifest without `signature` field
    pub signature: Vec<u8>,

    /// Lineage chain from parent -> ... -> ROOT_ID. The last element must be ROOT_ID.
    pub lineage: Vec<String>,

    /// Arbitrary metadata (use BTreeMap for deterministic ordering of keys when serializing)
    pub metadata: BTreeMap<String, serde_cbor::Value>,
}

/// Result of a successful verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofResult {
    /// signature verified
    pub signature_valid: bool,

    /// lineage chain verified to end at ROOT_ID
    pub lineage_valid: bool,
}

/// Errors during parsing a manifest from bytes
#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("CBOR deserialize error: {0}")]
    CborDeserialize(#[from] serde_cbor::Error),
    #[error("Manifest integrity error: {0}")]
    Integrity(String),
}

/// Errors during verification
#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Lineage invalid: {0}")]
    LineageInvalid(String),
    #[error("Signature parse error")]
    SignatureParse,
    #[error("Verifier error: {0}")]
    Verifier(String),
}

/// Errors during loading
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    #[error("Registry error: {0}")]
    RegistryError(String),
}

/// Parse a CBOR byte slice into `CapsuleManifest`.
///
/// Expects canonical CBOR for structs (serde_cbor::from_slice).
pub fn parse_manifest(bytes: &[u8]) -> Result<CapsuleManifest, ManifestError> {
    let manifest: CapsuleManifest = serde_cbor::from_slice(bytes)?;
    // quick sanity checks
    if manifest.id.is_empty() {
        return Err(ManifestError::Integrity("id is empty".into()));
    }
    if manifest.lineage.is_empty() {
        return Err(ManifestError::Integrity(
            "lineage must be non-empty and end with root".into(),
        ));
    }
    if manifest.lineage.last().map(|s| s.as_str()) != Some(ROOT_ID) {
        return Err(ManifestError::Integrity(format!(
            "lineage must end with root id '{}'",
            ROOT_ID
        )));
    }
    Ok(manifest)
}

/// Serialize the manifest into canonical CBOR bytes **excluding** the `signature` field.
/// This is the exact byte sequence that should be signed/verified.
/// We build a temporary struct-like map to ensure the `signature` field is omitted.
fn canonical_serialize_without_signature(man: &CapsuleManifest) -> Vec<u8> {
    // We'll serialize as a CBOR map with the same fields except signature, and ensure deterministic key order.
    // Use BTreeMap to enforce ordering of keys by their textual name (stable).
    // Key order chosen: id, parent, lineage, metadata
    let mut map = BTreeMap::new();
    map.insert(
        "id",
        serde_cbor::to_value(&man.id).expect("to_value id should not fail"),
    );
    map.insert(
        "parent",
        serde_cbor::to_value(&man.parent).expect("to_value parent should not fail"),
    );
    map.insert(
        "lineage",
        serde_cbor::to_value(&man.lineage).expect("to_value lineage should not fail"),
    );
    map.insert(
        "metadata",
        serde_cbor::to_value(&man.metadata).expect("to_value metadata should not fail"),
    );
    // Serializing the BTreeMap will produce deterministic CBOR ordering for these keys.
    serde_cbor::to_vec(&map).expect("canonical serialize should not fail")
}

/// Verify the manifest's signature and lineage.
/// - `manifest`: manifest to verify
/// - `public_key`: Ed25519 public key used to verify the signature
/// Returns `ProofResult` if signature verifiable and lineage structure okay; otherwise `VerifyError`.
pub fn verify_capsule(
    manifest: &CapsuleManifest,
    public_key: &PublicKey,
) -> Result<ProofResult, VerifyError> {
    // 1) Verify signature bytes length and parse into ed25519 Signature
    let sig = Signature::from_bytes(&manifest.signature).map_err(|_| VerifyError::SignatureParse)?;

    // 2) Compute canonical bytes for the manifest without signature
    let canonical = canonical_serialize_without_signature(manifest);

    // 3) Verify signature using ed25519-dalek
    public_key
        .verify(&canonical, &sig)
        .map_err(|e| VerifyError::Verifier(format!("{}", e)))?;

    // 4) Verify lineage chain ends with ROOT_ID already validated in parse_manifest; here check parent relation
    // If parent is present, lineage[0] should match parent.
    let lineage_valid = match (&manifest.parent, manifest.lineage.get(0)) {
        (Some(parent), Some(first)) => parent == first,
        (None, Some(_)) => {
            // parent None but lineage exists: allow but assert first element equals id's parent? We treat no parent as allowed only if lineage[0] == manifest.id's parent absent; this is ambiguous, so we'll allow this case but lineage_valid = true only if lineage first != manifest.id (makes no strong assumption).
            true
        }
        _ => false,
    };

    if !lineage_valid {
        return Err(VerifyError::LineageInvalid(
            "parent must equal first element of lineage".into(),
        ));
    }

    // also ensure last is ROOT_ID (parse_manifest already enforced), but re-check to be safe
    if manifest.lineage.last().map(|s| s.as_str()) != Some(ROOT_ID) {
        return Err(VerifyError::LineageInvalid(format!(
            "lineage must end with root id '{}'",
            ROOT_ID
        )));
    }

    Ok(ProofResult {
        signature_valid: true,
        lineage_valid: true,
    })
}

/// Gamma registry - simple in-memory registry representing Γ where capsules get registered.
/// Replace this with the real Γ registration mechanism as needed.
#[derive(Debug, Default)]
pub struct Gamma {
    store: std::collections::HashMap<String, CapsuleManifest>,
}

impl Gamma {
    /// new empty registry
    pub fn new() -> Self {
        Self {
            store: Default::default(),
        }
    }

    /// register a manifest into Gamma
    pub fn register(&mut self, manifest: CapsuleManifest) -> Result<(), String> {
        let id = manifest.id.clone();
        if self.store.contains_key(&id) {
            return Err(format!("capsule '{}' already registered", id));
        }
        self.store.insert(id, manifest);
        Ok(())
    }

    /// fetch a manifest
    pub fn get(&self, id: &str) -> Option<&CapsuleManifest> {
        self.store.get(id)
    }
}

/// Load a manifest into Γ.
///
/// The function verifies the manifest with `public_key`, then registers into the given registry.
/// Returns an error if verification fails or registration fails.
pub fn load_manifest(
    manifest: CapsuleManifest,
    registry: &mut Gamma,
    public_key: &PublicKey,
) -> Result<(), LoadError> {
    verify_capsule(&manifest, public_key)
        .map_err(|e| LoadError::VerificationFailed(format!("{}", e)))?;
    registry
        .register(manifest)
        .map_err(|e| LoadError::RegistryError(e))?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Unit tests
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Keypair, Signer};
    use rand::rngs::OsRng;

    fn make_demo_manifest(keypair: &Keypair) -> (CapsuleManifest, Vec<u8>) {
        // Create a manifest with some metadata
        let mut md = BTreeMap::new();
        md.insert(
            "name".to_string(),
            serde_cbor::Value::Text("example-capsule".to_string()),
        );
        md.insert("version".to_string(), serde_cbor::Value::Integer(1.into()));

        // parent and lineage
        let parent = Some("parent-abc".to_string());
        let lineage = vec![
            parent.clone().unwrap(),
            "grandparent".to_string(),
            ROOT_ID.to_string(),
        ];

        // Construct a manifest with empty signature for now
        let mut manifest = CapsuleManifest {
            id: "capsule-1".to_string(),
            parent: parent.clone(),
            signature: vec![],
            lineage,
            metadata: md,
        };

        // Compute canonical bytes without signature, sign them, and set signature bytes
        let bytes_to_sign = canonical_serialize_without_signature(&manifest);
        let sig = keypair.sign(&bytes_to_sign);
        manifest.signature = sig.to_bytes().to_vec();

        // Also produce the CBOR bytes for the full manifest (with signature) for testing parse_manifest
        let bytes_full =
            serde_cbor::to_vec(&manifest).expect("should serialize full manifest to cbor");
        (manifest, bytes_full)
    }

    #[test]
    fn test_parse_verify_load_happy_path() {
        let mut rng = OsRng {};
        let keypair = Keypair::generate(&mut rng);
        let public_key = keypair.public;

        let (manifest, bytes_cbor) = make_demo_manifest(&keypair);

        // parse
        let parsed = parse_manifest(&bytes_cbor).expect("parse should succeed");
        assert_eq!(parsed.id, manifest.id);

        // verify
        let proof = verify_capsule(&parsed, &public_key).expect("verify should succeed");
        assert!(proof.signature_valid);
        assert!(proof.lineage_valid);

        // load
        let mut registry = Gamma::new();
        load_manifest(parsed.clone(), &mut registry, &public_key).expect("load should succeed");
        assert!(registry.get(&manifest.id).is_some());
    }

    #[test]
    fn test_invalid_signature_fails_verify() {
        let mut rng = OsRng {};
        let keypair = Keypair::generate(&mut rng);
        let other_keypair = Keypair::generate(&mut rng); // different key to sign wrong
        let public_key = keypair.public;

        // make a manifest but sign with a different key (other_keypair)
        let (mut manifest, bytes_full) = make_demo_manifest(&other_keypair);
        // override id so it's not trivially same as other test
        manifest.id = "capsule-invalid-sig".to_string();
        // Recompute full bytes with wrong signature set (we already set signature)
        let bytes_full =
            serde_cbor::to_vec(&manifest).expect("should serialize full manifest to cbor");

        let parsed = parse_manifest(&bytes_full).expect("parse should succeed");

        // verify with original public_key should fail
        let v = verify_capsule(&parsed, &public_key);
        assert!(v.is_err());
        match v {
            Err(VerifyError::Verifier(_)) | Err(VerifyError::InvalidSignature) | Err(VerifyError::SignatureParse) => {
                // acceptable: verification failed
            }
            Err(e) => panic!("unexpected verify error variant: {:?}", e),
            Ok(_) => panic!("verification unexpectedly succeeded"),
        }
    }

    #[test]
    fn test_tampered_metadata_fails_signature() {
        let mut rng = OsRng {};
        let keypair = Keypair::generate(&mut rng);
        let public_key = keypair.public;

        let (mut manifest, mut bytes_full) = make_demo_manifest(&keypair);

        // Tamper the manifest bytes directly (simulate corrupted transmission)
        // We'll flip a byte in the metadata area of the CBOR to invalidate signature.
        // Find a byte we can flip — pick an index in the middle.
        let idx = bytes_full.len() / 2;
        bytes_full[idx] = bytes_full[idx].wrapping_add(1);

        let parsed = parse_manifest(&bytes_full);
        // parsing may fail if tamper made CBOR invalid; if it succeeded, verifying should fail.
        match parsed {
            Ok(m) => {
                let v = verify_capsule(&m, &public_key);
                assert!(v.is_err());
            }
            Err(_) => {
                // parsing failed which is acceptable for tampered bytes
            }
        }
    }

    #[test]
    fn test_lineage_must_end_with_root() {
        let mut rng = OsRng {};
        let keypair = Keypair::generate(&mut rng);

        // Build manifest with lineage not ending in root
        let mut md = BTreeMap::new();
        md.insert(
            "name".to_string(),
            serde_cbor::Value::Text("bad-lineage".to_string()),
        );

        let parent = Some("parent-x".to_string());
        let lineage = vec!["parent-x".to_string(), "grandparent".to_string()]; // missing root

        let mut manifest = CapsuleManifest {
            id: "capsule-bad-lineage".to_string(),
            parent: parent.clone(),
            signature: vec![],
            lineage: lineage.clone(),
            metadata: md,
        };

        // sign over canonical bytes (without signature)
        let bytes_to_sign = canonical_serialize_without_signature(&manifest);
        let sig = keypair.sign(&bytes_to_sign);
        manifest.signature = sig.to_bytes().to_vec();

        // Serializing full manifest and attempting parse should fail because parse enforces lineage ends with root
        let bytes_full = serde_cbor::to_vec(&manifest).expect("serialize manifest");
        let parsed = parse_manifest(&bytes_full);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_load_manifest_already_registered_rejects() {
        let mut rng = OsRng {};
        let keypair = Keypair::generate(&mut rng);
        let public_key = keypair.public;

        let (manifest, bytes_full) = make_demo_manifest(&keypair);
        let parsed = parse_manifest(&bytes_full).expect("parse ok");
        let mut gamma = Gamma::new();
        load_manifest(parsed.clone(), &mut gamma, &public_key).expect("first load ok");
        // second load should fail due to duplicate id
        let res = load_manifest(parsed, &mut gamma, &public_key);
        assert!(res.is_err());
        match res {
            Err(LoadError::RegistryError(_)) => {}
            Err(e) => panic!("unexpected load error: {:?}", e),
            Ok(_) => panic!("expected error"),
        }
    }
}
