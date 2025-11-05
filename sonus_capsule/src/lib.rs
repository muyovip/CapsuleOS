//! Sonus Capsule — Deterministic Wavefield Synthesis Core
//!
//! Implements canonical deterministic audio synthesis at 48kHz sampling rate.
//! Serializes waveforms as canonical CBOR and content-hashes with prefix "AudioV1".
//!
//! Provides Expression model for sine(freq, dur, amp) primitives.

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::f32::consts::PI;
use thiserror::Error;
use anyhow::Result;

/// Canonical sampling rate in Hz
pub const SAMPLE_RATE: u32 = 48_000;

/// Expression describing canonical audio generation forms.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Expression {
    /// Sine wave generator: frequency (Hz), duration (seconds), amplitude (0–1)
    Sine { freq: f32, duration: f32, amp: f32 },
}

/// Canonical PCM representation (float32)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WaveformExpr {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
    /// SHA256("AudioV1" || CBOR(samples))
    pub content_hash: String,
    pub metadata: AudioMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioMetadata {
    pub expr: Expression,
    pub length_samples: usize,
    pub duration_sec: f32,
}

#[derive(Debug, Error)]
pub enum SonusError {
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Deterministically synthesize a waveform expression.
pub fn synth(expr: &Expression) -> Result<WaveformExpr, SonusError> {
    match expr {
        Expression::Sine { freq, duration, amp } => synth_sine(*freq, *duration, *amp, SAMPLE_RATE),
    }
}

/// Canonical deterministic sine synthesis
fn synth_sine(freq: f32, duration: f32, amp: f32, sr: u32) -> Result<WaveformExpr, SonusError> {
    if freq <= 0.0 || duration <= 0.0 || amp < 0.0 {
        return Err(SonusError::InvalidParameter(format!(
            "freq={}, dur={}, amp={}",
            freq, duration, amp
        )));
    }

    let total_samples = (duration * sr as f32).round() as usize;
    let mut samples = Vec::with_capacity(total_samples);
    // deterministic synthesis (sample-accurate)
    for n in 0..total_samples {
        let t = n as f32 / sr as f32;
        let sample = (2.0 * PI * freq * t).sin() * amp;
        samples.push(sample);
    }

    // Canonical CBOR serialization and content hash
    let cbor_bytes = canonical_serialize(&samples)?;
    let hash = compute_content_hash_from_bytes(&cbor_bytes);

    let meta = AudioMetadata {
        expr: Expression::Sine {
            freq,
            duration,
            amp,
        },
        length_samples: total_samples,
        duration_sec: duration,
    };

    Ok(WaveformExpr {
        sample_rate: sr,
        samples,
        content_hash: hash,
        metadata: meta,
    })
}

/// Helper function to perform canonical CBOR serialization.
fn canonical_serialize<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, SonusError> {
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(value, &mut bytes)
        .map_err(|e| SonusError::Serialization(format!("CBOR serialization failed: {:?}", e)))?;
    Ok(bytes)
}

/// Compute content hash from CBOR bytes with "AudioV1" prefix
fn compute_content_hash_from_bytes(cbor_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"AudioV1");
    hasher.update(cbor_bytes);
    format!("{:x}", hasher.finalize())
}

/// Serialize waveform to canonical CBOR file
pub fn write_waveform_cbor(path: &std::path::Path, wf: &WaveformExpr) -> Result<(), SonusError> {
    let bytes = canonical_serialize(wf)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

/// Load waveform from canonical CBOR
pub fn read_waveform_cbor(path: &std::path::Path) -> Result<WaveformExpr, SonusError> {
    let bytes = std::fs::read(path)?;
    let wf: WaveformExpr = ciborium::de::from_reader(&bytes[..])
        .map_err(|e| SonusError::Serialization(format!("CBOR deserialization failed: {:?}", e)))?;
    Ok(wf)
}

/// Deterministic hash re-calculation for validation
pub fn compute_content_hash(samples: &[f32]) -> Result<String, SonusError> {
    let bytes = canonical_serialize(samples)?;
    Ok(compute_content_hash_from_bytes(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_synth_sine_440hz_deterministic() {
        let expr = Expression::Sine {
            freq: 440.0,
            duration: 1.0,
            amp: 0.5,
        };
        let wf1 = synth(&expr).unwrap();
        let wf2 = synth(&expr).unwrap();

        // identical hash and samples
        assert_eq!(wf1.content_hash, wf2.content_hash);
        assert_eq!(wf1.samples.len(), SAMPLE_RATE as usize);
        assert_relative_eq!(wf1.samples[0], wf2.samples[0]);
        assert_eq!(
            wf1.content_hash,
            compute_content_hash(&wf1.samples).unwrap()
        );
        
        println!("✓ Deterministic synthesis test passed");
        println!("  Hash: {}", wf1.content_hash);
        println!("  Samples: {}", wf1.samples.len());
    }
}
