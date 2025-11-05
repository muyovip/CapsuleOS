use anyhow::{Result, bail};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

pub mod scene;
mod cpu_renderer;
#[cfg(any(feature = "vulkan", feature = "wgpu"))]
mod gpu_renderer;

use scene::{Expression, FrameBufferExpr, CapsuleManifest};

const CONTENT_HASH_PREFIX: &[u8] = b"RenderV1";

#[derive(Debug, Clone, Copy)]
pub struct CapsuleHandle {
    pub id: u64, 
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrameBuffer {
    pub width: u32,
    pub height: u32,
    #[serde(with = "serde_bytes")]
    pub pixels: Vec<u8>,
}

impl FrameBuffer {
    pub fn canonical_serialize(&self) -> Result<Vec<u8>> {
        ciborium::ser::into_vec(&self).map_err(|e| anyhow::anyhow!("CBOR serialization error: {}", e))
    }

    pub fn compute_content_hash(&self) -> Result<String> {
        let serialized_bytes = self.canonical_serialize()?;
        let mut hasher = Sha256::new();
        hasher.update(CONTENT_HASH_PREFIX);
        hasher.update(&serialized_bytes);
        Ok(format!("{:x}", hasher.finalize()))
    }
}

pub fn load_capsule(manifest: CapsuleManifest) -> Result<CapsuleHandle> {
    let handle = CapsuleHandle { id: 0xDEADBEEF };
    
    #[cfg(feature = "vulkan")]
    println!("Initializing Vulkan backend (stub)...");
    #[cfg(feature = "wgpu")]
    println!("Initializing WGPU backend (stub)...");
    #[cfg(all(not(feature = "vulkan"), not(feature = "wgpu")))]
    {
        #[cfg(feature = "cpu_fallback")]
        println!("Initializing CPU Fallback renderer...");
        #[cfg(not(feature = "cpu_fallback"))]
        bail!("No rendering backend enabled (Vulkan, WGPU, or cpu_fallback).");
    }

    Ok(handle)
}

pub fn render_scene(handle: &CapsuleHandle, scene_expr: &Expression) -> Result<FrameBufferExpr> {
    let output_fb = if cfg!(feature = "cpu_fallback") {
        cpu_renderer::render_cpu_fallback(scene_expr)?
    } else if cfg!(any(feature = "vulkan", feature = "wgpu")) {
        #[cfg(any(feature = "vulkan", feature = "wgpu"))]
        {
            gpu_renderer::render_gpu_stub(scene_expr)?
        }
        #[cfg(not(any(feature = "vulkan", feature = "wgpu")))]
        bail!("No suitable renderer backend found.");
    } else {
        bail!("No suitable renderer backend found.");
    };

    let content_hash = output_fb.compute_content_hash()?;

    let fb_expr = FrameBufferExpr {
        width: output_fb.width,
        height: output_fb.height,
        content_hash,
    };

    Ok(fb_expr)
}
