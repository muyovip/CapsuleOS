//! Deterministic physics simulation stub capsule

use serde::{Serialize, Deserialize};
use render_core::RenderResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysResult {
    pub transforms: Vec<[f32; 16]>, // simple 4x4 transforms
}

pub fn simulate_physics(_render: &RenderResult) -> PhysResult {
    // Deterministic transforms
    let mut transforms = vec![];
    transforms.push([1.0,0.0,0.0,0.0, 0.0,1.0,0.0,0.0, 0.0,0.0,1.0,0.0, 0.0,0.0,0.0,1.0]);
    transforms.push([0.5,0.0,0.0,0.0, 0.0,0.5,0.0,0.0, 0.0,0.0,0.5,0.0, 0.5,0.0,0.0,1.0]);
    PhysResult { transforms }
}

impl PhysResult {
    pub fn canonical_serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(self, &mut bytes).expect("serialize ok");
        bytes
    }
}
