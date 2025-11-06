//! Deterministic rendering stub capsule

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderResult {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // grayscale for canonical simplicity
}

pub fn render_scene(_desc: &str) -> RenderResult {
    // Deterministic 64x64 grayscale image with pattern based on fixed seed
    let width = 64; 
    let height = 64;
    let mut pixels = Vec::with_capacity((width*height) as usize);
    for y in 0..height {
        for x in 0..width {
            let v = ((x * 37 + y * 91) % 256) as u8;
            pixels.push(v);
        }
    }
    RenderResult { width, height, pixels }
}

impl RenderResult {
    pub fn canonical_serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(self, &mut bytes).expect("serialize ok");
        bytes
    }
}
