use physix_capsule::{self, World, rigid_body::RigidBody, EventLog};
use nalgebra::Vector3;
use anyhow::Result;
use std::{fs::File, io::Write};
use sha2::Digest;

fn main() -> Result<()> {
    let mut world = World {
        bodies: vec![
            RigidBody::new_box("BodyA", Vector3::new(0.0, 5.0, 0.0), 1.0, Vector3::new(0.5, 0.5, 0.5)),
            RigidBody::new_box("BodyB", Vector3::new(0.0, 0.0, 0.0), 10000.0, Vector3::new(5.0, 0.5, 5.0)),
        ],
        time_step_count: 0,
    };
    world.bodies[0].linear_velocity = Vector3::new(0.0, -1.0, 0.0);

    let initial_state_bytes = physix_capsule::canonical_serialize(&world)?;
    let initial_state_hash = {
        let mut hasher = sha2::Sha256::new();
        hasher.update(&initial_state_bytes);
        format!("{:x}", hasher.finalize())
    };

    let mut event_log = EventLog {
        initial_state_hash,
        transforms: Vec::new(),
        total_steps: 0,
    };

    const SIM_STEPS: u64 = 120;
    println!("Starting deterministic simulation for {} steps...", SIM_STEPS);

    for _ in 0..SIM_STEPS {
        let transforms = physix_capsule::evolve_world(&mut world)?;
        event_log.transforms.extend(transforms);
    }
    event_log.total_steps = world.time_step_count;
    println!("Simulation complete. Total steps: {}", event_log.total_steps);

    let (cbor_bytes, content_hash) = physix_capsule::serialize_event_log(&event_log)?;

    let output_path = std::env::args().nth(1).unwrap_or("event_log.cbor".to_string());
    
    let mut file = File::create(&output_path)?;
    file.write_all(&cbor_bytes)?;

    println!("Wrote event log to: {}", output_path);
    println!("Content Hash: {}", content_hash);

    Ok(())
}
