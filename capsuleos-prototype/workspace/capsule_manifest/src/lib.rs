//! Capsule manifest parser and verifier with Ed25519 signatures

use serde::{Serialize, Deserialize};
use thiserror::Error;
use ed25519_dalek::{Verifier, VerifyingKey, Signature};

pub const ROOT_ID: &str = "⊙₀";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleManifest {
    pub id: String,
    pub parent: Option<String>,
    pub signer_pubkey: Vec<u8>,
    pub signature: Vec<u8>,
    pub lineage: Vec<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum ManifestError {
    #[error("CBOR error: {0}")]
    Cbor(String),
    #[error("Invalid: {0}")]
    Invalid(String),
}

#[derive(Error, Debug)]
pub enum VerifyError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid pubkey")]
    InvalidPubKey,
    #[error("Lineage error: {0}")]
    LineageError(String),
}

pub fn parse_manifest(bytes: &[u8]) -> Result<CapsuleManifest, ManifestError> {
    let m: CapsuleManifest = ciborium::de::from_reader(bytes)
        .map_err(|e| ManifestError::Cbor(format!("{:?}", e)))?;
    if m.id.is_empty() { 
        return Err(ManifestError::Invalid("empty id".into())); 
    }
    if m.signer_pubkey.is_empty() { 
        return Err(ManifestError::Invalid("missing pubkey".into())); 
    }
    Ok(m)
}

fn signing_bytes(m: &CapsuleManifest) -> Result<Vec<u8>, ManifestError> {
    let mut temp = m.clone();
    temp.signature = Vec::new();
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(&temp, &mut bytes)
        .map_err(|e| ManifestError::Cbor(format!("{:?}", e)))?;
    Ok(bytes)
}

pub fn verify_capsule(m: &CapsuleManifest) -> Result<(), VerifyError> {
    if m.signer_pubkey.len() != 32 { 
        return Err(VerifyError::InvalidPubKey); 
    }
    let pubkey_bytes: [u8; 32] = m.signer_pubkey[..32].try_into()
        .map_err(|_| VerifyError::InvalidPubKey)?;
    let pubkey = VerifyingKey::from_bytes(&pubkey_bytes)
        .map_err(|_| VerifyError::InvalidPubKey)?;
    
    if m.signature.len() != 64 { 
        return Err(VerifyError::InvalidSignature); 
    }
    let sig_bytes: [u8; 64] = m.signature[..64].try_into()
        .map_err(|_| VerifyError::InvalidSignature)?;
    let sig = Signature::from_bytes(&sig_bytes);
    
    let sb = signing_bytes(m).map_err(|_| VerifyError::InvalidSignature)?;
    pubkey.verify(&sb, &sig).map_err(|_| VerifyError::InvalidSignature)?;
    
    // lineage checks
    if m.lineage.is_empty() { 
        return Err(VerifyError::LineageError("empty lineage".into())); 
    }
    if m.lineage[0] != m.id { 
        return Err(VerifyError::LineageError("lineage[0] != id".into())); 
    }
    if m.lineage.last().map(|s| s.as_str()) != Some(ROOT_ID) { 
        return Err(VerifyError::LineageError("lineage must end at ROOT_ID".into())); 
    }
    Ok(())
}
