use batcherbird_core::{
    midi::MidiManager, 
    audio::AudioManager,
    sampler::{SamplingEngine, SamplingConfig, AudioLevels},
    export::{SampleExporter, ExportConfig, AudioFormat},
};
use midir::MidiOutputConnection;
use std::sync::{Mutex, Arc};
use std::time::Duration;
use std::process::Command;

// Simple working pattern - don't break what works
static MIDI_MANAGER: Mutex<Option<MidiManager>> = Mutex::new(None);
static MIDI_CONNECTION: Mutex<Option<MidiOutputConnection>> = Mutex::new(None);

// Simplified monitoring state (professional approach - use existing SamplingEngine)
static MONITORING_ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static GLOBAL_SAMPLING_ENGINE: Mutex<Option<Arc<SamplingEngine>>> = Mutex::new(None);
static MONITORING_THREAD: Mutex<Option<std::thread::JoinHandle<()>>> = Mutex::new(None);


/// Start audio input monitoring (simplified professional approach)
#[tauri::command]
async fn start_input_monitoring() -> Result<String, String> {
    println!("🎛️ Starting audio input monitoring (professional approach)");
    
    // Check if already monitoring
    if MONITORING_ACTIVE.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok("Audio monitoring already active".to_string());
    }
    
    // Set monitoring flag first
    MONITORING_ACTIVE.store(true, std::sync::atomic::Ordering::Relaxed);
    
    // Create monitoring in a separate thread (avoids Send+Sync issues)
    // Note: Can't clone AtomicBool, but we can access the static directly from the thread
    
    let handle = std::thread::spawn(move || {
        println!("🧵 Monitoring thread started (using SamplingEngine)");
        
        // Create SamplingEngine in this thread
        let config = SamplingConfig {
            note_duration_ms: 0,     // Not used for monitoring
            release_time_ms: 0,      // Not used for monitoring 
            pre_delay_ms: 0,         // Not used for monitoring
            post_delay_ms: 0,        // Not used for monitoring
            midi_channel: 0,         // Not used for monitoring
            velocity: 100,           // Not used for monitoring
        };
        
        let sampling_engine = match SamplingEngine::new(config) {
            Ok(engine) => {
                println!("✅ SamplingEngine created for monitoring");
                Arc::new(engine)
            },
            Err(e) => {
                println!("❌ Failed to create SamplingEngine: {}", e);
                MONITORING_ACTIVE.store(false, std::sync::atomic::Ordering::Relaxed);
                return;
            }
        };
        
        // Store the engine globally so we can access levels from get_audio_levels
        {
            let mut engine_guard = GLOBAL_SAMPLING_ENGINE.lock().unwrap();
            *engine_guard = Some(Arc::clone(&sampling_engine));
        }
        
        // Start monitoring stream using SamplingEngine's built-in method
        let stream = match sampling_engine.start_monitoring_stream() {
            Ok(s) => s,
            Err(e) => {
                println!("❌ Failed to create monitoring stream: {}", e);
                MONITORING_ACTIVE.store(false, std::sync::atomic::Ordering::Relaxed);
                return;
            }
        };
        
        // Start the stream
        use cpal::traits::StreamTrait;
        if let Err(e) = stream.play() {
            println!("❌ Failed to start monitoring stream: {}", e);
            MONITORING_ACTIVE.store(false, std::sync::atomic::Ordering::Relaxed);
            return;
        }
        
        println!("✅ SamplingEngine monitoring stream started and playing");
        
        // Keep the stream alive while monitoring is active
        while MONITORING_ACTIVE.load(std::sync::atomic::Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        
        // Stop the stream
        if let Err(e) = stream.pause() {
            println!("⚠️ Warning: Failed to pause monitoring stream: {}", e);
        }
        
        println!("✅ SamplingEngine monitoring thread finished");
    });
    
    // Store the thread handle
    {
        let mut thread_guard = MONITORING_THREAD.lock().unwrap();
        *thread_guard = Some(handle);
    }
    
    println!("✅ Audio input monitoring started (using SamplingEngine infrastructure)");
    Ok("Audio input monitoring started".to_string())
}


/// Stop audio input monitoring
#[tauri::command]
async fn stop_input_monitoring() -> Result<String, String> {
    println!("🎛️ Stopping audio input monitoring");
    
    // Clear monitoring flag - this will cause the monitoring thread to exit
    MONITORING_ACTIVE.store(false, std::sync::atomic::Ordering::Relaxed);
    
    // Wait for the monitoring thread to finish
    {
        let mut thread_guard = MONITORING_THREAD.lock().unwrap();
        if let Some(handle) = thread_guard.take() {
            // Drop the lock before joining to avoid deadlock
            drop(thread_guard);
            
            if let Err(e) = handle.join() {
                println!("⚠️ Warning: SamplingEngine monitoring thread did not exit cleanly: {:?}", e);
            } else {
                println!("✅ SamplingEngine monitoring thread joined successfully");
            }
        }
    }
    
    // Remove the global sampling engine
    {
        let mut engine_guard = GLOBAL_SAMPLING_ENGINE.lock().unwrap();
        *engine_guard = None;
    }
    
    println!("✅ Audio input monitoring stopped");
    Ok("Audio input monitoring stopped".to_string())
}

/// Get current audio levels for UI meters (simplified professional approach)
#[tauri::command]
async fn get_audio_levels() -> Result<AudioLevels, String> {
    // Only return real levels when monitoring is active
    if !MONITORING_ACTIVE.load(std::sync::atomic::Ordering::Relaxed) {
        // Return silent levels when monitoring is off (AKAI style)
        return Ok(AudioLevels {
            peak: 0.0,
            rms: 0.0,
            peak_db: -60.0,
            rms_db: -60.0,
        });
    }
    
    // Get levels from the global sampling engine (reuse existing infrastructure)
    let engine_guard = GLOBAL_SAMPLING_ENGINE.lock().unwrap();
    if let Some(engine) = engine_guard.as_ref() {
        let levels = engine.get_audio_levels();
        Ok(levels)
    } else {
        // Engine not available, return silent levels
        Ok(AudioLevels {
            peak: 0.0,
            rms: 0.0,
            peak_db: -60.0,
            rms_db: -60.0,
        })
    }
}

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
async fn select_output_directory(app: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    use std::sync::mpsc;
    
    println!("📁 Opening native macOS directory picker...");
    
    let (tx, rx) = mpsc::channel();
    
    app.dialog()
        .file()
        .set_title("Select Sample Output Directory")
        .pick_folder(move |file_path| {
            let _ = tx.send(file_path);
        });
    
    match rx.recv() {
        Ok(Some(path)) => {
            let path_str = path.to_string();
            println!("✅ User selected directory: {}", path_str);
            Ok(path_str)
        },
        Ok(None) => {
            println!("❌ User cancelled directory selection");
            Err("Directory selection cancelled".to_string())
        },
        Err(e) => {
            println!("❌ Directory picker error: {}", e);
            Err(format!("Directory picker failed: {}", e))
        }
    }
}

/// GUI Layer: Blocking orchestration following TAURI_AUDIO_ARCHITECTURE.md
/// Uses dedicated thread + channels pattern for thread safety
#[tauri::command]  // BLOCKING command (no async) - this is correct for audio
fn record_sample(note: u8, velocity: u8, duration: u32, output_directory: Option<String>, sample_name: Option<String>) -> Result<String, String> {
    println!("🎛️ GUI: Recording sample (note: {}, velocity: {}, duration: {}ms)", note, velocity, duration);
    
    // Step 1: Get MIDI connection (GUI responsibility)
    let mut connection = {
        let mut connection_guard = MIDI_CONNECTION.lock().unwrap();
        match connection_guard.take() {
            Some(conn) => conn,
            None => return Err("No MIDI connection established. Please select a MIDI device first.".to_string()),
        }
    };
    
    // Step 2: Audio processing in dedicated thread (follows architecture pattern)
    println!("📡 GUI: Delegating to Core Audio Engine in dedicated thread...");
    
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        println!("🧵 Audio thread started");
        
        // Configure Core Audio Engine
        println!("🔧 Configuring sampling engine...");
        let sampling_config = SamplingConfig {
            note_duration_ms: duration as u64,
            release_time_ms: 500,  // Professional standard: 500ms release capture
            pre_delay_ms: 100,     // Professional standard: 100ms pre-roll  
            post_delay_ms: 100,    // Clean buffer flush
            midi_channel: 0,       // Channel 1 (0-indexed)
            velocity,
        };
        
        println!("🎛️ Creating SamplingEngine with config: {:?}", sampling_config);
        let sampling_engine = match SamplingEngine::new(sampling_config) {
            Ok(engine) => {
                println!("✅ SamplingEngine created successfully");
                engine
            },
            Err(e) => {
                println!("❌ Failed to create SamplingEngine: {}", e);
                let _ = tx.send((Err(e), connection));
                return;
            }
        };
        
        // Use blocking method from Core Audio Engine
        println!("🎵 Starting sample recording for note {}", note);
        let result = sampling_engine.sample_single_note_blocking(&mut connection, note);
        
        match &result {
            Ok(sample) => println!("✅ Recording completed: {} samples", sample.audio_data.len()),
            Err(e) => println!("❌ Recording failed: {}", e),
        }
        
        // Send result back via channel
        println!("📡 Sending result back to main thread");
        let _ = tx.send((result, connection));
    });
    
    // Step 3: Block until audio operation completes (this is correct for audio)
    let (recording_result, returned_connection) = rx.recv()
        .map_err(|e| format!("Audio thread communication failed: {}", e))?;
    
    // Put the connection back
    *MIDI_CONNECTION.lock().unwrap() = Some(returned_connection);
    
    match recording_result {
        Ok(recorded_sample) => {
            println!("✅ GUI: Core Audio Engine completed recording successfully");
            println!("📊 GUI: Received {} samples from Core Engine", recorded_sample.audio_data.len());
            
            // Step 4: Handle export (GUI orchestration)
            let output_dir = if let Some(dir) = output_directory {
                if dir.trim().is_empty() {
                    // Use Desktop/Batcherbird Samples when field is empty
                    dirs::desktop_dir()
                        .map(|desktop| desktop.join("Batcherbird Samples"))
                        .unwrap_or_else(|| std::path::PathBuf::from("samples"))
                        .to_string_lossy()
                        .to_string()
                } else {
                    dir
                }
            } else {
                // Default to Desktop/Batcherbird Samples
                dirs::desktop_dir()
                    .map(|desktop| desktop.join("Batcherbird Samples"))
                    .unwrap_or_else(|| std::path::PathBuf::from("samples"))
                    .to_string_lossy()
                    .to_string()
            };
            
            let mut output_path = std::path::PathBuf::from(&output_dir);
            
            // Create subfolder if sample name is provided (professional organization)
            if let Some(name) = sample_name.as_ref().filter(|n| !n.trim().is_empty()) {
                output_path = output_path.join(name.trim());
                println!("📁 GUI: Creating subfolder for sample: {}", name.trim());
            }
            
            // Ensure output directory exists (including subfolder)
            if let Err(e) = std::fs::create_dir_all(&output_path) {
                println!("❌ GUI: Failed to create output directory: {}", e);
                return Err(format!("Failed to create output directory '{}': {}", output_path.display(), e));
            }
            
            println!("📁 GUI: Using output directory: {}", output_path.display());
            
            // Build naming pattern with optional sample name prefix
            let naming_pattern = if let Some(name) = sample_name.as_ref().filter(|n| !n.trim().is_empty()) {
                format!("{}_{{note_name}}_{{note}}_{{velocity}}.wav", name.trim())
            } else {
                "{note_name}_{note}_{velocity}.wav".to_string()
            };
            
            let export_config = ExportConfig {
                output_directory: output_path,
                naming_pattern,
                sample_format: AudioFormat::Wav32BitFloat, // Professional 32-bit float
                normalize: false, // Preserve original dynamics from core
                fade_in_ms: 0.0,
                fade_out_ms: 10.0,
                apply_detection: true, // Enable detection by default
                detection_config: Default::default(),
            };
            
            println!("🔧 GUI: Creating sample exporter...");
            let exporter = SampleExporter::new(export_config).map_err(|e| {
                println!("❌ GUI: Failed to create exporter: {}", e);
                format!("Failed to create sample exporter: {}", e)
            })?;
            
            println!("💾 GUI: Exporting sample...");
            let file_path = exporter.export_sample(&recorded_sample).map_err(|e| {
                println!("❌ GUI: Export failed: {}", e);
                format!("Failed to export sample: {}", e)
            })?;
            
            println!("💾 GUI: Sample exported to: {}", file_path.display());
            
            // Step 5: Return success to UI
            let filename = file_path.file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            
            let success_message = format!("Recording saved: {} ({} samples)\nLocation: {}", 
                filename, recorded_sample.audio_data.len(), file_path.display());
            
            println!("✅ GUI: {}", success_message);
            Ok(success_message)
        }
        Err(e) => {
            println!("❌ GUI: Core Audio Engine reported error: {}", e);
            Err(format!("Core Audio Engine error: {}", e))
        }
    }
}

#[tauri::command]
fn record_range(start_note: u8, end_note: u8, velocity: u8, duration: u32, output_directory: Option<String>, sample_name: Option<String>) -> Result<String, String> {
    println!("🎹 GUI: Recording range sampling (notes: {}-{}, velocity: {}, duration: {}ms)", start_note, end_note, velocity, duration);
    
    // Step 1: Get MIDI connection (GUI responsibility)
    let mut connection = {
        let mut connection_guard = MIDI_CONNECTION.lock().unwrap();
        match connection_guard.take() {
            Some(conn) => conn,
            None => return Err("No MIDI connection established. Please select a MIDI device first.".to_string()),
        }
    };
    
    // Step 2: Range sampling in dedicated thread (follows architecture pattern)
    println!("📡 GUI: Delegating to Core Audio Engine for range sampling...");
    
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        println!("🧵 Range sampling thread started");
        
        // Configure Core Audio Engine
        println!("🔧 Configuring sampling engine for range...");
        let sampling_config = SamplingConfig {
            note_duration_ms: duration as u64,
            release_time_ms: 500,  // Professional standard: 500ms release capture
            pre_delay_ms: 100,     // Professional standard: 100ms pre-roll  
            post_delay_ms: 100,    // Clean buffer flush
            midi_channel: 0,       // Channel 1 (0-indexed)
            velocity,
        };
        
        println!("🎛️ Creating SamplingEngine for range sampling...");
        let sampling_engine = match SamplingEngine::new(sampling_config) {
            Ok(engine) => {
                println!("✅ SamplingEngine created successfully");
                engine
            },
            Err(e) => {
                println!("❌ Failed to create SamplingEngine: {}", e);
                let _ = tx.send((Err(e), connection));
                return;
            }
        };
        
        // Use blocking range method from Core Audio Engine
        println!("🎵 Starting range recording for notes {}-{}", start_note, end_note);
        let result = sampling_engine.sample_note_range_blocking(&mut connection, start_note, end_note);
        
        match &result {
            Ok(samples) => println!("✅ Range recording completed: {} samples", samples.len()),
            Err(e) => println!("❌ Range recording failed: {}", e),
        }
        
        // Send result back via channel
        println!("📡 Sending range result back to main thread");
        let _ = tx.send((result, connection));
    });
    
    // Step 3: Block until range operation completes
    let (recording_result, returned_connection) = rx.recv()
        .map_err(|e| format!("Range sampling thread communication failed: {}", e))?;
    
    // Put the connection back
    *MIDI_CONNECTION.lock().unwrap() = Some(returned_connection);
    
    match recording_result {
        Ok(samples) => {
            println!("✅ GUI: Core Audio Engine completed range recording successfully");
            println!("📊 GUI: Received {} samples from Core Engine", samples.len());
            
            // Step 4: Handle export for all samples
            let output_dir = if let Some(dir) = output_directory {
                if dir.trim().is_empty() {
                    dirs::desktop_dir()
                        .map(|desktop| desktop.join("Batcherbird Samples"))
                        .unwrap_or_else(|| std::path::PathBuf::from("samples"))
                        .to_string_lossy()
                        .to_string()
                } else {
                    dir
                }
            } else {
                dirs::desktop_dir()
                    .map(|desktop| desktop.join("Batcherbird Samples"))
                    .unwrap_or_else(|| std::path::PathBuf::from("samples"))
                    .to_string_lossy()
                    .to_string()
            };
            
            let mut output_path = std::path::PathBuf::from(&output_dir);
            
            // Create subfolder if sample name is provided (professional organization)
            if let Some(name) = sample_name.as_ref().filter(|n| !n.trim().is_empty()) {
                output_path = output_path.join(name.trim());
                println!("📁 GUI: Creating subfolder for range samples: {}", name.trim());
            }
            
            // Ensure output directory exists (including subfolder)
            if let Err(e) = std::fs::create_dir_all(&output_path) {
                println!("❌ GUI: Failed to create output directory: {}", e);
                return Err(format!("Failed to create output directory '{}': {}", output_path.display(), e));
            }
            
            println!("📁 GUI: Using output directory: {}", output_path.display());
            
            // Export all samples with validation and safety delays
            let mut exported_files = Vec::new();
            for (index, sample) in samples.iter().enumerate() {
                println!("💾 GUI: Exporting sample {} of {} (note {}, {} audio samples)", 
                    index + 1, samples.len(), sample.note, sample.audio_data.len());
                
                // Validate sample before export
                if sample.audio_data.is_empty() {
                    println!("⚠️ GUI: Warning - Sample {} (note {}) has no audio data, skipping", index + 1, sample.note);
                    continue;
                }
                
                // Build naming pattern with optional sample name prefix (consistent with single sample recording)
                let naming_pattern = if let Some(name) = sample_name.as_ref().filter(|n| !n.trim().is_empty()) {
                    format!("{}_{{note_name}}_{{note}}_{{velocity}}.wav", name.trim())
                } else {
                    "{note_name}_{note}_{velocity}.wav".to_string()
                };
                
                let export_config = ExportConfig {
                    output_directory: output_path.clone(),
                    naming_pattern,
                    sample_format: AudioFormat::Wav32BitFloat,
                    normalize: false,
                    fade_in_ms: 0.0,
                    fade_out_ms: 10.0,
                    apply_detection: true, // Enable detection by default
                    detection_config: Default::default(),
                };
                
                let exporter = SampleExporter::new(export_config).map_err(|e| {
                    println!("❌ GUI: Failed to create exporter for note {}: {}", sample.note, e);
                    format!("Failed to create sample exporter for note {}: {}", sample.note, e)
                })?;
                
                let file_path = exporter.export_sample(&sample).map_err(|e| {
                    println!("❌ GUI: Export failed for note {}: {}", sample.note, e);
                    format!("Failed to export sample for note {}: {}", sample.note, e)
                })?;
                
                let filename = file_path.file_name().unwrap().to_string_lossy().to_string();
                println!("✅ GUI: Successfully exported: {}", filename);
                exported_files.push(filename);
                
                // Add longer delay between file exports to ensure proper file system sync
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            
            let success_message = format!("Range recording complete! {} samples saved to:\n{}", 
                exported_files.len(), output_path.display());
            
            println!("✅ GUI: {}", success_message);
            Ok(success_message)
        }
        Err(e) => {
            println!("❌ GUI: Core Audio Engine reported range recording error: {}", e);
            Err(format!("Range recording failed: {}", e))
        }
    }
}

#[tauri::command]
async fn send_midi_panic() -> Result<String, String> {
    println!("🚨 MIDI Panic command called from UI");
    
    // Extract the connection from the mutex and drop the guard
    let mut connection = {
        let mut connection_guard = MIDI_CONNECTION.lock().unwrap();
        match connection_guard.take() {
            Some(conn) => conn,
            None => return Err("No MIDI connection established. Please select a MIDI device first.".to_string()),
        }
    };
    
    // Send panic
    let result = MidiManager::send_midi_panic(&mut connection)
        .map_err(|e| e.to_string());
    
    // Put the connection back
    *MIDI_CONNECTION.lock().unwrap() = Some(connection);
    
    match result {
        Ok(_) => Ok("MIDI Panic sent successfully - all notes stopped".to_string()),
        Err(e) => Err(format!("MIDI Panic failed: {}", e)),
    }
}


#[tauri::command]
fn show_samples_in_finder() -> Result<String, String> {
    println!("📁 Opening samples folder in Finder...");
    
    // Get the default samples directory
    let samples_dir = dirs::desktop_dir()
        .map(|desktop| desktop.join("Batcherbird Samples"))
        .unwrap_or_else(|| std::path::PathBuf::from("samples"));
    
    // Create the directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&samples_dir) {
        return Err(format!("Failed to create samples directory: {}", e));
    }
    
    // Open in Finder on macOS
    match Command::new("open")
        .arg(&samples_dir)
        .status() {
        Ok(_) => {
            println!("✅ Opened {} in Finder", samples_dir.display());
            Ok(format!("Opened samples folder: {}", samples_dir.display()))
        },
        Err(e) => {
            println!("❌ Failed to open Finder: {}", e);
            Err(format!("Failed to open Finder: {}", e))
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(tauri::generate_handler![
      list_midi_devices, 
      list_audio_input_devices,
      list_audio_output_devices,
      connect_midi_device,
      test_midi_connection,
      preview_note,
      record_sample,
      record_range,
      select_output_directory,
      show_samples_in_finder,
      send_midi_panic,
      start_input_monitoring,
      stop_input_monitoring,
      get_audio_levels
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
