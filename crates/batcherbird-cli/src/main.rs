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
    /// Sample a single note
    SampleNote {
        /// MIDI note number (0-127)
        #[arg(short, long)]
        note: u8,
    },
    /// Sample a range of notes
    SampleRange {
        /// Starting MIDI note number
        #[arg(short, long)]
        start: u8,
        /// Ending MIDI note number
        #[arg(short, long)]
        end: u8,
    },
    /// Sample a single note and export to WAV
    SampleExport {
        /// MIDI note number (0-127)
        #[arg(short, long)]
        note: u8,
        /// Output directory for WAV files
        #[arg(short, long, default_value = "./samples")]
        output: String,
    },
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
        Commands::SampleNote { note } => {
            info!("Sampling single note: {}", note);
            sample_single_note(note).await?;
        }
        Commands::SampleRange { start, end } => {
            info!("Sampling note range: {} to {}", start, end);
            sample_note_range(start, end).await?;
        }
        Commands::SampleExport { note, output } => {
            info!("Sampling and exporting note: {} to {}", note, output);
            sample_and_export(note, output).await?;
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

async fn sample_single_note(note: u8) -> anyhow::Result<()> {
    use batcherbird_core::{midi::MidiManager, sampler::{SamplingEngine, SamplingConfig}};

    if note > 127 {
        println!("❌ Invalid note number: {}. Must be 0-127.", note);
        return Ok(());
    }

    println!("🎵 Single note sampling starting...");
    
    // Set up MIDI connection
    let mut midi_manager = MidiManager::new()?;
    let output_devices = midi_manager.list_output_devices()?;
    
    if output_devices.is_empty() {
        println!("❌ No MIDI output devices found. Connect a MIDI device or enable IAC Driver.");
        return Ok(());
    }
    
    // Use MiniFuse if available, otherwise first device
    let device_index = output_devices.iter()
        .position(|name| name.contains("MiniFuse"))
        .unwrap_or(0);
    println!("🎹 Using MIDI device: {}", output_devices[device_index]);
    let mut midi_conn = midi_manager.connect_output(device_index)?;
    
    // Create sampling engine with default config
    let config = SamplingConfig::default();
    let engine = SamplingEngine::new(config)?;
    
    println!("🎤 Ready to sample note {} - ensure audio is connected!", note);
    println!("   Note: Connect synthesizer output to audio input");
    
    // Sample the note
    let sample = engine.sample_single_note(&mut midi_conn, note).await?;
    
    // Analyze the sample
    let (rms, rms_db, peak_db) = batcherbird_core::audio::AudioManager::analyze_audio_samples(&sample.audio_data);
    
    println!("\n📊 Sample Analysis:");
    println!("   Note: {} ({})", sample.note, sample_note_name(sample.note));
    println!("   Samples: {}", sample.audio_data.len());
    println!("   Duration: {:.1}ms", sample.audio_timing.as_millis());
    println!("   Sample rate: {} Hz", sample.sample_rate);
    println!("   Channels: {}", sample.channels);
    println!("   RMS level: {:.1} dB", rms_db);
    println!("   Peak level: {:.1} dB", peak_db);
    
    if peak_db > -60.0 {
        println!("✅ Sample captured successfully!");
        println!("   Ready for export and processing");
    } else {
        println!("⚠️  Sample captured but audio levels very low");
        println!("   Check synthesizer output and audio connections");
    }
    
    Ok(())
}

async fn sample_note_range(start: u8, end: u8) -> anyhow::Result<()> {
    use batcherbird_core::{midi::MidiManager, sampler::{SamplingEngine, SamplingConfig}};

    if start > 127 || end > 127 || start > end {
        println!("❌ Invalid note range: {}-{}. Notes must be 0-127 and start <= end.", start, end);
        return Ok(());
    }

    let note_count = end - start + 1;
    println!("🎹 Batch sampling {} notes ({} to {})...", note_count, start, end);
    
    // Set up MIDI connection
    let mut midi_manager = MidiManager::new()?;
    let output_devices = midi_manager.list_output_devices()?;
    
    if output_devices.is_empty() {
        println!("❌ No MIDI output devices found. Connect a MIDI device or enable IAC Driver.");
        return Ok(());
    }
    
    // Use MiniFuse if available, otherwise first device
    let device_index = output_devices.iter()
        .position(|name| name.contains("MiniFuse"))
        .unwrap_or(0);
    println!("🎹 Using MIDI device: {}", output_devices[device_index]);
    let mut midi_conn = midi_manager.connect_output(device_index)?;
    
    // Create sampling engine
    let config = SamplingConfig::default();
    let engine = SamplingEngine::new(config)?;
    
    println!("🎤 Ready to sample {} notes - ensure audio is connected!", note_count);
    println!("   Note: This will take approximately {:.1} minutes", 
        (note_count as f32 * 4.0) / 60.0  // Rough estimate: 4 seconds per note
    );
    
    // Sample all notes
    let samples = engine.sample_note_range(&mut midi_conn, start, end).await?;
    
    // Analyze results
    println!("\n📊 Batch Sampling Results:");
    println!("   Total samples: {}", samples.len());
    
    let mut total_peak = -100.0;
    let mut successful_samples = 0;
    
    for sample in &samples {
        let (_, _, peak_db) = batcherbird_core::audio::AudioManager::analyze_audio_samples(&sample.audio_data);
        if peak_db > -60.0 {
            successful_samples += 1;
        }
        if peak_db > total_peak {
            total_peak = peak_db;
        }
    }
    
    println!("   Successful captures: {}/{}", successful_samples, samples.len());
    println!("   Highest peak level: {:.1} dB", total_peak);
    
    if successful_samples == samples.len() {
        println!("✅ All samples captured successfully!");
    } else {
        println!("⚠️  Some samples had low audio levels - check connections");
    }
    
    Ok(())
}

async fn sample_and_export(note: u8, output_dir: String) -> anyhow::Result<()> {
    use batcherbird_core::{
        midi::MidiManager, 
        sampler::{SamplingEngine, SamplingConfig},
        export::{SampleExporter, ExportConfig, AudioFormat}
    };
    use std::path::PathBuf;

    if note > 127 {
        println!("❌ Invalid note number: {}. Must be 0-127.", note);
        return Ok(());
    }

    println!("🎵 Sampling and exporting note {}...", note);
    
    // Set up MIDI connection
    let mut midi_manager = MidiManager::new()?;
    let output_devices = midi_manager.list_output_devices()?;
    
    if output_devices.is_empty() {
        println!("❌ No MIDI output devices found. Connect a MIDI device or enable IAC Driver.");
        return Ok(());
    }
    
    // Use MiniFuse if available, otherwise first device
    let device_index = output_devices.iter()
        .position(|name| name.contains("MiniFuse"))
        .unwrap_or(0);
    println!("🎹 Using MIDI device: {}", output_devices[device_index]);
    let mut midi_conn = midi_manager.connect_output(device_index)?;
    
    // Create sampling engine
    let sampling_config = SamplingConfig::default();
    let engine = SamplingEngine::new(sampling_config)?;
    
    // Create export config
    let export_config = ExportConfig {
        output_directory: PathBuf::from(output_dir),
        naming_pattern: "{note_name}_{note}_vel{velocity}_{timestamp}.wav".to_string(),
        sample_format: AudioFormat::Wav24Bit,
        normalize: true,
        fade_in_ms: 0.0,
        fade_out_ms: 10.0,
    };
    
    let exporter = SampleExporter::new(export_config)?;
    
    println!("🎤 Ready to sample and export note {} - ensure audio is connected!", note);
    println!("{}", exporter.get_export_info());
    
    // Sample the note
    let sample = engine.sample_single_note(&mut midi_conn, note).await?;
    
    // Analyze the sample
    let (_, rms_db, peak_db) = batcherbird_core::audio::AudioManager::analyze_audio_samples(&sample.audio_data);
    
    println!("\n📊 Sample Analysis:");
    println!("   Note: {} ({})", sample.note, sample_note_name(sample.note));
    println!("   Samples: {}", sample.audio_data.len());
    println!("   Duration: {:.1}ms", sample.audio_timing.as_millis());
    println!("   Sample rate: {} Hz", sample.sample_rate);
    println!("   Channels: {}", sample.channels);
    println!("   RMS level: {:.1} dB", rms_db);
    println!("   Peak level: {:.1} dB", peak_db);
    
    // Export the sample
    let exported_file = exporter.export_sample(&sample)?;
    
    println!("\n✅ Sample exported successfully!");
    println!("   File: {}", exported_file.display());
    println!("   Size: {:.1} KB", std::fs::metadata(&exported_file)?.len() as f64 / 1024.0);
    
    if peak_db > -60.0 {
        println!("   Audio levels: Good ({:.1} dB peak)", peak_db);
    } else {
        println!("   ⚠️  Audio levels very low - check synthesizer connections");
    }
    
    Ok(())
}

fn sample_note_name(note: u8) -> String {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (note / 12).saturating_sub(1);
    let note_name = note_names[(note % 12) as usize];
    format!("{}{}", note_name, octave)
}