//! Capsule test runner - verification and replay harness

use std::path::PathBuf;
use std::fs;
use clap::Parser;
use sha2::{Sha256, Digest};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "/var/gge/graph")]
    graph: PathBuf,
}

fn compute_hash(prefix: &str, bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(prefix.as_bytes());
    h.update(bytes);
    hex::encode(h.finalize())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    println!("════════════════════════════════════════════════════════");
    println!("   CAPSULE TEST RUNNER - Content Verification");
    println!("════════════════════════════════════════════════════════\n");
    
    println!("Checking graph files in: {}\n", args.graph.display());
    
    if !args.graph.exists() {
        println!("✗ Graph directory does not exist!");
        return Ok(());
    }
    
    let mut verified = 0;
    let mut total = 0;
    
    if let Ok(entries) = fs::read_dir(&args.graph) {
        for ent in entries.flatten() {
            let path = ent.path();
            if path.is_file() {
                total += 1;
                let bytes = fs::read(&path)?;
                
                // naive: guess prefix by filename
                let fname = path.file_name().unwrap().to_string_lossy();
                let prefix = if fname.starts_with("render_") { "RenderV1" }
                             else if fname.starts_with("phys_") { "NodeV1" }
                             else if fname.starts_with("audio_") { "AudioV1" }
                             else { "NodeV1" };
                
                let computed_hash = compute_hash(prefix, &bytes);
                
                // Extract expected hash from filename
                let parts: Vec<&str> = fname.rsplitn(2, '_').collect();
                if parts.len() == 2 {
                    let expected_hash = parts[0].trim_end_matches(".cbor");
                    
                    if computed_hash == expected_hash {
                        println!("✓ {} [{}]", fname, prefix);
                        println!("  Hash: {}\n", computed_hash);
                        verified += 1;
                    } else {
                        println!("✗ {} [MISMATCH]", fname);
                        println!("  Expected: {}", expected_hash);
                        println!("  Computed: {}\n", computed_hash);
                    }
                } else {
                    println!("? {} -> {} [{}]", fname, computed_hash, prefix);
                }
            }
        }
    }
    
    println!("════════════════════════════════════════════════════════");
    println!("Verification: {}/{} files verified", verified, total);
    
    if verified == total && total > 0 {
        println!("✓ All content hashes verified and matching!");
    } else if verified < total {
        println!("✗ Some content hashes did not match");
    }
    
    println!("════════════════════════════════════════════════════════\n");
    
    Ok(())
}
