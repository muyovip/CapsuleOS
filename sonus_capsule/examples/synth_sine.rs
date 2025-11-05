use sonus_capsule::{synth, write_waveform_cbor, Expression};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    let freq = if args.len() > 1 {
        args[1].parse::<f32>().unwrap_or(440.0)
    } else {
        440.0
    };
    
    let duration = if args.len() > 2 {
        args[2].parse::<f32>().unwrap_or(1.0)
    } else {
        1.0
    };
    
    let amp = if args.len() > 3 {
        args[3].parse::<f32>().unwrap_or(0.5)
    } else {
        0.5
    };
    
    let out_path = if args.len() > 4 {
        PathBuf::from(&args[4])
    } else {
        PathBuf::from("sine_wave.cbor")
    };

    println!("Synthesizing sine wave:");
    println!("  Frequency: {} Hz", freq);
    println!("  Duration: {} sec", duration);
    println!("  Amplitude: {}", amp);

    let expr = Expression::Sine {
        freq,
        duration,
        amp,
    };
    
    let waveform = synth(&expr).expect("synthesis failed");
    
    write_waveform_cbor(&out_path, &waveform).expect("write failed");
    
    println!("\n✅ Synthesized {}Hz for {}s -> {}", freq, duration, out_path.display());
    println!("Content Hash: {}", waveform.content_hash);
    println!("Sample Count: {}", waveform.samples.len());
    println!("Sample Rate: {} Hz", waveform.sample_rate);
    
    Ok(())
}
