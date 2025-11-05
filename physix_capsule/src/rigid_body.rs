use serde::{Serialize, Deserialize};
use nalgebra::{Vector3, Matrix3, UnitQuaternion};

use crate::World;

/// Canonical GΛLYPH scene definition for a rigid body.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    /// Position (center of mass).
    pub position: Vector3<f32>,
    /// Orientation.
    pub rotation: UnitQuaternion<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "shape_type")]
pub enum Shape {
    Box {
        half_extents: Vector3<f32>,
    },
    Sphere {
        radius: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigidBody {
    /// Canonical, unique body ID (important for deterministic ordering).
    pub id: String, 
    pub mass: f32,
    /// Inverse mass (0.0 for static bodies).
    pub inv_mass: f32,
    /// World-space inverse inertia tensor.
    pub inv_inertia: Matrix3<f32>, 
    
    pub transform: Transform,
    
    /// Linear velocity.
    pub linear_velocity: Vector3<f32>,
    /// Angular velocity.
    pub angular_velocity: Vector3<f32>,
    
    pub shape: Shape,
    pub external_force: Vector3<f32>, 
    pub external_torque: Vector3<f32>,
}

impl RigidBody {
    /// Creates a simple dynamic box.
    pub fn new_box(id: &str, pos: Vector3<f32>, mass: f32, half_extents: Vector3<f32>) -> Self {
        let inv_mass = 1.0 / mass;
        let i_diag = 1.0 / 12.0 * mass * Vector3::new(
            half_extents.y.powi(2) + half_extents.z.powi(2),
            half_extents.x.powi(2) + half_extents.z.powi(2),
            half_extents.x.powi(2) + half_extents.y.powi(2),
        );
        let inv_inertia = Matrix3::from_diagonal(&i_diag.map(|x| 1.0 / x));

        RigidBody {
            id: id.to_string(),
            mass,
            inv_mass,
            inv_inertia,
            transform: Transform {
                position: pos,
                rotation: UnitQuaternion::identity(),
            },
            linear_velocity: Vector3::zeros(),
            angular_velocity: Vector3::zeros(),
            shape: Shape::Box { half_extents },
            external_force: Vector3::new(0.0, -9.81 * mass, 0.0),
            external_torque: Vector3::zeros(),
        }
    }
}

/// Fixed time-step integrator (Semi-Implicit Euler).
pub fn integrate(world: &mut World) {
    let dt = crate::FIXED_DT;

    for body in &mut world.bodies {
        if body.inv_mass == 0.0 {
            continue;
        }

        let linear_accel = body.external_force * body.inv_mass;
        let angular_accel = body.inv_inertia * body.external_torque; 

        body.linear_velocity += linear_accel * dt;
        body.angular_velocity += angular_accel * dt;

        body.transform.position += body.linear_velocity * dt;

        let w_q = nalgebra::Quaternion::new(
            0.0,
            body.angular_velocity.x,
            body.angular_velocity.y,
            body.angular_velocity.z,
        );
        let q_dt = body.transform.rotation.as_ref() * w_q * 0.5 * dt;
        body.transform.rotation = UnitQuaternion::new_normalize(
            body.transform.rotation.as_ref() + q_dt
        );
    }
}
