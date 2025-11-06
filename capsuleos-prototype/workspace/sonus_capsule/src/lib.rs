//! Deterministic audio synthesis capsule

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::f32::consts::PI;

pub const SAMPLE_RATE: u32 = 48_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression { 
    Sine { freq: f32, duration: f32, amp: f32 } 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformExpr {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}

impl WaveformExpr {
    pub fn canonical_serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(self, &mut bytes).expect("serialize ok");
        bytes
    }
    
    pub fn compute_content_hash(&self) -> String {
        let cbor_bytes = self.canonical_serialize();
        let mut h = Sha256::new();
        h.update(b"AudioV1");
        h.update(&cbor_bytes);
        hex::encode(h.finalize())
    }
}

pub fn synth(expr: &Expression) -> anyhow::Result<WaveformExpr> {
    match expr {
        Expression::Sine { freq, duration, amp } => {
            let total = (*duration * SAMPLE_RATE as f32).round() as usize;
            let mut samples = Vec::with_capacity(total);
            for n in 0..total {
                let t = n as f32 / SAMPLE_RATE as f32;
                samples.push((2.0 * PI * freq * t).sin() * amp);
            }
            Ok(WaveformExpr { sample_rate: SAMPLE_RATE, samples })
        }
    }
}

pub fn write_waveform_cbor(path: &std::path::Path, wf: &WaveformExpr) -> anyhow::Result<()> {
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(wf, &mut bytes)?;
    std::fs::write(path, bytes)?;
    Ok(())
}
