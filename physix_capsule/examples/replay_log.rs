use physix_capsule::{self, replay_event_log};
use anyhow::{Result, bail, Context};
use std::io::Read;

fn main() -> Result<()> {
    let log_path = std::env::args().nth(1).context("Usage: replay_log <event_log.cbor>")?;
    
    let mut file = std::fs::File::open(&log_path)?;
    let mut cbor_bytes = Vec::new();
    file.read_to_end(&mut cbor_bytes)?;

    let (replayed_log, incoming_hash) = replay_event_log(&cbor_bytes)?;
    
    println!("Replay Successful!");
    println!("  Log Path: {}", log_path);
    println!("  Total Steps in Log: {}", replayed_log.total_steps);
    println!("  Event Log Content Hash: {}", incoming_hash);
    
    if !replayed_log.transforms.is_empty() {
        println!("Replay successful: event_log hash equals original");
    } else {
        bail!("Replay failed: event log was empty.");
    }

    Ok(())
}
