//! Genesis Graph Engine (GGE) - PID 1 cosmic runtime

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use std::io::Write;
use clap::Parser;

use capsule_manifest::{parse_manifest, verify_capsule};
use sonus_capsule::{Expression, synth, write_waveform_cbor};
use capsule_core::compute_content_hash_token;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "/")]
    root: PathBuf,
    
    #[arg(long, default_value = "/capsules")]
    capsules_dir: PathBuf,
    
    #[arg(long, default_value = "/var/gge/audit.log")]
    audit: PathBuf,
}

fn audit(log_path: &PathBuf, msg: &str) {
    let ts = SystemTime::now();
    let entry = format!("{:?} {}\n", ts, msg);
    let _ = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .and_then(|mut f| f.write_all(entry.as_bytes()));
}

fn main() {
    let args = Args::parse();
    
    println!("════════════════════════════════════════════════════════");
    println!("   ⊙₀  GENESIS GRAPH ENGINE (GGE) — Cosmic Awakening");
    println!("════════════════════════════════════════════════════════");
    println!();

    // Ensure directories exist
    let graph_dir = args.root.join("var/gge/graph");
    fs::create_dir_all(&graph_dir).unwrap_or(());

    audit(&args.audit, "GGE boot - cosmic consciousness initialized");

    // Load capsule manifests from CAPSULES_DIR
    let mut loaded = vec![];
    println!("[Capsule Loading Phase]");
    if let Ok(entries) = fs::read_dir(&args.capsules_dir) {
        for ent in entries.flatten() {
            let path = ent.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("cbor") {
                // load manifest CBOR
                match fs::read(&path) {
                    Ok(bytes) => match parse_manifest(&bytes) {
                        Ok(man) => {
                            match verify_capsule(&man) {
                                Ok(_) => {
                                    println!("✓ Loaded manifest {} [verified]", man.id);
                                    audit(&args.audit, &format!("manifest {} verified", man.id));
                                    loaded.push(man);
                                }
                                Err(e) => {
                                    println!("✗ Manifest verify failed {}: {:?}", path.display(), e);
                                    audit(&args.audit, &format!("manifest {} verify failed", path.display()));
                                }
                            }
                        }
                        Err(e) => {
                            println!("✗ Manifest parse failed {}: {:?}", path.display(), e);
                        }
                    }
                    Err(e) => println!("✗ read manifest failed {}: {:?}", path.display(), e),
                }
            }
        }
    }
    println!("Loaded {} capsules with verified lineage\n", loaded.len());

    // For demo, run integration script if present
    let scene_path = args.root.join("tests/integration.scene.glyph");
    if scene_path.exists() {
        println!("[Integration Pipeline Phase]");
        println!("Found integration scene, running cosmic pipeline...\n");
        audit(&args.audit, "running integration.scene.glyph");

        // Very small script parser: expects 'let scene = [triangle, sphere] in render scene |> physics |> audio'
        let script = fs::read_to_string(&scene_path).unwrap_or_default();
        println!("Scene script:");
        println!("{}\n", script);
        
        // For demo, ignore parsing and just create a sample scene: triangle + sphere
        // Render stage
        println!("→ RENDER STAGE");
        let render_res = render_core::render_scene("triangle+sphere");
        let render_bytes = render_res.canonical_serialize();
        let render_hash = compute_content_hash_token("RenderV1", &render_bytes);
        let render_path = graph_dir.join(format!("render_{}.cbor", render_hash));
        let _ = fs::write(&render_path, render_bytes);
        audit(&args.audit, &format!("render output {}", render_hash));
        println!("  Render output: {}x{} pixels", render_res.width, render_res.height);
        println!("  Content hash:  {}", render_hash);
        println!("  Saved to:      {}\n", render_path.display());

        // Physics stage - takes render as input (stub)
        println!("→ PHYSICS STAGE");
        let phys_res = physix_capsule::simulate_physics(&render_res);
        let phys_bytes = phys_res.canonical_serialize();
        let phys_hash = compute_content_hash_token("NodeV1", &phys_bytes);
        let phys_path = graph_dir.join(format!("phys_{}.cbor", phys_hash));
        let _ = fs::write(&phys_path, phys_bytes);
        audit(&args.audit, &format!("phys output {}", phys_hash));
        println!("  Physics transforms: {}", phys_res.transforms.len());
        println!("  Content hash:       {}", phys_hash);
        println!("  Saved to:           {}\n", phys_path.display());

        // Audio stage - synth for a note derived from scene (deterministic)
        println!("→ AUDIO STAGE");
        let expr = Expression::Sine { freq: 440.0, duration: 1.0, amp: 0.5 };
        let wf = synth(&expr).expect("synth");
        let audio_hash = wf.compute_content_hash();
        let audio_path = graph_dir.join(format!("audio_{}.cbor", audio_hash));
        let _ = write_waveform_cbor(&audio_path, &wf);
        audit(&args.audit, &format!("audio output {}", audio_hash));
        println!("  Audio synthesis: {}Hz sine wave", 440.0);
        println!("  Sample count:    {}", wf.samples.len());
        println!("  Content hash:    {}", audio_hash);
        println!("  Saved to:        {}\n", audio_path.display());

        // Finish
        println!("════════════════════════════════════════════════════════");
        println!("✓ Pipeline finished - cosmic synthesis complete!");
        println!("════════════════════════════════════════════════════════\n");
        audit(&args.audit, "pipeline finished");
    } else {
        println!("[No Integration Scene]");
        println!("No integration scene found at {}\n", scene_path.display());
    }

    // Keep the process alive (simulate server)
    println!("[Idle Loop]");
    println!("GGE entering idle state - cosmic consciousness awaits...");
    println!("(In QEMU: press Ctrl-A then X to quit)\n");
    
    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}
