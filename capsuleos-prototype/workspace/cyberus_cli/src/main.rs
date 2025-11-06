//! CyberusCLI - Command interface for capsule operations

use clap::{Parser, Subcommand};
use sonus_capsule::{Expression, synth, write_waveform_cbor};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cyberus_cli")]
#[command(about = "CapsuleOS command-line interface", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Synthesize audio and save as capsule
    Audio {
        #[arg(long, default_value = "440.0")]
        freq: f32,
        
        #[arg(long, default_value = "1.0")]
        duration: f32,
        
        #[arg(long, default_value = "0.5")]
        amp: f32,
        
        #[arg(long, default_value = "audio_output.cbor")]
        output: PathBuf,
    },
    
    /// Display capsule information
    Info {
        #[arg(value_name = "CAPSULE")]
        capsule: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Audio { freq, duration, amp, output } => {
            println!("Synthesizing audio capsule:");
            println!("  Frequency: {} Hz", freq);
            println!("  Duration:  {} sec", duration);
            println!("  Amplitude: {}", amp);
            
            let expr = Expression::Sine {
                freq: *freq,
                duration: *duration,
                amp: *amp,
            };
            
            let waveform = synth(&expr)?;
            let hash = waveform.compute_content_hash();
            write_waveform_cbor(output, &waveform)?;
            
            println!("\n✓ Audio capsule created:");
            println!("  Output:       {}", output.display());
            println!("  Samples:      {}", waveform.samples.len());
            println!("  Content hash: {}", hash);
        }
        
        Commands::Info { capsule } => {
            println!("Capsule info: {}", capsule.display());
            // Placeholder for future implementation
            println!("(Info command not yet implemented)");
        }
    }
    
    Ok(())
}
