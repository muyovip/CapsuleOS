use nalgebra::Vector3;
use crate::{World, rigid_body::RigidBody};

#[derive(Debug)]
pub struct Contact {
    pub body_id_a: String,
    pub body_id_b: String,
    pub contact_point_w: Vector3<f32>,
    pub normal_w: Vector3<f32>,
    pub penetration_depth: f32,
}

/// Implements deterministic collision detection.
pub fn detect_deterministic_collisions(bodies: &[RigidBody]) -> Vec<Contact> {
    let mut pairs = Vec::new();
    for i in 0..bodies.len() {
        for j in (i + 1)..bodies.len() {
            if bodies[i].inv_mass != 0.0 || bodies[j].inv_mass != 0.0 {
                pairs.push((i, j));
            }
        }
    }

    pairs.sort_by(|(i1, j1), (i2, j2)| {
        let id1_a = &bodies[*i1].id;
        let id1_b = &bodies[*j1].id;
        let id2_a = &bodies[*i2].id;
        let id2_b = &bodies[*j2].id;
        
        let pair1 = if id1_a < id1_b { (id1_a, id1_b) } else { (id1_b, id1_a) };
        let pair2 = if id2_a < id2_b { (id2_a, id2_b) } else { (id2_b, id2_a) };

        pair1.cmp(&pair2)
    });
    
    let mut contacts = Vec::new();
    for (i, j) in pairs {
        let body_a = &bodies[i];
        let body_b = &bodies[j];
        
        let dist = (body_a.transform.position - body_b.transform.position).norm();
        if dist < 1.0 {
            contacts.push(Contact {
                body_id_a: body_a.id.clone(),
                body_id_b: body_b.id.clone(),
                contact_point_w: (body_a.transform.position + body_b.transform.position) * 0.5,
                normal_w: (body_a.transform.position - body_b.transform.position).normalize(),
                penetration_depth: 1.0 - dist,
            });
        }
    }
    contacts
}

/// Implements the constraint solver (Sequential Impulse Stub).
pub fn solve_constraints(world: &mut World, contacts: &[Contact]) {
    const SOLVER_ITERATIONS: u32 = 4; 

    let mut body_map: std::collections::HashMap<&String, &mut RigidBody> = world.bodies
        .iter_mut()
        .map(|b| (&b.id, b))
        .collect();

    for _ in 0..SOLVER_ITERATIONS {
        for contact in contacts {
            let body_a = body_map.get_mut(&contact.body_id_a).unwrap();
            
            let restitution: f32 = 0.5;
            
            let v_rel_n = body_a.linear_velocity.dot(&contact.normal_w);
            
            let v_target = -restitution * v_rel_n;

            let impulse_magnitude = (v_target - v_rel_n) / body_a.inv_mass;
            
            let impulse = contact.normal_w * impulse_magnitude;

            body_a.linear_velocity += impulse * body_a.inv_mass;
        }
    }
}
