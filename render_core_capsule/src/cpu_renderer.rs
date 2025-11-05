use anyhow::{Result, bail};
use nalgebra::Vector3;
use crate::{scene::Expression, FrameBuffer};

const DEFAULT_WIDTH: u32 = 128;
const DEFAULT_HEIGHT: u32 = 128;

pub fn render_cpu_fallback(scene_expr: &Expression) -> Result<FrameBuffer> {
    let (width, height) = match scene_expr {
        Expression::Scene { width, height, .. } => (*width, *height),
        _ => (DEFAULT_WIDTH, DEFAULT_HEIGHT),
    };

    let mut pixels = vec![0u8; (width * height * 4) as usize];

    match scene_expr {
        Expression::Scene { elements, .. } => {
            let mut triangles = Vec::new();
            let mut material_map = std::collections::HashMap::new();

            for element in elements {
                match element {
                    Expression::Camera { .. } => {
                    },
                    Expression::Material { id, color } => {
                        material_map.insert(*id, *color);
                    },
                    Expression::Triangle { vertices, material_id, transform } => {
                        triangles.push((*vertices, *material_id, *transform));
                    },
                    Expression::Mesh { indices, positions, material_id, transform } => {
                        for chunk in indices.chunks(3) {
                            if chunk.len() == 3 {
                                let v0 = positions[chunk[0] as usize];
                                let v1 = positions[chunk[1] as usize];
                                let v2 = positions[chunk[2] as usize];
                                
                                triangles.push(([v0, v1, v2], *material_id, *transform));
                            }
                        }
                    },
                    _ => {}
                }
            }

            for (verts, mat_id, transform) in triangles {
                let color = material_map.get(&mat_id).cloned().unwrap_or(Vector3::new(1.0, 0.0, 1.0));
                
                let transformed_verts: [Vector3<f32>; 3] = verts.map(|v| {
                    let v_h = transform * v.to_homogeneous();
                    v_h.xyz()
                });

                let (min_x, max_x) = transformed_verts.iter().fold((width as f32, 0.0_f32), |(min_x, max_x), v| (min_x.min(v.x), max_x.max(v.x)));
                let (min_y, max_y) = transformed_verts.iter().fold((height as f32, 0.0_f32), |(min_y, max_y), v| (min_y.min(v.y), max_y.max(v.y)));

                let x_start = (min_x.floor() as u32).min(width).max(0);
                let x_end = (max_x.ceil() as u32).min(width).max(0);
                let y_start = (min_y.floor() as u32).min(height).max(0);
                let y_end = (max_y.ceil() as u32).min(height).max(0);

                for y in y_start..y_end {
                    for x in x_start..x_end {
                        let idx = ((y * width + x) * 4) as usize;
                        if idx + 3 < pixels.len() {
                            pixels[idx] = (color.z * 255.0) as u8;
                            pixels[idx + 1] = (color.y * 255.0) as u8;
                            pixels[idx + 2] = (color.x * 255.0) as u8;
                            pixels[idx + 3] = 255u8;
                        }
                    }
                }
            }
        }
        _ => bail!("Scene expression must be wrapped in a Scene container."),
    }

    Ok(FrameBuffer { width, height, pixels })
}
