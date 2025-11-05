use sonus_capsule::{synth, compute_content_hash, Expression, SAMPLE_RATE};

#[test]
fn test_deterministic_440hz_hash() {
    println!("=== Deterministic 440Hz Hash Test ===");
    
    let expr = Expression::Sine {
        freq: 440.0,
        duration: 1.0,
        amp: 0.5,
    };

    let waveform = synth(&expr).unwrap();
    assert_eq!(waveform.samples.len(), SAMPLE_RATE as usize);

    let hash2 = compute_content_hash(&waveform.samples).unwrap();
    assert_eq!(hash2, waveform.content_hash);
    
    // Verify it's a valid hex string
    assert_eq!(waveform.content_hash.len(), 64); // SHA256 = 64 hex chars
    assert!(waveform.content_hash.chars().all(|c| c.is_ascii_hexdigit()));
    
    println!("✓ Deterministic hash test passed");
    println!("  Hash: {}", waveform.content_hash);
    println!("  Samples: {}", waveform.samples.len());
}

#[test]
fn test_deterministic_synthesis_multiple_runs() {
    println!("\n=== Multiple Run Determinism Test ===");
    
    let expr = Expression::Sine {
        freq: 440.0,
        duration: 0.5,
        amp: 0.8,
    };

    let wf1 = synth(&expr).unwrap();
    let wf2 = synth(&expr).unwrap();
    let wf3 = synth(&expr).unwrap();

    // All runs produce identical hashes
    assert_eq!(wf1.content_hash, wf2.content_hash);
    assert_eq!(wf2.content_hash, wf3.content_hash);
    
    // All runs produce identical samples
    assert_eq!(wf1.samples, wf2.samples);
    assert_eq!(wf2.samples, wf3.samples);
    
    println!("✓ Multiple runs produce identical results");
    println!("  Hash (run 1): {}", wf1.content_hash);
    println!("  Hash (run 2): {}", wf2.content_hash);
    println!("  Hash (run 3): {}", wf3.content_hash);
}

#[test]
fn test_waveform_metadata() {
    println!("\n=== Waveform Metadata Test ===");
    
    let freq = 880.0;
    let duration = 2.0;
    let amp = 0.3;
    
    let expr = Expression::Sine {
        freq,
        duration,
        amp,
    };

    let wf = synth(&expr).unwrap();
    
    assert_eq!(wf.metadata.length_samples, (duration * SAMPLE_RATE as f32).round() as usize);
    assert_eq!(wf.metadata.duration_sec, duration);
    assert_eq!(wf.sample_rate, SAMPLE_RATE);
    
    match wf.metadata.expr {
        Expression::Sine { freq: f, duration: d, amp: a } => {
            assert_eq!(f, freq);
            assert_eq!(d, duration);
            assert_eq!(a, amp);
        }
    }
    
    println!("✓ Metadata validation passed");
    println!("  Length: {} samples", wf.metadata.length_samples);
    println!("  Duration: {} sec", wf.metadata.duration_sec);
}
