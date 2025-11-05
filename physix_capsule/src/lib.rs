use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

pub mod rigid_body;
mod solver;

use rigid_body::{RigidBody, Transform};

/// The fixed time step for deterministic simulation.
const FIXED_DT: f32 = 1.0 / 60.0;
/// The deterministic prefix for content hashing the event log.
const CONTENT_HASH_PREFIX: &[u8] = b"PhysixV1";


// --- Γ Integration: Graph Node Transform ---

/// Represents the state of a single RigidBody at a given time step.
/// This structure fulfills the requirement for state evolution as a GraphNode transform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNodeTransform {
    /// Canonical name/ID of the body (e.g., "CubeA").
    pub body_id: String,
    /// The body's new transformation (position and orientation).
    pub transform: Transform,
    /// The global deterministic time index.
    pub timestamp: u64, 
    /// Hash of the serialized transform, used for lineage tracking in a larger Graph.
    pub state_hash: String,
}

impl GraphNodeTransform {
    /// Computes the canonical state hash for this node.
    pub fn compute_hash(&self) -> Result<String> {
        #[derive(Serialize)]
        struct HashableTransform<'a> {
            body_id: &'a str,
            transform: &'a Transform,
            timestamp: u64,
        }
        let hashable = HashableTransform {
            body_id: &self.body_id,
            transform: &self.transform,
            timestamp: self.timestamp,
        };
        
        let bytes = canonical_serialize(&hashable)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        Ok(format!("{:x}", hasher.finalize()))
    }
}


// --- Simulation State and Event Log ---

/// The central state of the physics world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub bodies: Vec<RigidBody>,
    pub time_step_count: u64,
}

/// A log of all state transitions, required for deterministic replay.
#[derive(Debug, Serialize, Deserialize)]
pub struct EventLog {
    /// Metadata about the simulation run.
    pub initial_state_hash: String,
    /// The sequence of all GraphNode transforms.
    pub transforms: Vec<GraphNodeTransform>,
    /// The total elapsed time steps.
    pub total_steps: u64,
}


// --- Core Capsule API ---

/// Implements one deterministic fixed time-step of the physics simulation ($\Phi_\Gamma$).
pub fn evolve_world(world: &mut World) -> Result<Vec<GraphNodeTransform>> {
    // 1. Integrator (Semi-implicit Euler)
    rigid_body::integrate(world);
    
    // 2. Collision Detection (Deterministic Ordering)
    let potential_contacts = solver::detect_deterministic_collisions(&world.bodies); 
    
    // 3. Constraint Solver (Sequential Impulse)
    solver::solve_constraints(world, &potential_contacts); 

    // 4. Update Time
    world.time_step_count += 1;
    
    // 5. Emit GraphNode Transforms
    let mut transforms = Vec::new();
    for body in &world.bodies {
        let mut transform_node = GraphNodeTransform {
            body_id: body.id.clone(),
            transform: body.transform,
            timestamp: world.time_step_count,
            state_hash: String::new(),
        };
        transform_node.state_hash = transform_node.compute_hash()?;
        transforms.push(transform_node);
    }
    
    Ok(transforms)
}

/// Helper function to perform canonical CBOR serialization.
pub fn canonical_serialize<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    ciborium::ser::into_vec(value)
        .context("Failed to perform canonical CBOR serialization")
}

/// Serializes the entire EventLog to canonical CBOR and returns the hash.
pub fn serialize_event_log(log: &EventLog) -> Result<(Vec<u8>, String)> {
    let serialized_bytes = canonical_serialize(log)?;
    
    let mut hasher = Sha256::new();
    hasher.update(CONTENT_HASH_PREFIX);
    hasher.update(&serialized_bytes);
    let hash = format!("{:x}", hasher.finalize());

    Ok((serialized_bytes, hash))
}

/// Deserializes the EventLog from CBOR bytes and verifies its content hash.
pub fn replay_event_log(cbor_bytes: &[u8]) -> Result<(EventLog, String)> {
    // 1. Hash the incoming bytes to verify integrity/identity
    let mut hasher = Sha256::new();
    hasher.update(CONTENT_HASH_PREFIX);
    hasher.update(cbor_bytes);
    let incoming_hash = format!("{:x}", hasher.finalize());

    // 2. Deserialize the log
    let log: EventLog = ciborium::de::from_reader(cbor_bytes)
        .context("Failed to deserialize EventLog from CBOR")?;

    Ok((log, incoming_hash))
}
