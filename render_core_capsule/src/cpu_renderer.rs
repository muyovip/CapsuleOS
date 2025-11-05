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

                let x_start = min_x.floor().max(0.0_f32).min(width as f32) as u32;
                let x_end = max_x.ceil().max(0.0_f32).min(width as f32) as u32;
                let y_start = min_y.floor().max(0.0_f32).min(height as f32) as u32;
                let y_end = max_y.ceil().max(0.0_f32).min(height as f32) as u32;

                for y in y_start..y_end {
                    for x in x_start..x_end {
                        let px = x as f32 + 0.5_f32;
                        let py = y as f32 + 0.5_f32;
                        
                        if point_in_triangle(px, py, &transformed_verts) {
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
        }
        _ => bail!("Scene expression must be wrapped in a Scene container."),
    }

    Ok(FrameBuffer { width, height, pixels })
}

fn edge_function(ax: f32, ay: f32, bx: f32, by: f32, cx: f32, cy: f32) -> f32 {
    (cx - ax) * (by - ay) - (cy - ay) * (bx - ax)
}

fn point_in_triangle(px: f32, py: f32, verts: &[Vector3<f32>; 3]) -> bool {
    let v0 = &verts[0];
    let v1 = &verts[1];
    let v2 = &verts[2];
    
    let w0 = edge_function(v1.x, v1.y, v2.x, v2.y, px, py);
    let w1 = edge_function(v2.x, v2.y, v0.x, v0.y, px, py);
    let w2 = edge_function(v0.x, v0.y, v1.x, v1.y, px, py);
    
    (w0 >= 0.0_f32 && w1 >= 0.0_f32 && w2 >= 0.0_f32) || 
    (w0 <= 0.0_f32 && w1 <= 0.0_f32 && w2 <= 0.0_f32)
}
