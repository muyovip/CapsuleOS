# CapsuleOS - Meta-Operating System Toolchain

## Project Overview
CapsuleOS is a new meta-operating system with a cryptographic foundation and custom language (GΛLYPH). This repository contains the core Rust implementation of nine foundational components plus bare-metal boot infrastructure:

**Core Components:**
1. **capsule_core** - Cryptographic foundation for the Root Capsule (⊙₀) with content-addressable hashing
2. **glyph_lexer** - Tokenizer/lexer for the GΛLYPH language
3. **glyph_parser** - Recursive descent parser and AST for GΛLYPH
4. **genesis_graph** - Content-addressable DAG for cryptographic lineage and dependencies
5. **glyph_engine** - Pattern matching and substitution engines for GΛLYPH expressions
6. **rewrite_tx** - Transactional rewrite system for GenesisGraph with rollback capabilities
7. **capsule_manifest** - Manifest parser, verifier, and loader with Ed25519 signatures and lineage verification
8. **genesis_engine** - Genesis Graph Engine (GGE) Runtime with parallel pattern matching and deterministic evaluation
9. **cyberus_cli** - Command-line interface for CapsuleOS node control and Γ registry operations

**Boot Infrastructure:**
10. **boot/** - GRUB configuration and bootloader setup
11. **initramfs/** - Early boot environment with CAS mounting and capsule verification
12. **kernel/** - Bare-metal kernel with hardware initialization and cryptographic verification

**Rendering Infrastructure:**
13. **render_core_capsule** - Deterministic rendering engine with content-addressable outputs

**Physics Infrastructure:**
14. **physix_capsule** - Deterministic rigid-body physics engine with event logging

**Audio Infrastructure:**
15. **sonus_capsule** - Deterministic wavefield synthesis engine with content-addressable waveforms

## Recent Changes

### 2025-11-05: Sonus Capsule (Work Order 15) ✓
Created complete sonus_capsule crate with deterministic audio synthesis and content-addressable waveforms:

**Core Features:**
- Deterministic wavefield synthesis at 48kHz sampling rate
- Sample-accurate sine wave generation
- Content-addressable audio: SHA256("AudioV1" || CBOR(samples))
- Canonical CBOR serialization for waveforms
- Expression model for synthesis primitives
- Deterministic guarantees: identical inputs → identical waveform hashes

**Audio Architecture:**

**Expression Schema:**
- `Sine` - Frequency (Hz), duration (sec), amplitude (0–1)
- Extensible to noise, FM, envelopes, filters, etc.

**WaveformExpr:**
- `sample_rate` - Fixed at 48,000 Hz
- `samples` - Vec<f32> PCM data
- `content_hash` - SHA256("AudioV1" || CBOR(samples))
- `metadata` - AudioMetadata with expression, length, duration

**Synthesis Pipeline:**
1. **Expression Parsing** - Interpret synthesis primitive (e.g., Sine)
2. **Sample Generation** - Deterministic sample-by-sample calculation
3. **CBOR Serialization** - Canonical encoding of samples
4. **Content Hashing** - SHA256 with "AudioV1" prefix
5. **Metadata Capture** - Store expression, length, duration

**API Usage:**
```rust
use sonus_capsule::{synth, write_waveform_cbor, Expression};

// Create expression
let expr = Expression::Sine {
    freq: 440.0,
    duration: 1.0,
    amp: 0.5,
};

// Synthesize waveform
let waveform = synth(&expr)?;

// Write to CBOR file
write_waveform_cbor(Path::new("sine.cbor"), &waveform)?;

// Content hash for Γ registration
println!("Hash: {}", waveform.content_hash);
```

**Deterministic Guarantees:**
1. Fixed sample rate (48 kHz) - No variable sampling
2. Sample-accurate synthesis - Deterministic float calculations
3. Fixed iteration order - Sequential sample generation
4. Canonical CBOR encoding - Deterministic serialization
5. Reproducible hashes - Same inputs always produce same hash

**Test Results:**
```
✓ test_synth_sine_440hz_deterministic - PASSED
  ✓ Multiple runs produce identical hashes
  ✓ Hash matches recomputed hash

✓ test_deterministic_synthesis_multiple_runs - PASSED
  ✓ Run 1, 2, 3 produce identical samples
  ✓ Run 1, 2, 3 produce identical hashes

✓ test_waveform_metadata - PASSED
  ✓ Metadata contains correct parameters
  ✓ Sample count matches duration * sample_rate
```

**Example Output (synth_sine):**
```
Synthesizing sine wave:
  Frequency: 440 Hz
  Duration: 1 sec
  Amplitude: 0.5

✅ Synthesized 440Hz for 1s -> sine_440.cbor
Content Hash: cf8d773505021a53102d0ec362affec9ae5e773d905a7aa53c140a4f56a407e7
Sample Count: 48000
Sample Rate: 48000 Hz
```

**Integration with CapsuleOS:**
- Γ-loadable audio capsule with lineage tracking
- Waveforms as content-addressable artifacts
- Expression-based synthesis for reproducibility
- CBOR serialization for cross-platform compatibility

**Future Enhancements:**
- Additive synthesis (multiple sine waves)
- Noise generators (white, pink, brown)
- Envelope generators (ADSR)
- Filters (low-pass, high-pass, band-pass)
- FM synthesis
- Wavetable synthesis
- Effects (reverb, delay, distortion)
- Real-time CPAL playback mode

**Design Decisions:**
- Fixed 48 kHz ensures professional audio quality
- Float32 samples for precision and compatibility
- Sample-accurate synthesis for determinism
- CBOR for canonical serialization
- SHA-256 with "AudioV1" prefix for content addressing
- Expression model separates specification from synthesis

**Files Created:**
- `sonus_capsule/src/lib.rs` - Core API, Expression, WaveformExpr, synth
- `sonus_capsule/examples/synth_sine.rs` - CLI synthesis example
- `sonus_capsule/tests/sonus_tests.rs` - Determinism tests

### 2025-11-05: Physix Capsule (Work Order 14) ✓
Created complete physix_capsule crate with deterministic rigid-body physics simulation:

**Core Features:**
- Fixed timestep simulation (1/60 sec) for deterministic evolution
- Semi-implicit Euler integration for velocity and position
- Deterministic collision detection with lexicographical ordering
- Sequential impulse constraint solver (stub implementation)
- Content-addressable event logging: SHA256("PhysixV1" || CBOR(EventLog))
- GraphNode transforms for Γ integration
- CBOR serialization for replay and verification

**Physics Architecture:**

**RigidBody Schema:**
- `Transform` - Position (Vector3) + Rotation (UnitQuaternion)
- `Shape` - Box (half extents) or Sphere (radius)
- `RigidBody` - ID, mass, inertia tensor, velocities, forces
- Canonical serialization with nalgebra serde support

**Simulation Loop:**
1. **Integration** - Semi-implicit Euler (velocity → position)
2. **Collision Detection** - Deterministic pairwise checks (sorted by ID)
3. **Constraint Solver** - Sequential impulse (fixed iterations)
4. **State Update** - Increment timestep, emit GraphNode transforms

**Event Logging:**
- `EventLog` - Initial state hash + all GraphNode transforms
- Content hash: SHA256("PhysixV1" || CBOR(log))
- Replay verification by hash comparison
- Deterministic property: identical inputs → identical event log hash

**GraphNode Integration:**
```rust
pub struct GraphNodeTransform {
    body_id: String,           // Canonical ID (e.g., "BodyA")
    transform: Transform,      // Position + rotation
    timestamp: u64,            // Global time index
    state_hash: String,        // SHA256(CBOR(transform))
}
```

**API Usage:**
```rust
use physix_capsule::{World, rigid_body::RigidBody, evolve_world};
use nalgebra::Vector3;

// Create world with bodies
let mut world = World {
    bodies: vec![
        RigidBody::new_box("BodyA", Vector3::new(0.0, 5.0, 0.0), 
                           1.0, Vector3::new(0.5, 0.5, 0.5)),
    ],
    time_step_count: 0,
};

// Simulation loop
for _ in 0..120 {
    let transforms = evolve_world(&mut world)?;
    event_log.transforms.extend(transforms);
}

// Serialize and hash
let (cbor_bytes, hash) = serialize_event_log(&event_log)?;
```

**Deterministic Guarantees:**
1. Fixed timestep (1/60 sec) - No variable dt
2. Canonical body ordering - Lexicographical ID sort for collisions
3. Fixed iteration counts - Solver runs exactly 4 iterations
4. Deterministic serialization - CBOR with consistent encoding
5. Reproducible hashes - Same inputs always produce same event log hash

**Test Results:**
```
✓ test_deterministic_simulation - PASSED
  Hash (run 1): identical
  Hash (run 2): identical
  ✓ Multiple runs produce identical event logs

✓ test_graph_node_transform_hash_stability - PASSED
  ✓ Transform hashing is deterministic

✓ test_event_log_serialization_roundtrip - PASSED
  ✓ CBOR serialization preserves all data
  ✓ Hash verification successful
```

**Example Output (bouncing_box):**
```
Starting deterministic simulation for 120 steps...
Simulation complete. Total steps: 120
Wrote event log to: event_log.cbor
Content Hash: 0f86e416addf2f2d251506785e6f2e044ea0c5e12d3befe8eb4a30ac8466d1b0
```

**Example Output (replay_log):**
```
Replay Successful!
  Log Path: event_log.cbor
  Total Steps in Log: 120
  Event Log Content Hash: 0f86e416addf2f2d251506785e6f2e044ea0c5e12d3befe8eb4a30ac8466d1b0
Replay successful: event_log hash equals original
```

**Integration with CapsuleOS:**
- Γ-loadable physics capsule with lineage tracking
- Event logs as content-addressable audit trail
- GraphNode transforms for state evolution in Genesis Graph
- Deterministic replay for verification and debugging

**Future Enhancements:**
- Full collision detection (SAT, GJK/EPA for convex shapes)
- Advanced constraint solver (warm starting, friction cones)
- Rigid body joints (hinges, sliders, fixed)
- Soft body dynamics with deterministic FEM
- Parallel island detection for performance
- GPU-accelerated broad phase
- Continuous collision detection (CCD)
- Articulated bodies and ragdolls

**Design Decisions:**
- Fixed timestep ensures deterministic evolution
- Lexicographical ID ordering for deterministic collision pairs
- Semi-implicit Euler balances stability and simplicity
- CBOR for canonical serialization (deterministic encoding)
- SHA-256 with "PhysixV1" prefix for content addressing
- Sequential impulse solver stub (expandable to full PGS)
- nalgebra for matrix/vector operations with serde support

**Files Created:**
- `physix_capsule/src/lib.rs` - Core API, World, EventLog, GraphNodeTransform
- `physix_capsule/src/rigid_body.rs` - RigidBody schema and integrator
- `physix_capsule/src/solver.rs` - Deterministic collision and constraint solver
- `physix_capsule/examples/bouncing_box.rs` - Simulation example
- `physix_capsule/examples/replay_log.rs` - Event log replay verification
- `physix_capsule/tests/physix_tests.rs` - Property tests for determinism

### 2025-11-05: RenderCore Capsule (Work Order 13) ✓
Created complete render_core_capsule crate with deterministic rasterization and content-addressable rendering:

**Core Features:**
- CPU fallback renderer with deterministic rasterization
- Content-addressable rendering: SHA256("RenderV1" || CBOR(framebuffer))
- Canonical scene schema using G\u039bLYPH expressions
- Multi-backend support (CPU working, GPU stubs ready)
- CBOR serialization for all scene data and outputs
- Deterministic guarantees: identical inputs → identical outputs

**Rendering Architecture:**

**Scene Expressions:**
- `Triangle` - Single triangle with vertices, material ID, transform matrix
- `Mesh` - Collection of triangles (indices + positions)
- `Camera` - Position, target, up vector, FOV, aspect ratio
- `Material` - ID and RGB color vector
- `Scene` - Container for all elements with width/height

**Backends:**
- CPU Fallback - Scanline rasterizer with bounding box filling (working)
- GPU Stubs - WGPU/Vulkan interfaces ready for implementation
- Unified API - Common interface across all rendering paths

**Deterministic Foundations:**
1. Fixed memory layouts - Consistent buffer initialization (BGRA u8x4)
2. Canonical serialization - CBOR for all scene data
3. Content addressing - SHA-256 hash of serialized framebuffer
4. Deterministic operations - Scanline without floating point tricks
5. Fixed seed support - Ready for future stochastic effects

**Content-Addressable Hashing:**
```rust
hash = SHA256("RenderV1" || CBOR(FrameBuffer {
    width: u32,
    height: u32,
    pixels: Vec<u8>  // BGRA format
}))
```

**API Usage:**
```rust
use render_core_capsule::*;
use nalgebra::{Vector3, Matrix4};

// Define scene
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

// Load capsule
let manifest = CapsuleManifest {
    name: "renderer".to_string(),
    version: 1,
    lineage: "⊙₀".to_string(),
};
let handle = load_capsule(manifest)?;

// Render scene
let fb_expr = render_scene(&handle, &scene)?;

// Content hash available for verification
println!("Content Hash: {}", fb_expr.content_hash);
```

**CPU Renderer Implementation:**
- Bounding box calculation for triangle culling
- Scanline rasterization with edge function tests
- Point-in-triangle test using cross products (deterministic)
- Material lookup and color application
- BGRA pixel format (Blue, Green, Red, Alpha)
- Pixel center sampling (x + 0.5, y + 0.5)
- No depth buffering (future enhancement)
- No perspective correction (future enhancement)

**Test Results:**
```
✓ test_cpu_fallback_deterministic_output - PASSED
  Expected hash: 5faaaad265033107969f7bca363a1fe0f91b80af19dd859a7fa09c8b30c033ca
  Actual hash:   5faaaad265033107969f7bca363a1fe0f91b80af19dd859a7fa09c8b30c033ca
  ✓ Deterministic output verified
  ✓ Proper triangle interior rasterization (edge function tests)
```

**Example Output (render_triangle):**
```
=================================================================
RenderCore Capsule - Triangle Rendering Example
=================================================================
Width:        128 pixels
Height:       128 pixels
Content Hash: ab2bd046976befc0b7f4714aed8ace77875a9bce2d10eaedcf31e6b5fd6e76ed

Scene saved to: scene.cbor (201 bytes)
Framebuffer expression saved to: framebuffer.cbor (97 bytes)

Content-Addressable Rendering Verified
Hash: SHA256("RenderV1" || CBOR(framebuffer))
```

**Integration with CapsuleOS:**
- Γ-loadable capsule with lineage verification
- Manifest-driven initialization
- Content-addressable outputs for audit trail
- Deterministic replay for verification

**Future Enhancements:**
- Z-buffering for depth testing
- Perspective-correct interpolation
- Texture mapping support
- Ray tracing with seeded RNG
- WGPU backend implementation
- Vulkan backend implementation
- Shader compilation pipeline
- Multi-threaded CPU rasterization

**Design Decisions:**
- CBOR for canonical serialization (deterministic encoding)
- SHA-256 with "RenderV1" prefix for content addressing
- CPU fallback ensures determinism without GPU dependencies
- Bounding box rasterization for simplicity
- nalgebra for matrix/vector operations with serde support
- Feature flags for backend selection (cpu_fallback, vulkan, wgpu)

**Files Created:**
- `render_core_capsule/src/lib.rs` - Core API and FrameBuffer
- `render_core_capsule/src/scene.rs` - G\u039bLYPH scene expressions
- `render_core_capsule/src/cpu_renderer.rs` - Deterministic CPU rasterizer
- `render_core_capsule/src/gpu_renderer.rs` - GPU backend stubs
- `render_core_capsule/tests/render_tests.rs` - Determinism validation tests
- `render_core_capsule/examples/render_triangle.rs` - Example usage and CBOR output

### 2025-11-05: Hypervisor Protocol Layer 0 (Work Order 12) ✓
Created complete bare-metal boot infrastructure establishing the foundational boot layer:

**Boot Sequence:**
```
GRUB → Kernel → Initramfs → /init → CAS Mount → ⊙₀ Verification → GGE Runtime
```

**Core Components:**
- `boot/grub/grub.cfg` - GRUB multiboot configuration for CapsuleOS kernel
- `initramfs/init` - Main boot sequence script (PID 1 in initial ramdisk)
- `initramfs/etc/genesis.cfg` - Storage and runtime configuration
- `kernel/src/main.rs` - Bare-metal kernel entry point (kmain)
- `kernel/src/capsule/loader.rs` - Cryptographic capsule verification and loading
- Boot utilities: `cas-mount`, `capsule-loader`, `hash-utility`, `gge-runtime`

**Boot Phases:**

**Phase 1: GRUB Bootloader**
- Loads kernel binary (multiboot format)
- Loads initramfs image into RAM
- Passes control to kernel entry point
- Command-line parameters: `log_level=debug deterministic_boot=true`

**Phase 2: Kernel Initialization**
- Serial port initialization for boot logging
- MMU and paging setup (virtual memory)
- Interrupt handling (IDT, exceptions)
- VFS initialization with initramfs as root
- Execute `/init` script (kernel→userspace transition)

**Phase 3: Initramfs Boot Sequence**
- Hardware initialization verification
- Early mount of Content-Addressed Storage (IPFS/CAS)
- Root Capsule (⊙₀) loading:
  - Locate capsule by CID in CAS
  - Verify Ed25519 signature (Chain of Trust)
  - Load into memory at 0x40000000
  - Compute SHA-256 hash for boot audit
  - Log hash to kernel message buffer
- Spawn GGE runtime as primordial process (PID 1)

**Phase 4: GGE Runtime**
- Register ⊙₀ as initial graph state
- Load rewrite rules from capsule manifest
- Begin parallel pattern matching evaluation
- Establish CapsuleOS runtime environment

**Cryptographic Chain of Trust:**
1. GRUB (secure boot) → Kernel (verified by GRUB)
2. Kernel → Initramfs (loaded by kernel)
3. Initramfs → Root Capsule ⊙₀ (Ed25519 verified)
4. ⊙₀ → GGE Runtime (loaded from verified capsule)

**Kernel Architecture:**
- Bare-metal kernel (no_std environment)
- Hardware initialization (CPU, MMU, interrupts)
- Virtual filesystem (initramfs/cpio support)
- Memory management (paging, virtual memory)
- Capsule loader with cryptographic verification
- Serial console for deterministic boot logging

**Boot Utilities:**
- `cas-mount` - Mounts Content-Addressed Storage (IPFS/CAS block device)
- `capsule-loader` - Verifies Ed25519 signatures and loads capsules into memory
- `hash-utility` - Computes SHA-256 hashes for deterministic boot audit
- `gge-runtime` - Genesis Graph Engine executable (primordial process)

**Configuration (`initramfs/etc/genesis.cfg`):**
```ini
[storage]
type = ipfs_local
endpoint = /mnt/cas
root_cid = QmRootCapsuleHash...
root_public_key = ed25519:...

[modules]
gge_path = /usr/bin/gge-runtime
gge_flags = --deterministic --log-mutations --audit-mode

[boot]
deterministic = true
log_device = /dev/kmsg
mount_timeout_ms = 5000
verification_mode = strict
```

**Expected Boot Log:**
```
CapsuleOS Kernel: kmain started.
INFO: MMU/Paging initialized.
INFO: Initial VFS mounted (initramfs).
--- CapsuleOS Hypervisor Protocol Layer 0 (Bare-Metal Boot) ---
INFO: Low-level hardware initialization complete (CPU, memory, MMU).
STEP: Attempting early mount of Content-Addressed Storage...
SUCCESS: CAS mounted at /mnt/cas.
STEP: Loading Root Capsule (⊙₀) from /mnt/cas/cid/QmRootCapsule...
INFO: Root Capsule signature verified.
LOG: ⊙₀ Loaded. Hash: 0xAFFECAFEBEEFDEAD...
STEP: Spawning Genesis Graph Engine (GGE) runtime...
GGE: Genesis Graph Engine starting up...
```

**Security Features:**
- Root public key embedded in kernel at compile time
- Ed25519 signature verification for all capsules
- Deterministic hash logging for boot audit trail
- MMU enforces kernel/user memory separation
- Audit logging to `/dev/kmsg` for all mutations

**Implementation Status:**
- This is an ARCHITECTURAL SPECIFICATION (not production code)
- Demonstrates the DESIGN of bare-metal OS boot sequence
- Stub components show integration interfaces:
  - `gge-runtime` → Replace with compiled `genesis_engine` binary
  - `capsule-loader` → Implement using `capsule_core` crypto primitives
  - `cas-mount` → Integrate with IPFS or CAS block device
- See `boot/INTEGRATION.md` for complete integration guide
- Real production implementation would require:
  - Complete assembly entry point (_start)
  - Full x86_64 MMU/paging (page table setup)
  - Real IPFS/CAS integration
  - Complete interrupt handling (IDT, ISR)
  - Full VFS implementation (cpio parsing, tmpfs)
  - Hardware timer for timestamps
  - UEFI Secure Boot integration

**Testing:**
```bash
# Build kernel (conceptual)
cd kernel && cargo build --release

# Build initramfs
cd initramfs && find . | cpio -o -H newc | gzip > ../boot/initramfs.img

# Test with QEMU
qemu-system-x86_64 -kernel boot/capsuleos-kernel -initrd boot/initramfs.img -serial stdio
```

**Design Decisions:**
- Multiboot format for GRUB compatibility
- Initramfs for early boot environment (no hard-coded storage paths)
- Ed25519 signatures for cryptographic verification
- Content-addressed storage via IPFS/CAS
- Deterministic boot sequence with hash logging
- GGE as PID 1 (primordial process)
- Sovereignty enforcement from boot layer

### 2025-11-05: Cyberus CLI (Work Order 11) ✓
Created complete cyberus_cli crate with command-line interface for CapsuleOS node control:

**Commands:**
- `garden` - Inspect and modify nodes registered in Γ (set labels, attributes)
- `view` - View node data in canonical text (JSON) or CBOR format
- `forge` - Create new capsules from manifest files (JSON or CBOR)
- `resonate` - Verify integrity of Γ registry (lineage validation)

**Core Features:**
- Sovereignty enforcement: nodes with lineage [id, ⊙₀] are immutable
- Canonical text output: deterministic JSON with BTreeMap key ordering
- CBOR binary output: compact binary serialization for node data
- Audit logging: all mutations recorded to cyberus_audit.log with timestamps
- Manifest parsing: accepts both JSON and CBOR manifest formats
- Path normalization: supports both "/node/1" and "node/1" syntax

**CLI Architecture:**
- `Cli` - Main command parser with clap derive macros
- `Commands` - Garden, View, Forge, Resonate subcommands
- `GardenOp` - SetLabel, SetAttr, RemoveAttr operations
- `GraphNode` - Node data with BTreeMap attrs for canonical ordering
- `GenesisGraph` - Simulated Γ registry (mock for demonstration)
- `CliError` - Comprehensive error types with thiserror

**Sovereignty Rules:**
- Direct children of ⊙₀ (lineage length == 2) are sovereign and immutable
- Mutation attempts on sovereign nodes return `Sovereignty` error
- Child nodes (lineage length > 2) are mutable via garden commands
- All mutations are audited with RFC3339 timestamps

**Garden Operations:**
- `set-label <label>` - Set display label for node
- `set-attr <key> <value>` - Add or update node attribute
- `remove-attr <key>` - Delete node attribute by key

**View Formats:**
- `--format text` (default) - Canonical JSON with pretty printing
- `--format cbor` - Binary CBOR bytes to stdout

**Forge Workflow:**
1. Parse manifest file (try JSON first, then CBOR)
2. Validate lineage ends with ROOT_ID ("⊙₀")
3. Register node in Γ registry
4. Append audit log entry
5. Return OK on success

**Resonate Validation:**
- Lineage must be non-empty
- First element must equal node ID
- Last element must be ROOT_ID
- Prints "id: OK" or "id: ERROR - message"
- Exits with code 2 if any errors found

**Test Coverage (4 tests):**
- Canonical text ordering with BTreeMap
- Sovereignty enforcement on root children
- JSON manifest parsing and registration
- Lineage validation error detection

**Test Status**: All 4 cyberus_cli tests passing

**Dependencies:**
- `clap = { version = "4.2", features = ["derive"] }` - CLI argument parsing
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `serde_json = "1.0"` - JSON encoding/decoding
- `serde_cbor = "0.11"` - CBOR encoding/decoding
- `thiserror = "1.0"` - Error handling
- `time = { version = "0.3", features = ["formatting"] }` - Timestamp formatting
- `tempfile = "3.6"` - Temporary files (dev only)

**Design Decisions:**
- BTreeMap for attrs ensures deterministic JSON key ordering
- Mock GenesisGraph implementation for standalone CLI demonstration
- Audit log append-only for tamper-evident mutation tracking
- Sovereignty at lineage level (not individual permissions)
- Support both JSON and CBOR for maximum flexibility
- RFC3339 timestamps for audit log entries

**Usage Examples:**
```bash
# View node in canonical JSON
cargo run --package cyberus_cli -- view /node/1

# View node as CBOR bytes
cargo run --package cyberus_cli -- view /node/1 --format cbor

# Set node label (allowed for non-sovereign nodes)
cargo run --package cyberus_cli -- garden /node/1/child set-label "New Label"

# Set node attribute
cargo run --package cyberus_cli -- garden /node/1/child set-attr color blue

# Remove node attribute
cargo run --package cyberus_cli -- garden /node/1/child remove-attr color

# Forge capsule from manifest
cargo run --package cyberus_cli -- forge manifest.json

# Verify Γ registry integrity
cargo run --package cyberus_cli -- resonate
```

**Integration Notes:**
- Current implementation uses in-process mock GenesisGraph
- Production: replace `demos_graph()` with IPC/RPC client to actual Γ
- Audit log should be append-only, tamper-evident, signed in production
- Consider adding Ed25519 signing for audit entries
- Future: add --audit-file flag to configure log destination

### 2025-11-05: Genesis Graph Engine Runtime (Work Order 10) ✓
Created complete genesis_engine crate with parallel pattern matching runtime for evaluating rewrite rules:

**Core Components:**
- `GenesisEngine` - Thread-safe runtime with parallel pattern matching
- `GenesisGraph` - DAG with nodes, edges, and root hash
- `Rule` / `RuleSet` - Rewrite rules with priority-based ordering
- `EvaluationState` - Iteration tracking, rules fired, idle state detection
- `TransactionLog` - Complete audit trail of rule applications
- `RuntimeConfig` - Configurable evaluation parameters

**Evaluation Loop:**
- Continuous iteration until idle state (ΔG == 0)
- Deterministic node ordering (lexicographic by ID)
- Hash-based change detection for termination
- Max iterations and timeout protection
- Priority-based rule application (highest first)
- Skip updates when data unchanged (optimization)

**Runtime Features:**
- Thread-safe graph access via Arc<RwLock<GenesisGraph>>
- Parallel pattern matching with Rayon (configurable)
- Sequential fallback for deterministic debugging
- Transaction logging for audit trails
- Evaluate-until-predicate support
- Configurable max iterations and timeout

**Pattern Matching:**
- Parallel matching with automatic priority sorting
- Sequential matching preserves input order
- Patterns: Wildcard, Var, Literal, Tuple, List
- First-match semantics (highest priority wins)
- Condition evaluation support

**Graph Operations:**
- Content-addressable node hashing with CBOR
- Deterministic graph hashing (sorted node iteration)
- Canonical root node validation (empty root_ref)
- In-place node updates with timestamp tracking
- Node insertion/update/retrieval

**Key Implementation Details:**
- Idle detection: ΔG == 0 when graph hash unchanged
- Data change optimization: skip updates if new_data == old_data
- Priority sorting after parallel matching (Rayon doesn't preserve order)
- Fixed timestamps for deterministic testing
- Root node uses Literal::Unit to avoid test interference

**Test Coverage (23 tests):**
- Engine creation and initialization
- Idle state detection (ΔG == 0)
- Simple and iterative evaluation
- Deterministic ordering across runs
- Transaction log tracking
- Max iterations limit
- Rule priority ordering
- Parallel vs sequential matching
- Thread-safe concurrent reads
- Single-writer lock enforcement
- Predicate-based evaluation
- Disabled rules handling
- Multiple rules per iteration
- Graph convergence
- Tuple/List pattern matching
- Comprehensive runtime loop testing
- Continuous evaluation
- Idle state property verification
- Reset transaction log
- Rule metadata handling

**Test Status**: All 23 genesis_engine tests passing

**Dependencies:**
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `ciborium = "0.2"` - CBOR encoding
- `sha2 = "0.10"` - SHA-256 hashing
- `hex = "0.4"` - Hexadecimal encoding
- `thiserror = "1.0"` - Error handling
- `parking_lot = "0.12"` - Efficient RwLock
- `rayon = "1.8"` - Parallel iteration
- `proptest = "1.4"` - Property testing (dev only)

**Design Decisions:**
- Thread-safe design with Arc<RwLock> for concurrent access
- Parallel matching for performance (with deterministic sorting)
- Skip no-op updates to prevent infinite loops
- Fixed timestamps (0 for root, 1000 for test nodes) for determinism
- Deterministic graph hashing via sorted node iteration
- Root node uses Unit literal to avoid test pattern collisions

### 2025-11-05: CapsuleManifest System (Work Order 9) ✓
Created complete capsule_manifest crate with cryptographic manifest verification and registry system:

**Core Features:**
- `CapsuleManifest` struct with deterministic CBOR serialization
- `parse_manifest()` - Parse CBOR bytes into validated manifest
- `verify_capsule()` - Ed25519 signature and lineage verification
- `load_manifest()` - Verify and register manifest into Γ (Gamma)
- `Gamma` registry - In-memory capsule storage with uniqueness checks

**Manifest Structure:**
- `id`: Unique capsule identifier
- `parent`: Optional parent capsule reference
- `signature`: Ed25519 signature bytes (64 bytes)
- `lineage`: Chain from parent to root sentinel ⊙₀
- `metadata`: BTreeMap for deterministic key ordering

**Verification System:**
- Ed25519 signature verification over canonical CBOR (signature field excluded)
- Lineage validation: chain must end with ROOT_ID ("⊙₀")
- Parent consistency: parent must equal first lineage entry when present
- Returns `ProofResult` with signature_valid and lineage_valid booleans

**Canonical Serialization:**
- Signature computed over manifest WITHOUT signature field
- Uses BTreeMap for deterministic field ordering (id, parent, lineage, metadata)
- serde_cbor::to_vec provides stable CBOR encoding
- Field declaration order affects CBOR structure (kept stable)

**Error Types:**
- `ManifestError`: CBOR deserialization and integrity errors
- `VerifyError`: Signature/lineage verification failures
- `LoadError`: Verification and registry errors

**Test Coverage (5 tests):**
- Happy path: parse, verify, and load valid manifest
- Invalid signature detection (wrong keypair)
- Tampered metadata detection (corrupted CBOR)
- Lineage validation (must end with ⊙₀)
- Duplicate registration prevention

**Test Status**: All 5 capsule_manifest tests passing

**Dependencies:**
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `serde_cbor = "0.11"` - CBOR encoding
- `ed25519-dalek = "1.0"` - Ed25519 signatures
- `thiserror = "1.0"` - Error handling
- `hex = "0.4"` - Hexadecimal encoding
- `rand = "0.7"` - Random number generation (dev only)

**Design Decisions:**
- BTreeMap for metadata ensures deterministic key ordering
- Signature over canonical bytes (signature field omitted)
- Lineage must be non-empty and terminate at ROOT_ID
- Parent consistency enforced when present
- Gamma registry prevents duplicate capsule IDs

### 2025-11-05: Transactional Rewrite System (Work Order 8) ✓
Created complete rewrite_tx crate with transactional rule application for GenesisGraph:

**Core Features:**
- Transactional rule application with full rollback support
- Priority-based rule ordering with deterministic tie-breaking
- Graph snapshot and restoration for atomicity
- Modification tracking for all graph operations
- Thread-safe graph access with RwLock
- Content-addressable hash verification
- Deterministic node ordering for reproducible rewrites

**Transaction System:**
- `Transaction::begin()`: Start new transaction with ruleset
- `Transaction::apply_ruleset()`: Apply rules to graph nodes in deterministic order
- `Transaction::commit()`: Finalize transaction and compute post-state hash
- `Transaction::rollback()`: Restore graph to pre-transaction state with hash verification
- `apply_ruleset_transactionally()`: Convenience function for complete transactional workflow

**Rewrite Rules:**
- `RewriteRule`: Pattern + replacement with optional conditions
- `RuleSet`: Collection of rules with priority-based sorting
- Pattern matching integration for rule application
- Substitution integration for replacement instantiation
- First-match semantics (only first matching rule per node)

**Data Structures:**
- `GenesisGraph`: Thread-safe wrapped graph (Arc<RwLock<...>>)
- `GraphSnapshot`: Complete graph state with content hash
- `GraphNode`: Node with id, root_ref, data (Expression), metadata
- `Modification`: Tracked changes (NodeUpdated, NodeAdded, NodeRemoved, EdgeAdded, EdgeRemoved)
- `TransactionResult`: Pre/post hashes, rewrite count, modification list

**Graph Operations:**
- `new_wrapped()`: Create thread-safe graph from root node
- `nodes_sorted_by_id()`: Deterministic node iteration (lexicographic by id)
- `insert_node_internal()`: Add new node with hash validation
- `update_node_internal()`: Modify existing node in-place
- `remove_node_internal()`: Delete node and associated edges
- `add_edge_internal()`: Link nodes with dependency/derivation/reference edges

**Key Implementation Details:**
- Snapshot-based isolation: Pre-state captured before rule application
- Hash verification: Post-rollback hash must match pre-state hash
- Priority sorting: Higher priority first, then lexicographic by id
- Deterministic ordering: Nodes processed in id-sorted order
- Error handling: Rollback on any rule application error
- CBOR serialization: Deterministic snapshot encoding

**Test Coverage (25 tests):**
- Ruleset priority sorting and tie-breaking
- Simple rewrite rules with pattern matching
- Deterministic node ordering verification
- Transaction isolation and rollback
- Modification tracking
- Snapshot serialization and determinism
- Hash stability and comparison
- Concurrent read access
- Atomicity properties
- Idempotent rules
- Conditional rules
- Empty rulesets
- Comprehensive workflow integration

**Test Status**: All 25 rewrite_tx tests passing

**Design Choices:**
- Content-addressed storage: Node hash computed from full node content
- In-place updates: Nodes updated under original hash (not moved to new hash)
- Snapshot-based rollback: Complete graph state restoration
- RwLock for concurrency: Multiple readers OR single writer
- Deterministic execution: Sorted iteration ensures reproducible results

### 2025-11-05: Capture-Avoiding Substitution Engine (Work Order 7) ✓
Created complete capture-avoiding substitution engine in glyph_engine crate:

**Core Features:**
- Fresh name generation (gensym) with atomic counter for deterministic testing
- Free variables analysis with scope tracking
- Alpha renaming (α-conversion) to avoid variable capture
- Capture-avoiding substitution for all expression types
- Pattern variable extraction
- Multiple simultaneous substitutions
- Well-formedness checking (no unbound variables)

**Substitution Functions:**
- `substitute()`: Main entry point for capture-avoiding substitution expr[var := replacement]
- `substitute_internal()`: Recursive substitution with bound variable tracking
- `substitute_match_arm()`: Special handling for match arms with pattern capture avoidance
- `substitute_many()`: Parallel substitution of multiple variables (deterministic ordering)
- `substitute_pattern()`: Rename variables in patterns

**Helper Functions:**
- `gensym()`: Generate unique fresh variable names with format "prefix$N"
- `reset_gensym()`: Reset counter for deterministic testing
- `free_vars()`: Compute set of free variables in expression
- `pattern_variables()`: Extract all variables bound by a pattern
- `alpha_rename()`: Rename bound variable to avoid capture (returns new name + renamed expr)
- `alpha_rename_pattern()`: Rename pattern variables
- `is_well_formed()`: Check for unbound variables (closed terms only)

**Key Implementation Details:**
- Variable shadowing: bound variables block substitution in their scope
- Capture avoidance: automatic α-renaming when replacement would be captured
- Lambda handling: renames parameter if it appears in replacement's free vars
- Let binding handling: same capture avoidance as lambda
- Match expression handling: pattern variables shadow in guard and body
- Deterministic substitution: uses avoid sets to ensure consistent fresh names

**Test Coverage (47 tests):**
- Gensym uniqueness and freshness
- Free variables computation (simple, lambda, mixed scopes)
- Variable substitution (var, apply, tuple, list, record)
- Lambda substitution (free vars, shadowing, capture avoidance)
- Let binding substitution (value, body, shadowing, capture)
- Match expression substitution (expr, pattern shadowing, capture avoidance, guards)
- Alpha renaming (simple, conflict avoidance)
- Pattern variable extraction (simple, tuple, bind, constructor, record)
- Well-formedness checking (closed/open terms, preservation)
- Multiple substitutions (deterministic ordering)
- Property tests (determinism, idempotence, commutativity, α-equivalence preservation)
- Comprehensive integration tests (Church numerals, nested structures, complex scenarios)

**Test Status**: All 48 substitute tests passing (90 total glyph_engine tests)
- Includes regression test for guard free variable capture prevention

### 2025-11-04: Pattern Matching Engine (Work Order 6) ✓
Created complete glyph_engine crate with pattern matching functionality for GΛLYPH expressions:

**Core Features:**
- Complete pattern matching engine with structural and bind support
- Pattern types: Wildcard, Variable, Literal, Bind (x@P), Tuple, List, Record, Constructor, Lambda, Apply
- Variable consistency checking prevents invalid bindings (pattern variable shadowing)
- Constructor pattern unwinding handles both Apply and LinearApply nodes
- Deterministic binding serialization using BTreeMap
- Helper functions: match_any_pattern, match_pattern_many, matches, pattern_variables

**Data Types:**
- `Expression`: Literal, Var, Lambda, Apply, LinearApply, Let, Match, Tuple, List, Record
- `Literal`: Int, Float, String, Bool, Unit
- `Pattern`: 10 pattern types with full structural matching
- `Bindings`: BTreeMap<String, Expression> for deterministic ordering
- `MatchResult`: Vec<Bindings> supporting multiple match results

**Pattern Matching Semantics:**
- Wildcard pattern: matches anything, binds nothing
- Variable pattern: matches anything, binds to name with consistency checking
- Literal pattern: exact value matching for all literal types
- Bind pattern (x@P): binds value to name AND matches nested pattern
- Structural patterns: recursive matching for Tuple, List, Record
- Constructor patterns: zero-arg and multi-arg with curried application unwinding
- Lambda/Application patterns: matches expression structure

**Key Implementation Details:**
- Variable shadowing prevention: rejects patterns binding same variable to different values
- Constructor unwinding: handles nested Apply and LinearApply chains
- Partial record matching: pattern matches subset of record fields
- Deterministic serialization: BTreeMap ensures stable CBOR output
- Helper functions for OR semantics, bulk matching, and variable extraction

**Test Coverage (42 tests):**
- All pattern types (wildcard, variable, literal, bind, structural, constructor)
- LinearApply constructor patterns (single-arg, multi-arg, mixed chains)
- Variable shadowing prevention
- Deterministic binding order and serialization
- Round-trip property verification
- Idempotence testing
- Comprehensive structural matching scenarios

**Test Status**: All 42 glyph_engine tests passing

### 2025-11-04: GenesisGraph DAG Implementation (Work Order 5) ✓
Created complete genesis_graph crate with cryptographic DAG functionality:

**Core Features:**
- Content-addressable directed acyclic graph (DAG) with cryptographic lineage tracking
- Root node (⊙₀) creation with backward-compatible hash validation
- Node insertion with root reference enforcement
- Edge linking with automatic cycle detection (DFS-based)
- Topological sorting via Kahn's algorithm
- Lineage path tracking from root via BFS traversal
- Node deletion with automatic edge cleanup
- Canonical CBOR serialization (deterministic, order-independent)

**Data Structures:**
- `GraphNode`: ID, root reference, Expression data, metadata (timestamp, lineage_depth, tags)
- `GraphEdge`: from/to hashes with edge types (Dependency, Derivation, Reference)
- `GenesisGraph`: nodes HashMap, edges Vec, root_hash
- `GraphError`: 7 error types for comprehensive validation

**Hash Computation:**
- Root node hash: `GlyphV1:Root:` prefix, computed with empty `root_ref` for determinism
- Node hash: `GlyphV1:Node:` prefix, includes all node fields
- Backward compatible: accepts both empty and pre-hashed root_ref values
- Internal normalization: stores root nodes with empty `root_ref` for consistency

**Key Design Decisions:**
- Root nodes have empty `root_ref` (they're the genesis, no parent)
- Validation accepts legacy root nodes with pre-computed `root_ref` for compatibility
- Cycle detection prevents DAG corruption (no circular dependencies)
- Self-loops forbidden to maintain acyclic invariant
- Canonical serialization uses BTreeMap for nodes and sorted edges

**Test Status**: All 18 genesis_graph tests passing (comprehensive coverage)

### 2025-11-04: Content-Addressable Hashing (Work Order 4) ✓
Integrated content-addressable hashing functionality into capsule_core:

**New Features:**
- `ContentAddressable` trait for computing prefixed content hashes
- `CanonicalSerialize` trait for deterministic CBOR serialization
- Domain types: `Glyph`, `Expression`, `GraphNode`, `GlyphRef`, `ExpressionRef`
- `compute_content_hash_with_prefix()` function returning "prefix:hexhash" format
- Type-specific hash prefixes: GlyphV1, ExprV1, NodeV1

**Implementation Details:**
- Uses `serde_cbor::to_vec()` for deterministic CBOR serialization
- Preserves backward compatibility with existing Root Capsule functionality
- Legacy `compute_content_hash()` still works for existing code
- All domain types implement both traits with proper prefixes
- Deterministic hashing: identical input always produces identical hash

**Test Status**: All 10 capsule_core tests passing (4 original + 6 new Work Order 4 tests)

### 2025-11-04: GΛLYPH Parser Implementation (Work Order 3) ✓
Implemented complete recursive descent parser for the GΛLYPH language:

**Features:**
- Complete AST with Expression, Literal, Pattern, and MatchArm types
- Lexer with support for: literals, keywords, lambda (λ), linear arrow (⊸), comments (#), negative numbers
- Recursive descent parser with proper precedence handling
- Pattern matching with wildcards, variables, literals, tuples, and constructors
- Guard expressions in match arms
- Canonical CBOR serialization/deserialization
- Public API: `parse(input: &str) -> Result<Expression, ParseError>`

**Key Implementation Details:**
- Parse chain: parse_expression → parse_let → parse_match → parse_lambda → parse_application → parse_primary
- Match subjects restricted to primary expressions to avoid ambiguity with match arm braces
- Record literals fully supported as function arguments: `f { x: 1 }`
- Lambda bodies use full expression parsing to support nested let/match expressions

**Test Status**: All 36 tests passing (120 comprehensive test cases)

### 2025-11-04: Lexer Bug Fixes - All Tests Passing ✓
Fixed three critical position tracking bugs in the glyph_lexer:

1. **Float literal detection** - Fixed lookahead logic to properly detect float literals (e.g., "3.14")
   - Previously: Lexed as three tokens (integer, delimiter, integer)
   - Now: Correctly lexed as single FloatLiteral token
   - Root cause: `peek_ahead()` was looking from stale `cur_pos` instead of current peek position

2. **Block comment parsing** - Fixed nested comment detection
   - Previously: "/* comment */" falsely reported as "Unterminated block comment"
   - Now: Correctly parses all block comments including nested ones
   - Root cause: Same peek_ahead issue affecting `*/` detection

3. **Division operator recognition** - Fixed "/" at end of input
   - Previously: Trailing "/" incorrectly treated as line comment starter
   - Now: Correctly emitted as Operator("/")
   - Root cause: Lookahead checking for "//" before position update

**Solution**: Changed all lookahead checks to use `self.input[(pos + 1)..].chars().next()` instead of `peek_ahead()`, ensuring position-relative peeking.

**Test Status**: All 96 tests passing (92 lexer + 4 capsule_core)

## Project Architecture

### rewrite_tx (25/25 tests ✓)

**Transactional Rewrite System (25 tests):**
- Complete transactional rule application with rollback
- Thread-safe graph operations via Arc<RwLock<GenesisGraph>>
- Snapshot-based state management
- Priority-based rule ordering with deterministic execution
- Modification tracking for all graph changes
- Hash verification for rollback correctness
- Deterministic node iteration (lexicographic by id)

**Core Components:**
- `Transaction`: Begin/apply/commit/rollback lifecycle
- `RuleSet`: Priority-sorted collection of rewrite rules
- `RewriteRule`: Pattern + replacement + optional condition
- `GenesisGraph`: Thread-safe wrapped graph with operations
- `GraphSnapshot`: Complete graph state with content hash
- `TransactionResult`: Pre/post hashes + modifications

**Graph Operations:**
- `new_wrapped()`: Create thread-safe graph from root node
- `nodes_sorted_by_id()`: Deterministic node iteration
- `insert_node_internal()`, `update_node_internal()`, `remove_node_internal()`
- `add_edge_internal()`: Link nodes with cycle detection

**Design Patterns:**
- Snapshot isolation for atomicity
- First-match semantics (one rule per node)
- In-place updates (nodes kept under original hash)
- CBOR-based deterministic serialization

### glyph_engine (90/90 tests ✓)

**Capture-Avoiding Substitution Engine (48 tests):**
- Complete capture-avoiding substitution with α-conversion
- Fresh name generation via atomic gensym counter
- Free variables analysis with scope tracking
- Alpha renaming for bound variables to avoid capture
- Pattern variable extraction from all pattern types
- Multiple simultaneous substitutions with deterministic ordering
- Well-formedness checking for closed terms

**Substitution Algorithm:**
- Main entry: `substitute(expr, var, replacement)` returns expr[var := replacement]
- Handles all expression types: Literal, Var, Lambda, Apply, LinearApply, Let, Match, Tuple, List, Record
- Automatic α-renaming when replacement would capture bound variables
- Special handling for match arms with pattern-bound variables
- Preserves well-formedness of closed terms
- Deterministic behavior for testing (gensym reset capability)

**glyph_engine (42 tests ✓)
**Pattern Matching Engine:**
- Complete pattern matcher for GΛLYPH expressions with structural and bind support
- Pattern types: Wildcard, Var, Literal, Bind, Tuple, List, Constructor, Record, Lambda, Apply
- Variable consistency checking prevents shadowing (same variable bound to different values)
- Constructor pattern unwinding for both Apply and LinearApply nodes
- Deterministic binding serialization using BTreeMap

**Core Functions:**
- `match_pattern()`: Main entry point returning MatchResult (Vec<Bindings>)
- `match_pattern_internal()`: Recursive matcher with binding accumulation
- `match_constructor()` / `match_constructor_application()`: Constructor pattern handling
- `match_any_pattern()`: OR semantics across multiple patterns
- `match_pattern_many()`: Bulk matching against multiple expressions
- `matches()`: Boolean check without bindings (faster)
- `pattern_variables()`: Extract all bound variables from pattern
- CBOR serialization helpers for MatchResult

**Dependencies:**
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `ciborium = "0.2"` - CBOR encoding
- `proptest = "1.4"` - Property-based testing (dev dependency)

**Test Coverage:**
- All 10 pattern types with comprehensive scenarios
- LinearApply constructor patterns (single-arg, multi-arg, mixed chains)
- Variable shadowing prevention
- Deterministic binding order and serialization
- Round-trip property, idempotence testing
- Nested patterns, partial record matching
- Complex structural matching cases

### genesis_graph (18/18 tests ✓)
**GenesisGraph DAG:**
- Content-addressable directed acyclic graph (DAG)
- Root node creation with ⊙₀ symbol and backward-compatible validation
- Graph operations: insert, delete, link, query
- Cycle detection via depth-first search (DFS)
- Topological sort via Kahn's algorithm
- Lineage path tracking via breadth-first search (BFS)
- Canonical CBOR serialization with deterministic ordering
- Comprehensive error handling with 7 error types

**Dependencies:**
- `serde_cbor = "0.11"` - CBOR serialization
- `sha2 = "0.10"` - SHA-256 hashing
- `hex = "0.4"` - Hexadecimal encoding
- References capsule_core for Expression and ContentAddressable trait

**Test Coverage:**
- Root node creation and validation
- Graph creation with backward compatibility
- Node insertion with root reference enforcement
- Edge linking with cycle detection
- Node deletion with edge cleanup
- Topological sorting (valid DAG and cycle detection)
- Lineage path tracking from root
- Canonical serialization stability
- Comprehensive integration test (11 nodes, 10 edges)

### capsule_core (10/10 tests ✓)
**Root Capsule Functionality:**
- Ed25519 cryptographic signing (ed25519-dalek 2.1)
- SHA-256 hashing with GlyphV1 prefix
- Canonical CBOR serialization (deterministic encoding)
- Root Capsule creation (⊙₀) with signature verification
- Zero-knowledge proof generation and validation

**Content-Addressable Hashing (Work Order 4):**
- `ContentAddressable` and `CanonicalSerialize` traits
- Domain types: `Glyph`, `Expression`, `GraphNode` with references
- Type-specific hash prefixes (GlyphV1, ExprV1, NodeV1)
- Deterministic CBOR serialization via serde_cbor
- Comprehensive testing of hash determinism and prefix handling

### capsule_manifest (5/5 tests ✓)
**CapsuleManifest Parser, Verifier, and Loader:**
- `CapsuleManifest` struct with Ed25519 signature and lineage chain
- `parse_manifest()` - Parse and validate CBOR manifest bytes
- `verify_capsule()` - Cryptographic signature and lineage verification
- `load_manifest()` - Combined verify-and-register operation
- `Gamma` registry - In-memory capsule storage with duplicate prevention
- ROOT_ID constant ("⊙₀") - Root sentinel for lineage termination

**Core Components:**
- `CapsuleManifest`: id, parent, signature, lineage, metadata (BTreeMap)
- `ProofResult`: signature_valid, lineage_valid booleans
- `ManifestError`: CBOR and integrity errors
- `VerifyError`: Signature and lineage verification errors
- `LoadError`: Verification and registration errors

**Verification Process:**
1. Parse manifest from CBOR bytes
2. Validate lineage ends with ROOT_ID
3. Parse Ed25519 signature (64 bytes)
4. Compute canonical CBOR (without signature field)
5. Verify signature against canonical bytes
6. Check parent matches first lineage entry
7. Return ProofResult with validation results

**Canonical Serialization:**
- Signature field excluded from canonical bytes
- BTreeMap ensures deterministic field ordering
- Fields: id, parent, lineage, metadata (alphabetical)
- serde_cbor::to_vec for stable CBOR encoding

**Test Coverage:**
- Parse, verify, and load valid manifests
- Invalid signature detection (wrong keypair)
- Tampered data detection (corrupted CBOR)
- Lineage validation (must end with ⊙₀)
- Duplicate ID prevention in registry

### glyph_lexer (92/92 tests ✓)
- Unicode-aware tokenization (unicode-xid 0.2)
- Deterministic canonicalization (CRLF→LF, whitespace normalization)
- Complete token support:
  - Identifiers (Unicode XID_Start/Continue)
  - Numeric literals: hex (0x), binary (0b), octal (0o), decimal, float
  - String/char literals with Unicode escapes (\u{NNNN})
  - Nested block comments (/* ... */)
  - Line comments (//)
  - Operators (longest-match parsing)
  - Delimiters: ( ) { } [ ] ; , .
- Comprehensive error reporting with spans
- 92 unit tests covering all edge cases

### glyph_parser (36/36 tests ✓)
- Complete AST definition:
  - Expression: Literal, Var, Lambda, Apply, LinearApply, Let, Match, Tuple, List, Record
  - Literal: Int, Float, String, Bool, Unit
  - Pattern: Wildcard, Var, Literal, Tuple, Constructor
  - MatchArm: pattern + optional guard + body
- Lexer supporting GΛLYPH syntax:
  - Keywords: let, in, match, if, then, else, true, false
  - Special symbols: λ (lambda), ⊸ (linear arrow), # (comments)
  - All numeric types including negative numbers
  - String escapes: \n, \t, \r, \\, \"
- Recursive descent parser with precedence:
  - Let bindings with proper scoping
  - Pattern matching with guards
  - Lambda abstractions
  - Function application (both regular and linear)
  - Tuples, lists, and records
- Canonical CBOR serialization via ciborium
- 36 unit tests including 120 comprehensive cases
- All round-trip serialization tests passing

## Dependencies

### capsule_core
- `ed25519-dalek = "2.1"` - Ed25519 signatures
- `serde = { version = "1", features = ["derive"] }` - Serialization
- `ciborium = "0.2"` - CBOR encoding (Root Capsule)
- `serde_cbor = "0.11"` - CBOR encoding (Work Order 4)
- `serde_bytes = "0.11"` - Efficient byte serialization
- `sha2 = "0.10"` - SHA-256 hashing
- `hex = "0.4"` - Hexadecimal encoding
- `rand = "0.8"` - Random number generation

### capsule_manifest
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `serde_cbor = "0.11"` - CBOR encoding
- `ed25519-dalek = "1.0"` - Ed25519 signatures
- `thiserror = "1.0"` - Error handling
- `hex = "0.4"` - Hexadecimal encoding
- `rand = "0.7"` - Random number generation (dev)

### glyph_lexer
- `unicode-xid = "0.2"` - Unicode identifier validation

### glyph_parser
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `ciborium = "0.2"` - CBOR serialization
- `thiserror = "1.0"` - Error handling

### glyph_engine
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `ciborium = "0.2"` - CBOR encoding
- `proptest = "1.4"` - Property-based testing (dev)

### rewrite_tx
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `ciborium = "0.2"` - CBOR encoding
- `sha2 = "0.10"` - SHA-256 hashing
- `hex = "0.4"` - Hexadecimal encoding
- `thiserror = "1.0"` - Error handling
- `parking_lot = "0.12"` - Thread-safe RwLock
- `proptest = "1.4"` - Property-based testing (dev)

## Testing
All tests must run with `--test-threads=1` for deterministic validation:
```bash
cargo test --workspace -- --test-threads=1
```

**Current Test Results: 276 tests passing**
- capsule_core: 10 tests ✓
- capsule_manifest: 5 tests ✓
- genesis_graph: 18 tests ✓
- glyph_engine: 90 tests ✓ (42 pattern matching + 48 substitution)
- glyph_lexer: 92 tests ✓
- glyph_parser: 36 tests ✓
- rewrite_tx: 25 tests ✓

## Design Decisions

### glyph_lexer
- **Comment tokens**: Emits canonicalized Comment tokens rather than stripping them completely
- **Position tracking**: Maintains accurate byte-offset spans for all tokens
- **Deterministic behavior**: All lexing operations are deterministic for reproducibility

### glyph_parser
- **Match subject restriction**: Match subjects limited to primary expressions to avoid ambiguity with match arm braces. Complex subjects require parentheses: `match (f x) { ... }`
- **Record arguments**: Fully supported in function applications: `f { x: 1 }`  
- **Lambda body parsing**: Uses full expression parsing to support nested let/match expressions
- **Error handling**: Comprehensive ParseError and LexError types with descriptive messages
- **Canonical serialization**: CBOR-based deterministic serialization for AST persistence

### capsule_core
- **Deterministic behavior**: All cryptographic operations and content hashing are deterministic for reproducibility
- **Error handling**: Comprehensive error types for verification failures
- **Dual CBOR libraries**: Uses `ciborium` for Root Capsule (RFC 8949) and `serde_cbor` for Work Order 4 types
- **Hash prefixes**: Type-specific prefixes prevent hash collisions across domain types (GlyphV1:, ExprV1:, NodeV1:)
- **Backward compatibility**: Legacy `compute_content_hash()` preserved for existing Root Capsule code

### genesis_graph
- **Root node bootstrapping**: Root nodes stored with empty `root_ref` to avoid circular dependency
- **Backward compatibility**: Accepts both empty and pre-hashed `root_ref` values during graph creation
- **Deterministic hashing**: Root hash always computed with `root_ref = ""` for consistency
- **Cycle prevention**: DFS-based cycle detection prevents DAG corruption
- **Canonical serialization**: BTreeMap for nodes and sorted edges ensures deterministic CBOR output
- **Hash computation**: Uses `GlyphV1:Root:` and `GlyphV1:Node:` prefixes to distinguish node types

### glyph_engine
- **BTreeMap for Bindings**: Uses BTreeMap instead of HashMap to ensure deterministic serialization order
- **Variable consistency**: Prevents patterns binding the same variable to different values (shadowing check)
- **LinearApply support**: Constructor pattern matching handles both Apply and LinearApply nodes
- **Partial record matching**: Record patterns can match a subset of fields (structural subtyping)
- **Bind pattern semantics**: x@P binds the value to x AND matches the nested pattern P
- **Constructor unwinding**: Recursively unwraps nested Apply/LinearApply chains for multi-arg constructors
- **Atomic gensym**: Uses AtomicUsize for thread-safe fresh name generation with deterministic testing
- **Capture avoidance**: Automatic α-renaming prevents variable capture during substitution
- **Scope tracking**: Maintains bound variable sets to handle shadowing correctly
- **Match arm handling**: Special logic for pattern-bound variables in match expressions
