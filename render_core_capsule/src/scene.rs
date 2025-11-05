use serde::{Serialize, Deserialize};
use nalgebra::{Vector3, Matrix4};

#[derive(Debug, Serialize, Deserialize)]
pub struct CapsuleManifest {
    pub name: String,
    pub version: u32,
    pub lineage: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Expression {
    Triangle { 
        vertices: [Vector3<f32>; 3],
        material_id: u64,
        transform: Matrix4<f32>,
    },
    Mesh {
        indices: Vec<u32>,
        positions: Vec<Vector3<f32>>,
        material_id: u64,
        transform: Matrix4<f32>,
    },
    Camera {
        position: Vector3<f32>,
        target: Vector3<f32>,
        up: Vector3<f32>,
        fov_y: f32,
        aspect_ratio: f32,
    },
    Material {
        id: u64,
        color: Vector3<f32>,
    },
    Scene {
        elements: Vec<Expression>,
        width: u32,
        height: u32,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrameBufferExpr {
    pub width: u32,
    pub height: u32,
    pub content_hash: String,
}
