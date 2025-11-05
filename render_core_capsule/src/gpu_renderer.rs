use anyhow::Result;
use crate::{scene::Expression, FrameBuffer};

pub fn render_gpu_stub(_scene_expr: &Expression) -> Result<FrameBuffer> {
    let width = 128;
    let height = 128;
    let pixels = vec![0xCCu8; (width * height * 4) as usize];

    println!("GPU rendering stub executed. Check GPU feature flags.");
    
    Ok(FrameBuffer { width, height, pixels })
}
