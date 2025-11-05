use physix_capsule::{World, rigid_body::RigidBody, evolve_world, serialize_event_log, EventLog};
use nalgebra::Vector3;
use anyhow::Result;
use sha2::Digest;

#[test]
fn test_deterministic_simulation() -> Result<()> {
    fn run_simulation() -> Result<String> {
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

        const SIM_STEPS: u64 = 60;
        for _ in 0..SIM_STEPS {
            let transforms = evolve_world(&mut world)?;
            event_log.transforms.extend(transforms);
        }
        event_log.total_steps = world.time_step_count;

        let (_, content_hash) = serialize_event_log(&event_log)?;
        Ok(content_hash)
    }

    let hash1 = run_simulation()?;
    let hash2 = run_simulation()?;
    
    assert_eq!(hash1, hash2, "Determinism violated: two identical simulations produced different event log hashes");
    
    println!("✓ Deterministic simulation test passed");
    println!("  Hash (run 1): {}", hash1);
    println!("  Hash (run 2): {}", hash2);
    
    Ok(())
}

#[test]
fn test_graph_node_transform_hash_stability() -> Result<()> {
    use physix_capsule::GraphNodeTransform;
    use physix_capsule::rigid_body::Transform;
    use nalgebra::UnitQuaternion;

    let transform = Transform {
        position: Vector3::new(1.0, 2.0, 3.0),
        rotation: UnitQuaternion::identity(),
    };

    let node1 = GraphNodeTransform {
        body_id: "TestBody".to_string(),
        transform,
        timestamp: 42,
        state_hash: String::new(),
    };

    let node2 = GraphNodeTransform {
        body_id: "TestBody".to_string(),
        transform,
        timestamp: 42,
        state_hash: String::new(),
    };

    let hash1 = node1.compute_hash()?;
    let hash2 = node2.compute_hash()?;

    assert_eq!(hash1, hash2, "GraphNodeTransform hashing is not deterministic");
    
    println!("✓ GraphNodeTransform hash stability test passed");
    println!("  Hash: {}", hash1);
    
    Ok(())
}

#[test]
fn test_event_log_serialization_roundtrip() -> Result<()> {
    use physix_capsule::{serialize_event_log, replay_event_log};

    let mut world = World {
        bodies: vec![
            RigidBody::new_box("BodyA", Vector3::new(0.0, 5.0, 0.0), 1.0, Vector3::new(0.5, 0.5, 0.5)),
        ],
        time_step_count: 0,
    };

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

    for _ in 0..10 {
        let transforms = evolve_world(&mut world)?;
        event_log.transforms.extend(transforms);
    }
    event_log.total_steps = world.time_step_count;

    let (cbor_bytes, original_hash) = serialize_event_log(&event_log)?;
    let (replayed_log, replayed_hash) = replay_event_log(&cbor_bytes)?;

    assert_eq!(original_hash, replayed_hash, "Event log hash mismatch after roundtrip");
    assert_eq!(event_log.total_steps, replayed_log.total_steps, "Total steps mismatch after roundtrip");
    
    println!("✓ Event log serialization roundtrip test passed");
    println!("  Original hash:  {}", original_hash);
    println!("  Replayed hash:  {}", replayed_hash);
    
    Ok(())
}
