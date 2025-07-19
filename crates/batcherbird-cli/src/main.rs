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
    println!("MIDI connectivity test - not implemented yet");
    println!("This will send a test note and verify MIDI I/O works");
    Ok(())
}

async fn test_audio() -> anyhow::Result<()> {
    println!("Audio recording test - not implemented yet");
    println!("This will record a few seconds of audio to verify recording works");
    Ok(())
}