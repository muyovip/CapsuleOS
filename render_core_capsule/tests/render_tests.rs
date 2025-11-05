use render_core_capsule::scene::*;
use render_core_capsule::*;
use nalgebra::{Vector3, Matrix4};
use anyhow::Result;

const TEST_WIDTH: u32 = 64;
const TEST_HEIGHT: u32 = 64;

fn create_deterministic_scene() -> Expression {
    Expression::Scene {
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        elements: vec![
            Expression::Material {
                id: 100,
                color: Vector3::new(0.0, 1.0, 0.0),
            },
            Expression::Triangle {
                vertices: [
                    Vector3::new(10.0, 10.0, 0.0),
                    Vector3::new(50.0, 10.0, 0.0),
                    Vector3::new(10.0, 50.0, 0.0),
                ],
                material_id: 100,
                transform: Matrix4::identity(),
            },
        ],
    }
}

#[test]
#[cfg(feature = "cpu_fallback")]
fn test_cpu_fallback_deterministic_output() -> Result<()> {
    let scene = create_deterministic_scene();
    let manifest = CapsuleManifest {
        name: "test_cpu".to_string(),
        version: 1,
        lineage: "test".to_string(),
    };
    let handle = load_capsule(manifest)?;

    let fb_expr = render_scene(&handle, &scene)?;

    let expected_hash = "77370fa934c5688ef77a06a2e46b1be2c40213d2a700fa3247f48a04b50c6b12"; 
    
    assert_eq!(
        fb_expr.content_hash,
        expected_hash.to_string(),
        "CPU fallback output hash does not match canonical hash. Determinism failed."
    );

    println!("Canonical Hash Matched: {}", fb_expr.content_hash);

    Ok(())
}

#[test]
#[cfg(any(feature = "vulkan", feature = "wgpu"))]
fn test_gpu_stub_produces_hash() -> Result<()> {
    let scene = create_deterministic_scene();
    let manifest = CapsuleManifest {
        name: "test_gpu".to_string(),
        version: 1,
        lineage: "test".to_string(),
    };
    let handle = load_capsule(manifest)?;

    let fb_expr = render_scene(&handle, &scene)?;
    
    assert!(!fb_expr.content_hash.is_empty(), "GPU stub failed to produce a content hash.");

    let expected_stub_hash = "50702d098e9b626154c1534065476a88b50f75e7a93540b615cc7fa82798e4af";

    assert_eq!(
        fb_expr.content_hash,
        expected_stub_hash.to_string(),
        "GPU stub output hash does not match expected stub hash."
    );

    Ok(())
}
