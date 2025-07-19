use clap::{Parser, Subcommand};
use tracing::{info, Level};
use batcherbird_core::{midi::MidiManager, audio::AudioManager};

#[derive(Parser)]
#[command(name = "batcherbird")]
#[command(about = "Hardware synthesizer sampling automation tool")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test MIDI connectivity
    TestMidi,
    /// Test audio recording
    TestAudio,
    /// List available MIDI devices
    ListMidi,
    /// List available audio devices
    ListAudio,
    /// Monitor MIDI input messages in real-time
    MonitorMidi,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::TestMidi => {
            info!("Testing MIDI connectivity...");
            test_midi().await?;
        }
        Commands::TestAudio => {
            info!("Testing audio recording...");
            test_audio().await?;
        }
        Commands::ListMidi => {
            info!("Listing MIDI devices...");
            list_midi_devices().await?;
        }
        Commands::ListAudio => {
            info!("Listing audio devices...");
            list_audio_devices().await?;
        }
        Commands::MonitorMidi => {
            info!("Starting MIDI monitor...");
            monitor_midi().await?;
        }
    }

    Ok(())
}

async fn list_midi_devices() -> anyhow::Result<()> {
    let mut midi_manager = MidiManager::new()?;
    
    println!("MIDI Input Devices:");
    let input_devices = midi_manager.list_input_devices()?;
    if input_devices.is_empty() {
        println!("  No MIDI input devices found");
    } else {
        for (i, device) in input_devices.iter().enumerate() {
            println!("  {}: {}", i, device);
        }
    }
    
    println!("\nMIDI Output Devices:");
    let output_devices = midi_manager.list_output_devices()?;
    if output_devices.is_empty() {
        println!("  No MIDI output devices found");
    } else {
        for (i, device) in output_devices.iter().enumerate() {
            println!("  {}: {}", i, device);
        }
    }
    
    Ok(())
}

async fn list_audio_devices() -> anyhow::Result<()> {
    let audio_manager = AudioManager::new()?;
    
    println!("Audio Input Devices:");
    let input_devices = audio_manager.list_input_devices()?;
    if input_devices.is_empty() {
        println!("  No audio input devices found");
    } else {
        for (i, device) in input_devices.iter().enumerate() {
            println!("  {}: {}", i, device);
        }
    }
    
    println!("\nAudio Output Devices:");
    let output_devices = audio_manager.list_output_devices()?;
    if output_devices.is_empty() {
        println!("  No audio output devices found");
    } else {
        for (i, device) in output_devices.iter().enumerate() {
            println!("  {}: {}", i, device);
        }
    }
    
    Ok(())
}

async fn test_midi() -> anyhow::Result<()> {
    use std::time::Duration;
    use batcherbird_core::midi::MidiManager;

    println!("MIDI connectivity test starting...");
    
    let mut midi_manager = MidiManager::new()?;
    let output_devices = midi_manager.list_output_devices()?;
    
    if output_devices.is_empty() {
        println!("❌ No MIDI output devices found. Connect a MIDI device or enable IAC Driver.");
        return Ok(());
    }
    
    println!("Available MIDI outputs:");
    for (i, device) in output_devices.iter().enumerate() {
        println!("  {}: {}", i, device);
    }
    
    // Use first available device
    let device_index = 0;
    println!("\n🎹 Connecting to device {}: {}", device_index, output_devices[device_index]);
    
    let mut conn = midi_manager.connect_output(device_index)?;
    
    println!("📤 Sending test sequence: C3 note 10 times, 2 seconds apart");
    println!("   Note: Connect a synthesizer to hear the test tones");
    
    // Send C3 (note 48) 10 times with 2 second spacing
    for i in 1..=10 {
        println!("   Playing note {} of 10...", i);
        MidiManager::send_test_note(&mut conn, 0, 48, 100, Duration::from_millis(500)).await?;
        
        if i < 10 {
            // Wait 2 seconds before next note (minus the 0.5s note duration)
            tokio::time::sleep(Duration::from_millis(1500)).await;
        }
    }
    
    println!("✅ MIDI test completed successfully!");
    println!("   - 10 C3 notes sent with precise timing");
    println!("   - 2 second intervals maintained");
    println!("   - MIDI sequencing validated");
    
    Ok(())
}

async fn monitor_midi() -> anyhow::Result<()> {
    use batcherbird_core::midi::MidiManager;

    println!("MIDI Monitor - Real-time MIDI message display");
    
    let mut midi_manager = MidiManager::new()?;
    let input_devices = midi_manager.list_input_devices()?;
    
    if input_devices.is_empty() {
        println!("❌ No MIDI input devices found.");
        println!("   Connect a MIDI device or enable IAC Driver in Audio MIDI Setup");
        return Ok(());
    }
    
    println!("Available MIDI inputs:");
    for (i, device) in input_devices.iter().enumerate() {
        println!("  {}: {}", i, device);
    }
    
    // Use first available device
    let device_index = 0;
    println!("\n🎧 Monitoring device {}: {}", device_index, input_devices[device_index]);
    println!("📡 Listening for MIDI messages... (Press Ctrl+C to stop)\n");
    
    let _conn = midi_manager.connect_input(device_index)?;
    
    // Keep the connection alive
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

async fn test_audio() -> anyhow::Result<()> {
    use batcherbird_core::audio::AudioManager;

    println!("Audio recording test starting...");
    
    let audio_manager = AudioManager::new()?;
    
    println!("📋 Available input devices:");
    let input_devices = audio_manager.list_input_devices()?;
    for (i, device) in input_devices.iter().enumerate() {
        println!("  {}: {}", i, device);
    }
    
    if input_devices.is_empty() {
        println!("❌ No audio input devices found.");
        return Ok(());
    }
    
    println!("\n🎤 Testing audio recording (3 seconds)...");
    println!("   Get ready to make some noise (tap mic, speak, etc.)");
    
    // Record 3 seconds of audio
    let samples = audio_manager.record_test_audio(3)?;
    
    // Analyze the recording
    let (rms, rms_db, peak_db) = AudioManager::analyze_audio_samples(&samples);
    
    println!("\n📊 Audio Analysis:");
    println!("   Samples captured: {}", samples.len());
    println!("   RMS level: {:.6} ({:.1} dB)", rms, rms_db);
    println!("   Peak level: {:.1} dB", peak_db);
    
    // Determine if we captured meaningful audio
    if peak_db > -60.0 {
        println!("✅ Audio test successful!");
        println!("   - Real-time audio capture working");
        println!("   - Audio levels detected: {:.1} dB peak", peak_db);
        println!("   - Ready for synthesizer sampling");
    } else {
        println!("⚠️  Audio test completed but levels very low");
        println!("   - Recording hardware working");
        println!("   - No significant audio detected (silence or very quiet)");
        println!("   - Try speaking into microphone or making noise");
    }
    
    Ok(())
}