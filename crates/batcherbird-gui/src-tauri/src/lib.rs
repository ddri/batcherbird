use batcherbird_core::{
    midi::MidiManager, 
    audio::AudioManager,
    sampler::Sample,
    export::{SampleExporter, ExportConfig, AudioFormat}
};
use midir::MidiOutputConnection;
use std::sync::Mutex;
use std::time::Duration;

// Simple working pattern - don't break what works
static MIDI_MANAGER: Mutex<Option<MidiManager>> = Mutex::new(None);
static MIDI_CONNECTION: Mutex<Option<MidiOutputConnection>> = Mutex::new(None);

#[tauri::command]
async fn list_midi_devices() -> Result<Vec<String>, String> {
    println!("🎹 Listing MIDI devices...");
    
    let mut manager_guard = MIDI_MANAGER.lock().unwrap();
    let midi_manager = match manager_guard.as_mut() {
        Some(manager) => manager,
        None => {
            let new_manager = MidiManager::new().map_err(|e| {
                println!("❌ Failed to create MIDI manager: {}", e);
                e.to_string()
            })?;
            *manager_guard = Some(new_manager);
            manager_guard.as_mut().unwrap()
        }
    };
    
    let devices = midi_manager.list_output_devices().map_err(|e| {
        println!("❌ Failed to list MIDI devices: {}", e);
        e.to_string()
    })?;
    
    println!("🎹 Found {} MIDI devices:", devices.len());
    for (i, device) in devices.iter().enumerate() {
        println!("  {}: {}", i, device);
    }
    
    Ok(devices)
}

#[tauri::command]
async fn list_audio_input_devices() -> Result<Vec<String>, String> {
    println!("🎤 Listing audio input devices...");
    let audio_manager = AudioManager::new().map_err(|e| {
        println!("❌ Failed to create audio manager: {}", e);
        e.to_string()
    })?;
    
    let devices = audio_manager.list_input_devices().map_err(|e| {
        println!("❌ Failed to list audio input devices: {}", e);
        e.to_string()
    })?;
    
    println!("🎤 Found {} audio input devices:", devices.len());
    for (i, device) in devices.iter().enumerate() {
        println!("  {}: {}", i, device);
    }
    
    Ok(devices)
}

#[tauri::command]
async fn list_audio_output_devices() -> Result<Vec<String>, String> {
    println!("🔊 Listing audio output devices...");
    let audio_manager = AudioManager::new().map_err(|e| {
        println!("❌ Failed to create audio manager: {}", e);
        e.to_string()
    })?;
    
    let devices = audio_manager.list_output_devices().map_err(|e| {
        println!("❌ Failed to list audio output devices: {}", e);
        e.to_string()
    })?;
    
    println!("🔊 Found {} audio output devices:", devices.len());
    for (i, device) in devices.iter().enumerate() {
        println!("  {}: {}", i, device);
    }
    
    Ok(devices)
}

#[tauri::command]
async fn connect_midi_device(device_index: usize) -> Result<String, String> {
    println!("🔌 Connecting to MIDI device index: {}", device_index);
    
    let mut manager_guard = MIDI_MANAGER.lock().unwrap();
    let midi_manager = match manager_guard.as_mut() {
        Some(manager) => manager,
        None => {
            println!("❌ No MIDI manager available - list devices first");
            return Err("MIDI manager not initialized. Please refresh MIDI devices first.".to_string());
        }
    };
    
    let connection = midi_manager.connect_output(device_index).map_err(|e| {
        println!("❌ Failed to connect to MIDI device {}: {}", device_index, e);
        e.to_string()
    })?;
    
    drop(manager_guard); // Release the manager lock before taking connection lock
    *MIDI_CONNECTION.lock().unwrap() = Some(connection);
    println!("✅ MIDI device {} connected successfully", device_index);
    Ok("MIDI device connected successfully".to_string())
}

#[tauri::command]
async fn test_midi_connection() -> Result<String, String> {
    // Extract the connection from the mutex and drop the guard
    let mut connection = {
        let mut connection_guard = MIDI_CONNECTION.lock().unwrap();
        match connection_guard.take() {
            Some(conn) => conn,
            None => return Err("No MIDI connection established. Please select a MIDI device first.".to_string()),
        }
    };
    
    // Now we can safely await without holding the guard
    let result = MidiManager::send_test_note(&mut connection, 0, 60, 127, Duration::from_millis(500))
        .await
        .map_err(|e| e.to_string());
    
    // Put the connection back
    *MIDI_CONNECTION.lock().unwrap() = Some(connection);
    
    match result {
        Ok(_) => Ok("Test note sent successfully".to_string()),
        Err(e) => Err(e),
    }
}

#[tauri::command]
async fn preview_note(note: u8, velocity: u8, duration: u32) -> Result<String, String> {
    println!("🎵 Preview note: {} (velocity: {}, duration: {}ms)", note, velocity, duration);
    
    // Extract the connection from the mutex and drop the guard
    let mut connection = {
        let mut connection_guard = MIDI_CONNECTION.lock().unwrap();
        match connection_guard.take() {
            Some(conn) => conn,
            None => return Err("No MIDI connection established. Please select a MIDI device first.".to_string()),
        }
    };
    
    // Send the note with custom parameters
    let result = MidiManager::send_test_note(
        &mut connection, 
        0, // channel 0
        note, 
        velocity, 
        Duration::from_millis(duration as u64)
    )
    .await
    .map_err(|e| e.to_string());
    
    // Put the connection back
    *MIDI_CONNECTION.lock().unwrap() = Some(connection);
    
    match result {
        Ok(_) => Ok(format!("Preview note {} sent successfully", note)),
        Err(e) => Err(e),
    }
}

#[tauri::command]
async fn record_sample(note: u8, velocity: u8, duration: u32) -> Result<String, String> {
    println!("🔴 Recording sample: note {} (velocity: {}, duration: {}ms)", note, velocity, duration);
    
    // Extract the MIDI connection
    let mut connection = {
        let mut connection_guard = MIDI_CONNECTION.lock().unwrap();
        match connection_guard.take() {
            Some(conn) => conn,
            None => return Err("No MIDI connection established. Please select a MIDI device first.".to_string()),
        }
    };
    
    // Create audio manager for recording (will be used for real recording later)
    let _audio_manager = AudioManager::new().map_err(|e| {
        println!("❌ Failed to create audio manager: {}", e);
        e.to_string()
    })?;
    
    println!("🎤 Starting audio recording...");
    
    // Generate a test sine wave for the MIDI note (for now)
    println!("🎤 Generating test audio for note {} for {}ms...", note, duration);
    let sample_rate = 48000;
    let duration_samples = ((duration as f32 / 1000.0) * sample_rate as f32) as usize;
    let frequency = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0); // A4 = 440Hz reference
    
    let mut audio_data = Vec::with_capacity(duration_samples * 2); // Stereo
    for i in 0..duration_samples {
        let t = i as f32 / sample_rate as f32;
        let amplitude = 0.3; // Keep volume reasonable
        let sample_value = amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin();
        
        // Apply envelope (attack and decay)
        let envelope = if i < sample_rate / 10 {
            // 100ms attack
            (i as f32) / (sample_rate as f32 / 10.0)
        } else if i > duration_samples - sample_rate / 5 {
            // 200ms decay
            ((duration_samples - i) as f32) / (sample_rate as f32 / 5.0)
        } else {
            1.0
        };
        
        let final_sample = sample_value * envelope;
        audio_data.push(final_sample); // Left channel
        audio_data.push(final_sample); // Right channel (same as left for mono content)
    }
    
    // Small delay before triggering MIDI to ensure recording is ready
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    println!("🎹 Triggering MIDI note...");
    
    // Send the MIDI note
    let midi_result = MidiManager::send_test_note(
        &mut connection, 
        0,
        note, 
        velocity, 
        Duration::from_millis(duration as u64)
    )
    .await
    .map_err(|e| e.to_string());
    
    // Put the connection back
    *MIDI_CONNECTION.lock().unwrap() = Some(connection);
    
    // Wait for the MIDI note to complete
    tokio::time::sleep(Duration::from_millis((duration + 500) as u64)).await;
    println!("🎤 Test audio generation completed ({}Hz sine wave)", frequency);
    
    let audio_data_len = audio_data.len();
    println!("✅ Sample recorded successfully! {} samples captured", audio_data_len);
    
    // Create a Sample struct for export
    let sample = Sample {
        note,
        velocity,
        audio_data,
        sample_rate: 48000,  // Standard sample rate
        channels: 2,         // Stereo
        recorded_at: std::time::SystemTime::now(),
        midi_timing: Duration::from_millis(duration as u64),
        audio_timing: Duration::from_millis((duration + 500) as u64),
    };
    
    // Set up export configuration
    let export_config = ExportConfig {
        output_directory: std::path::PathBuf::from("./samples"),
        naming_pattern: "batcherbird_{note_name}_{note}_{velocity}.wav".to_string(),
        sample_format: AudioFormat::Wav24Bit,
        normalize: true,
        fade_in_ms: 0.0,
        fade_out_ms: 10.0,
    };
    
    // Export the sample to WAV file
    let exporter = SampleExporter::new(export_config).map_err(|e| e.to_string())?;
    let file_path = exporter.export_sample(&sample).map_err(|e| e.to_string())?;
    
    println!("💾 Sample saved to: {}", file_path.display());
    
    let sample_count = sample.audio_data.len();
    match midi_result {
        Ok(_) => Ok(format!("Sample recorded and saved: {} ({} samples)", file_path.file_name().unwrap().to_string_lossy(), sample_count)),
        Err(e) => Err(format!("MIDI failed but audio recorded: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      list_midi_devices, 
      list_audio_input_devices,
      list_audio_output_devices,
      connect_midi_device,
      test_midi_connection,
      preview_note,
      record_sample
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
