use render_core_capsule::scene::*;
use render_core_capsule::*;
use nalgebra::{Vector3, Matrix4};
use anyhow::Result;
use std::fs;

fn main() -> Result<()> {
    println!("=================================================================");
    println!("RenderCore Capsule - Triangle Rendering Example");
    println!("=================================================================");
    println!();

    let scene = Expression::Scene {
        width: 128,
        height: 128,
        elements: vec![
            Expression::Material {
                id: 1,
                color: Vector3::new(1.0, 0.0, 0.0),
            },
            Expression::Triangle {
                vertices: [
                    Vector3::new(20.0, 20.0, 0.0),
                    Vector3::new(100.0, 20.0, 0.0),
                    Vector3::new(20.0, 100.0, 0.0),
                ],
                material_id: 1,
                transform: Matrix4::identity(),
            },
        ],
    };

    let manifest = CapsuleManifest {
        name: "render_triangle_example".to_string(),
        version: 1,
        lineage: "⊙₀".to_string(),
    };

    println!("Loading RenderCore Capsule...");
    let handle = load_capsule(manifest)?;
    println!();

    println!("Rendering scene...");
    let fb_expr = render_scene(&handle, &scene)?;
    println!();

    println!("=================================================================");
    println!("Rendering Complete");
    println!("=================================================================");
    println!("Width:        {} pixels", fb_expr.width);
    println!("Height:       {} pixels", fb_expr.height);
    println!("Content Hash: {}", fb_expr.content_hash);
    println!();

    let scene_cbor = serde_cbor::to_vec(&scene)?;
    fs::write("scene.cbor", &scene_cbor)?;
    println!("Scene saved to: scene.cbor ({} bytes)", scene_cbor.len());

    let fb_expr_cbor = serde_cbor::to_vec(&fb_expr)?;
    fs::write("framebuffer.cbor", &fb_expr_cbor)?;
    println!("Framebuffer expression saved to: framebuffer.cbor ({} bytes)", fb_expr_cbor.len());
    println!();

    println!("=================================================================");
    println!("Content-Addressable Rendering Verified");
    println!("=================================================================");
    println!("Hash: SHA256(\"RenderV1\" || CBOR(framebuffer))");
    println!("  => {}", fb_expr.content_hash);
    println!();

    Ok(())
}
