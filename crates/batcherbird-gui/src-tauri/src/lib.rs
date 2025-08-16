use batcherbird_core::{
    midi::MidiManager, 
    audio::AudioManager,
    sampler::{SamplingEngine, SamplingConfig, AudioLevels, VizChunk, Sample},
    export::{SampleExporter, ExportConfig, AudioFormat},
    loop_detection::LoopDetectionConfig,
    playback::AudioPlayback,
    session::{SessionConfig, ValidationReport},
    session_manager::{SessionManager, DeviceTestResult},
    ProfessionalMeterEngine, ProfessionalMeterReadings, GainStagingAssistant, GainStagingAnalysis,
    IntelligentSampleDetector, IntelligentDetectionConfig, IntelligentDetectionResult,
    ProfessionalTrimmer,
    BatcherbirdError,
};
use midir::MidiOutputConnection;
use std::sync::{Mutex, Arc};
use std::time::Duration;
use std::process::Command;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use tauri::Emitter;

mod session;
use session::{RecordingSession, SessionInfo};

/// Convert MIDI note number to note name
fn note_to_name(note: u8) -> String {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (note / 12) as i8 - 2;
    let note_index = (note % 12) as usize;
    format!("{}{}", note_names[note_index], octave)
}

/// Validates and secures file paths for desktop app usage
fn validate_file_path(path: &str) -> Result<PathBuf, String> {
    let path_buf = Path::new(path);
    
    // Basic validation - reject obviously malicious patterns
    if path.contains("..") || path.starts_with('/') && !path.starts_with("/Users/") {
        return Err("Invalid path pattern".to_string());
    }
    
    // For macOS desktop app, allow reasonable user directories
    let allowed_prefixes = [
        "/Users/",
        "/tmp/",
        "/var/folders/", // macOS temp files
    ];
    
    let absolute_path = if path_buf.is_absolute() {
        path_buf.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|_| "Cannot get current directory")?
            .join(path_buf)
    };
    
    // Allow common user directories for a desktop app
    let path_str = absolute_path.to_string_lossy();
    if allowed_prefixes.iter().any(|prefix| path_str.starts_with(prefix)) {
        Ok(absolute_path)
    } else {
        Err("Path outside allowed directories".to_string())
    }
}

/// Serializable version of LoopCandidate for JSON responses
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoopCandidateResponse {
    pub start_sample: usize,
    pub end_sample: usize,
    pub length_samples: usize,
    pub quality_score: f32,
    pub zero_crossing_aligned: bool,
    pub correlation: f32,
}

/// Serializable response structure for loop detection results
#[derive(Serialize, Deserialize, Debug)]
pub struct LoopDetectionResponse {
    pub success: bool,
    pub sample_rate: u32,
    pub candidates: Vec<LoopCandidateResponse>,
    pub best_candidate: Option<LoopCandidateResponse>,
    pub failure_reason: Option<String>,
}

/// Waveform data for visualization
#[derive(Serialize, Deserialize, Debug)]
pub struct WaveformData {
    pub peaks: WaveformPeaks,
    pub sample_rate: u32,
    pub duration: f64,
    pub channels: u8,
    pub format: String,
}

/// Peak data for waveform rendering
#[derive(Serialize, Deserialize, Debug)]
pub struct WaveformPeaks {
    pub positive: Vec<f32>,
    pub negative: Vec<f32>,
}

/// Audio device channel information
#[derive(Serialize, Deserialize, Debug)]
pub struct AudioDeviceInfo {
    pub device_name: String,
    pub total_channels: u16,
    pub sample_rate: u32,
    pub channel_names: Vec<String>,
}

// Simple working pattern - don't break what works
static MIDI_MANAGER: Mutex<Option<MidiManager>> = Mutex::new(None);
static MIDI_CONNECTION: Mutex<Option<MidiOutputConnection>> = Mutex::new(None);

// Simplified monitoring state (professional approach - use existing SamplingEngine)
static MONITORING_ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static GLOBAL_SAMPLING_ENGINE: Mutex<Option<Arc<SamplingEngine>>> = Mutex::new(None);
static MONITORING_THREAD: Mutex<Option<std::thread::JoinHandle<()>>> = Mutex::new(None);

// Recording cancellation flag for Epic 4
static RECORDING_CANCELLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

// Audio playback state (following existing patterns)
static AUDIO_PLAYBACK: Mutex<Option<Arc<AudioPlayback>>> = Mutex::new(None);

// Professional session management (new architecture)
static SESSION_MANAGER: Mutex<Option<SessionManager>> = Mutex::new(None);

// Professional meter engine (Epic 3.1.3)
static PROFESSIONAL_METER_ENGINE: Mutex<Option<ProfessionalMeterEngine>> = Mutex::new(None);
static GAIN_STAGING_ASSISTANT: Mutex<Option<GainStagingAssistant>> = Mutex::new(None);


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
            ..SamplingConfig::default() // Use defaults for input_mode and input_channels
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

#[tauri::command]
async fn start_input_monitoring_with_playthrough(enable_playthrough: bool) -> Result<String, String> {
    println!("🎛️ Starting audio input monitoring with playthrough: {}", enable_playthrough);
    
    // Check if already monitoring
    if MONITORING_ACTIVE.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok("Audio monitoring already active".to_string());
    }
    
    // Set monitoring flag first
    MONITORING_ACTIVE.store(true, std::sync::atomic::Ordering::Relaxed);
    
    // Create monitoring with playthrough in a separate thread
    let handle = std::thread::spawn(move || {
        println!("🧵 Monitoring thread started (with playthrough: {})", enable_playthrough);
        
        // Create SamplingEngine in this thread
        let config = SamplingConfig {
            note_duration_ms: 0,     // Not used for monitoring
            release_time_ms: 0,      // Not used for monitoring 
            pre_delay_ms: 0,         // Not used for monitoring
            post_delay_ms: 0,        // Not used for monitoring
            midi_channel: 0,         // Not used for monitoring
            velocity: 127,           // Not used for monitoring
            ..SamplingConfig::default()
        };
        
        let sampling_engine = match SamplingEngine::new(config) {
            Ok(engine) => {
                println!("✅ SamplingEngine created for monitoring with playthrough");
                Arc::new(engine)
            },
            Err(e) => {
                println!("❌ Failed to create SamplingEngine: {}", e);
                MONITORING_ACTIVE.store(false, std::sync::atomic::Ordering::Relaxed);
                return;
            }
        };
        
        // Store the engine globally for level access
        {
            let mut global_engine = GLOBAL_SAMPLING_ENGINE.lock().unwrap();
            *global_engine = Some(Arc::clone(&sampling_engine));
        }
        
        // Start monitoring with playthrough
        let (input_stream, output_stream) = match sampling_engine.start_monitoring_stream_with_playthrough(enable_playthrough) {
            Ok(streams) => streams,
            Err(e) => {
                println!("❌ Failed to create monitoring streams: {}", e);
                MONITORING_ACTIVE.store(false, std::sync::atomic::Ordering::Relaxed);
                return;
            }
        };
        
        // Start input stream
        use cpal::traits::StreamTrait;
        if let Err(e) = input_stream.play() {
            println!("❌ Failed to start input stream: {}", e);
            MONITORING_ACTIVE.store(false, std::sync::atomic::Ordering::Relaxed);
            return;
        }
        
        // Start output stream if playthrough enabled
        if let Some(ref output) = output_stream {
            if let Err(e) = output.play() {
                println!("❌ Failed to start output stream: {}", e);
                MONITORING_ACTIVE.store(false, std::sync::atomic::Ordering::Relaxed);
                return;
            }
        }
        
        println!("✅ Audio monitoring streams started (playthrough: {})", enable_playthrough);
        
        // Keep streams alive while monitoring is active
        while MONITORING_ACTIVE.load(std::sync::atomic::Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        
        // Stop streams
        if let Err(e) = input_stream.pause() {
            println!("⚠️ Warning: Failed to stop input stream: {}", e);
        }
        if let Some(ref output) = output_stream {
            if let Err(e) = output.pause() {
                println!("⚠️ Warning: Failed to stop output stream: {}", e);
            }
        }
        
        println!("✅ Audio monitoring streams stopped");
    });
    
    // Store the thread handle for cleanup
    {
        let mut thread_guard = MONITORING_THREAD.lock().unwrap();
        *thread_guard = Some(handle);
    }
    
    Ok("Audio monitoring with playthrough started successfully".to_string())
}

#[tauri::command]
async fn get_midi_connection_status() -> Result<bool, String> {
    let connection_guard = MIDI_CONNECTION.lock().unwrap();
    let is_connected = connection_guard.is_some();
    println!("🔍 MIDI connection status query: {}", if is_connected { "Connected" } else { "Disconnected" });
    Ok(is_connected)
}


/// Generate instrument files from existing WAV samples in a directory
#[tauri::command]
fn generate_instrument_files(directory: String, export_format: String, sample_name: Option<String>, creator_name: Option<String>, instrument_description: Option<String>) -> Result<String, String> {
    println!("🎹 GUI: Generating instrument files from directory: {}", directory);
    println!("   Format: {}, Sample name: {:?}", export_format, sample_name);
    
    use std::path::PathBuf;
    use std::collections::HashMap;
    use batcherbird_core::sampler::Sample;
    use batcherbird_core::export::{SampleExporter, ExportConfig, AudioFormat};
    use batcherbird_core::detection::DetectionConfig;
    
    let dir_path = PathBuf::from(&directory);
    if !dir_path.exists() || !dir_path.is_dir() {
        return Err(format!("Directory does not exist: {}", directory));
    }
    
    // Scan directory for WAV files
    let wav_files: Vec<PathBuf> = match std::fs::read_dir(&dir_path) {
        Ok(entries) => {
            entries.filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()?.to_str()? == "wav" {
                    Some(path)
                } else {
                    None
                }
            }).collect()
        },
        Err(e) => return Err(format!("Failed to read directory: {}", e))
    };
    
    if wav_files.is_empty() {
        return Err("No WAV files found in directory".to_string());
    }
    
    println!("   📁 Found {} WAV files", wav_files.len());
    
    // Parse WAV filenames to extract note and velocity information
    // Expected format: {prefix}_{note_name}_{note_number}_{velocity}.wav
    let mut samples = Vec::new();
    
    for wav_file in &wav_files {
        let filename = wav_file.file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("");
        
        // Parse filename: look for patterns like "Roland-EM1014_C4_60_vel127" or "Batcherbird_F4_v127_rk65"
        let note_number;
        let velocity;
        
        // Try pattern 1: "Roland-EM1014_C4_60_vel127" or "Roland-EM1017_B4_71_vel127"
        if let Some(captures) = regex::Regex::new(r".*_([A-G][#b]?\d+)_(\d+)_vel(\d+)$")
            .unwrap()
            .captures(filename) {
            
            let note_str = &captures[2];
            let velocity_str = &captures[3];
            
            if let (Ok(note), Ok(vel)) = (note_str.parse::<u8>(), velocity_str.parse::<u8>()) {
                note_number = note;
                velocity = vel;
            } else {
                println!("   ⚠️ Could not parse note/velocity from: {}", filename);
                continue;
            }
        }
        // Try pattern 2: "Batcherbird_F4_v127_rk65"  
        else if let Some(captures) = regex::Regex::new(r".*_([A-G][#b]?\d+)_v(\d+)_rk(\d+)$")
            .unwrap()
            .captures(filename) {
            
            let velocity_str = &captures[2];
            let note_str = &captures[3];
            
            if let (Ok(note), Ok(vel)) = (note_str.parse::<u8>(), velocity_str.parse::<u8>()) {
                note_number = note;
                velocity = vel;
            } else {
                println!("   ⚠️ Could not parse note/velocity from: {}", filename);
                continue;
            }
        }
        else {
            println!("   ⚠️ Filename format not recognized: {}", filename);
            continue;
        }
        
        // Create a minimal sample struct (we only need note/velocity for instrument file generation)
        let sample = Sample {
            note: note_number,
            velocity,
            audio_data: vec![0.0], // Dummy data - not used for instrument file generation
            sample_rate: 44100,   // Dummy data
            channels: 1,          // Dummy data
            recorded_at: std::time::SystemTime::now(),
            midi_timing: std::time::Duration::from_millis(100),
            audio_timing: std::time::Duration::from_millis(2000),
        };
        
        samples.push(sample);
        println!("   📄 Parsed: {} -> Note {}, Velocity {}", filename, note_number, velocity);
    }
    
    if samples.is_empty() {
        return Err("No valid samples found (could not parse filenames)".to_string());
    }
    
    // Determine export format
    let sample_format = match export_format.as_str() {
        "decentsampler" => AudioFormat::DecentSampler,
        "sfz" => AudioFormat::SFZ,
        _ => return Err(format!("Unsupported export format: {}", export_format))
    };
    
    // Build naming pattern 
    let naming_pattern = if let Some(name) = sample_name.as_ref().filter(|n| !n.trim().is_empty()) {
        format!("{}_{{note_name}}_{{note}}_vel{{velocity:03}}.wav", name.trim())
    } else {
        "{note_name}_{note}_vel{velocity:03}.wav".to_string()
    };
    
    // Create export config
    let export_config = ExportConfig {
        output_directory: dir_path.clone(),
        naming_pattern,
        sample_format: sample_format.clone(), // Clone to avoid move
        normalize: false,
        fade_in_ms: 0.0,
        fade_out_ms: 10.0,
        apply_detection: false, // Don't re-process existing samples
        detection_config: DetectionConfig::default(),
        creator_name: creator_name.clone(),
        instrument_description: instrument_description.clone(),
    };
    
    // Create exporter and generate instrument files
    let exporter = SampleExporter::new(export_config).map_err(|e| {
        format!("Failed to create exporter: {}", e)
    })?;
    
    // Generate instrument files using existing sample generation logic
    println!("🎼 Generating {} instrument file...", export_format);
    
    match sample_format {
        AudioFormat::DecentSampler => {
            // Group samples by velocity
            let mut velocity_groups = HashMap::new();
            for (i, sample) in samples.iter().enumerate() {
                if i < wav_files.len() {
                    velocity_groups.entry(sample.velocity)
                        .or_insert_with(Vec::new)
                        .push((sample, &wav_files[i]));
                }
            }
            
            let _preset_name = sample_name.unwrap_or_else(|| "Batcherbird_Instrument".to_string());
            let dspreset_path = exporter.generate_dspreset_file(&samples, &wav_files)
                .map_err(|e| format!("Failed to generate Decent Sampler file: {}", e))?;
            
            println!("   ✅ Generated: {}", dspreset_path.display());
            Ok(format!("Generated Decent Sampler file: {}", dspreset_path.display()))
        },
        AudioFormat::SFZ => {
            let sfz_path = exporter.generate_sfz_file(&samples, &wav_files)
                .map_err(|e| format!("Failed to generate SFZ file: {}", e))?;
            
            println!("   ✅ Generated: {}", sfz_path.display());
            Ok(format!("Generated SFZ file: {}", sfz_path.display()))
        },
        _ => Err("Invalid format for instrument file generation".to_string())
    }
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

/// Start real-time meter streaming using lock-free architecture
/// Streams meter data at ~60fps via Tauri channels for professional UI updates
#[tauri::command]
async fn start_realtime_meter_stream(app: tauri::AppHandle) -> Result<(), String> {
    use std::thread;
    use std::time::Duration;
    use rtrb::Consumer;
    
    // TODO: Get meter consumer from active recording/monitoring session
    // For now, return a placeholder implementation
    
    // Start meter streaming thread (60fps updates to UI)
    thread::spawn(move || {
        loop {
            // Check if we should stop streaming
            if !MONITORING_ACTIVE.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            
            // TODO: Pop meter data from ring buffer and emit
            // Placeholder meter data for testing
            let ui_meter_data = serde_json::json!({
                "peak_left": -12.0,
                "peak_right": -12.0,
                "rms_left": -18.0,
                "rms_right": -18.0,
                "is_clipping": false,
                "timestamp": 0,
            });
            
            // Emit to frontend via Tauri channel
            let _ = app.emit("meter_update", ui_meter_data);
            
            // Sleep for ~16ms (60fps)
            thread::sleep(Duration::from_millis(16));
        }
    });
    
    Ok(())
}

/// Get current audio levels for UI meters (simplified professional approach)
/// DEPRECATED: Use start_realtime_meter_stream for lock-free real-time updates
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

/// Get professional meter readings (Epic 3.1.3)
#[tauri::command]
async fn get_professional_meter_readings() -> Result<ProfessionalMeterReadings, String> {
    if !MONITORING_ACTIVE.load(std::sync::atomic::Ordering::Relaxed) {
        // Return silent readings when monitoring is off
        return Ok(ProfessionalMeterReadings {
            vu_db: -60.0,
            ppm_db: -60.0,
            peak_db: -60.0,
            peak_hold_db: -60.0,
            lufs: -70.0,
            gain_staging: batcherbird_core::GainStagingStatus::TooQuiet,
        });
    }
    
    // Initialize professional meter engine if needed
    {
        let mut meter_guard = PROFESSIONAL_METER_ENGINE.lock().unwrap();
        if meter_guard.is_none() {
            *meter_guard = Some(ProfessionalMeterEngine::new(44100.0));
        }
    }
    
    // Get current audio levels and convert to professional meter readings
    let engine_guard = GLOBAL_SAMPLING_ENGINE.lock().unwrap();
    if let Some(engine) = engine_guard.as_ref() {
        let levels = engine.get_audio_levels();
        
        // Convert the AudioLevels to realistic sample data for professional meter processing
        // Generate a short buffer that represents the current audio characteristics
        let buffer_size = 64; // Small buffer for meter processing
        let mut mock_samples = Vec::with_capacity(buffer_size);
        
        // Create samples that reflect the current RMS and peak characteristics
        let rms_amplitude = levels.rms;
        let peak_amplitude = levels.peak;
        
        // Generate a sine wave scaled to current RMS with occasional peaks
        for i in 0..buffer_size {
            let phase = 2.0 * std::f32::consts::PI * (i as f32) / (buffer_size as f32) * 4.0; // 4 cycles
            let base_sample = rms_amplitude * phase.sin();
            
            // Add occasional peaks that match the reported peak level
            let peak_factor = if i % 16 == 0 { peak_amplitude / rms_amplitude.max(0.001) } else { 1.0 };
            let sample = base_sample * peak_factor;
            
            mock_samples.push(sample.clamp(-1.0, 1.0));
        }
        
        // Process through professional meters
        let mut meter_guard = PROFESSIONAL_METER_ENGINE.lock().unwrap();
        if let Some(meter_engine) = meter_guard.as_mut() {
            let readings = meter_engine.process_samples(&mock_samples);
            Ok(readings)
        } else {
            Err("Professional meter engine not initialized".to_string())
        }
    } else {
        Err("Audio engine not available".to_string())
    }
}

/// Get gain staging analysis (Epic 3.1.3)
#[tauri::command]
async fn get_gain_staging_analysis() -> Result<GainStagingAnalysis, String> {
    if !MONITORING_ACTIVE.load(std::sync::atomic::Ordering::Relaxed) {
        return Err("Audio monitoring not active".to_string());
    }
    
    // Initialize gain staging assistant if needed
    {
        let mut assistant_guard = GAIN_STAGING_ASSISTANT.lock().unwrap();
        if assistant_guard.is_none() {
            *assistant_guard = Some(GainStagingAssistant::new());
        }
    }
    
    // Get professional meter readings first
    let readings = get_professional_meter_readings().await?;
    
    // Analyze with gain staging assistant
    let mut assistant_guard = GAIN_STAGING_ASSISTANT.lock().unwrap();
    if let Some(assistant) = assistant_guard.as_mut() {
        let analysis = assistant.analyze_level(&readings);
        Ok(analysis)
    } else {
        Err("Gain staging assistant not initialized".to_string())
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
fn create_directory(path: String) -> Result<bool, String> {
    println!("📁 Creating directory: {}", path);
    
    // Validate path for security
    let dir_path = validate_file_path(&path)?;
    
    if dir_path.exists() {
        println!("✅ Directory already exists: {}", path);
        return Ok(true);
    }
    
    match std::fs::create_dir_all(dir_path) {
        Ok(_) => {
            println!("✅ Successfully created directory: {}", path);
            Ok(true)
        },
        Err(e) => {
            println!("❌ Failed to create directory: {} - Error: {}", path, e);
            Err(format!("Failed to create directory '{}': {}", path, e))
        }
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
fn record_sample(note: u8, velocity: u8, duration: u32, output_directory: Option<String>, sample_name: Option<String>, _export_format: Option<String>, _creator_name: Option<String>, _instrument_description: Option<String>) -> Result<String, String> {
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
            ..SamplingConfig::default() // Use defaults for input_mode and input_channels
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
                format!("{}_{{note_name}}_{{note}}_vel{{velocity:03}}.wav", name.trim())
            } else {
                "{note_name}_{note}_vel{velocity:03}.wav".to_string()
            };
            
            // Single sample recording always exports WAV only - sampler files generated later
            let sample_format = AudioFormat::Wav24Bit; // Always WAV for individual samples
            
            let export_config = ExportConfig {
                output_directory: output_path,
                naming_pattern,
                sample_format,
                normalize: false, // Preserve original dynamics from core
                fade_in_ms: 0.0,
                fade_out_ms: 10.0,
                apply_detection: true, // Enable detection by default
                detection_config: Default::default(),
                creator_name: None, // No metadata needed for individual WAV files
                instrument_description: None, // No metadata needed for individual WAV files
            };
            
            println!("🔧 GUI: Creating sample exporter...");
            let exporter = SampleExporter::new(export_config).map_err(|e| {
                println!("❌ GUI: Failed to create exporter: {}", e);
                format!("Failed to create sample exporter: {}", e)
            })?;
            
            println!("💾 GUI: Exporting sample (WAV only)...");
            let file_path = exporter.export_sample(&recorded_sample).map_err(|e| {
                println!("❌ GUI: Export failed: {}", e);
                format!("Failed to export sample: {}", e)
            })?;
            
            println!("💾 GUI: Sample exported: {}", file_path.display());
            println!("   📂 Full path: {:?}", file_path);
            println!("   📂 Parent directory: {:?}", file_path.parent());
            
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

/// GUI Layer: Record sample synchronously - blocks until complete
#[tauri::command]
fn start_recording_with_viz(
    _app_handle: tauri::AppHandle,
    note: u8, 
    velocity: u8, 
    duration: u32, 
    output_directory: Option<String>, 
    sample_name: Option<String>, 
    export_format: Option<String>, 
    creator_name: Option<String>, 
    instrument_description: Option<String>
) -> Result<String, String> {
    println!("🎛️ GUI: Starting SYNCHRONOUS recording (note: {}, velocity: {}, duration: {}ms)", note, velocity, duration);
    
    // Step 1: Get MIDI connection
    let mut connection = {
        let mut connection_guard = MIDI_CONNECTION.lock().unwrap();
        match connection_guard.take() {
            Some(conn) => conn,
            None => return Err("No MIDI connection established. Please select a MIDI device first.".to_string()),
        }
    };
    
    // Step 2: Configure and create sampling engine
    println!("🔧 Configuring sampling engine...");
    let sampling_config = SamplingConfig {
        note_duration_ms: duration as u64,
        release_time_ms: 500,
        pre_delay_ms: 100,
        post_delay_ms: 100,
        midi_channel: 0,
        velocity,
        ..SamplingConfig::default()
    };
    
    let sampling_engine = match SamplingEngine::new(sampling_config) {
        Ok(engine) => engine,
        Err(e) => {
            *MIDI_CONNECTION.lock().unwrap() = Some(connection);
            return Err(format!("Failed to create sampling engine: {}", e));
        }
    };
    
    // Step 3: Record synchronously (blocks until complete)
    println!("🎵 Recording note {} synchronously...", note);
    let recorded_sample = match sampling_engine.sample_single_note_blocking(&mut connection, note) {
        Ok(sample) => {
            println!("✅ Recording completed: {} samples", sample.audio_data.len());
            sample
        },
        Err(e) => {
            *MIDI_CONNECTION.lock().unwrap() = Some(connection);
            return Err(format!("Recording failed: {}", e));
        }
    };
    
    // Step 4: Export synchronously (blocks until file is written)
    let file_path = export_sample_synchronously(
        recorded_sample, 
        output_directory, 
        sample_name, 
        export_format, 
        creator_name, 
        instrument_description
    )?;
    
    // Put connection back
    *MIDI_CONNECTION.lock().unwrap() = Some(connection);
    
    println!("✅ SYNCHRONOUS recording complete: {}", file_path);
    Ok(file_path)
}

// Helper function to export sample synchronously
fn export_sample_synchronously(
    recorded_sample: Sample,
    output_directory: Option<String>, 
    sample_name: Option<String>, 
    export_format: Option<String>, 
    creator_name: Option<String>, 
    instrument_description: Option<String>
) -> Result<String, String> {
    println!("✅ GUI: Core Audio Engine completed recording successfully");
    println!("📊 GUI: Received {} samples from Core Engine", recorded_sample.audio_data.len());
    
    // Handle export with metadata support
    let output_dir = if let Some(dir) = output_directory {
        if dir.trim().is_empty() {
            // Use Desktop/Batcherbird Samples when field is empty
            println!("⚠️ Empty output directory provided, using fallback");
            dirs::desktop_dir()
                .map(|desktop| desktop.join("Batcherbird Samples"))
                .unwrap_or_else(|| std::path::PathBuf::from("samples"))
                .to_string_lossy()
                .to_string()
        } else {
            println!("✅ Using session-initialized directory: {}", dir);
            dir
        }
    } else {
        // Default to Desktop/Batcherbird Samples
        println!("⚠️ No output directory provided, using fallback");
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
                format!("{}_{{note_name}}_{{note}}_vel{{velocity:03}}.wav", name.trim())
            } else {
                "{note_name}_{note}_vel{velocity:03}.wav".to_string()
            };
            
            // Determine export format based on user selection
            let sample_format = match export_format.as_deref() {
                Some("wav16") => AudioFormat::Wav16Bit,
                Some("wav24") => AudioFormat::Wav24Bit,
                Some("wav32") => AudioFormat::Wav32BitFloat,
                Some("dspreset") => AudioFormat::DecentSampler,
                Some("sfz") => AudioFormat::SFZ,
                Some("all") => AudioFormat::Wav24Bit, // All formats - start with 24-bit WAV, additional formats generated later
                _ => AudioFormat::Wav24Bit, // Default to 24-bit WAV
            };
    
    println!("📁 GUI: Using output directory: {}", output_path.display());
    
    // Build naming pattern with optional sample name prefix
    let naming_pattern = if let Some(name) = sample_name.as_ref().filter(|n| !n.trim().is_empty()) {
        format!("{}_{{note_name}}_{{note}}_vel{{velocity:03}}.wav", name.trim())
    } else {
        "{note_name}_{note}_vel{velocity:03}.wav".to_string()
    };
    
    // Determine export format based on user selection
    let sample_format = match export_format.as_deref() {
        Some("wav16") => AudioFormat::Wav16Bit,
        Some("wav24") => AudioFormat::Wav24Bit,
        Some("wav32") => AudioFormat::Wav32BitFloat,
        Some("dspreset") => AudioFormat::DecentSampler,
        Some("sfz") => AudioFormat::SFZ,
        Some("all") => AudioFormat::Wav24Bit, // All formats - start with 24-bit WAV, additional formats generated later
        _ => AudioFormat::Wav24Bit, // Default to 24-bit WAV
    };
    
    let export_config = ExportConfig {
        output_directory: output_path.clone(),
        naming_pattern,
        sample_format,
        normalize: false, // Preserve original dynamics from core
        fade_in_ms: 0.0,
        fade_out_ms: 10.0,
        apply_detection: true, // Enable detection by default
        detection_config: Default::default(),
        creator_name: creator_name.clone(),
        instrument_description: instrument_description.clone(),
    };
    
    println!("🔧 GUI: Creating sample exporter...");
    let exporter = match SampleExporter::new(export_config) {
        Ok(exporter) => exporter,
        Err(e) => {
            println!("❌ GUI: Failed to create exporter: {}", e);
            return Err(format!("Failed to create sample exporter: {}", e));
        }
    };
    
    println!("💾 GUI: Exporting sample (WAV only)...");
    let file_path = match exporter.export_sample(&recorded_sample) {
        Ok(path) => path,
        Err(e) => {
            println!("❌ GUI: Export failed: {}", e);
            return Err(format!("Failed to export sample: {}", e));
        }
    };
    
    let file_path_str = file_path.to_string_lossy().to_string();
    println!("✅ GUI: Sample recorded and saved to: {}", file_path_str);
    
    Ok(file_path_str)
}

/// Test command to verify Tauri channel throughput at 60fps
#[tauri::command]
fn test_viz_throughput(app_handle: tauri::AppHandle) -> Result<String, String> {
    println!("🧪 Testing visualization throughput at 60fps...");
    
    // Create a test ring buffer
    let (mut producer, mut consumer) = rtrb::RingBuffer::<VizChunk>::new(64);
    
    // Producer thread (simulates audio thread)
    let producer_handle = std::thread::spawn(move || {
        for i in 0..600 { // 10 seconds worth of 60fps data
            let test_samples = vec![0.1 * (i as f32), -0.1 * (i as f32)];
            let chunk = VizChunk::from_samples(&test_samples, i * 2);
            
            // Try to push (never block)
            if producer.push(chunk).is_err() {
                println!("⚠️ Buffer full at chunk {}", i);
            }
            
            // Simulate audio callback timing (roughly 1ms chunks)
            std::thread::sleep(Duration::from_millis(1));
        }
        println!("✅ Producer finished sending 600 chunks");
    });
    
    // Consumer thread (simulates visualization thread)
    let app_clone = app_handle.clone();
    let consumer_handle = std::thread::spawn(move || {
        let mut chunks_sent = 0;
        let start_time = std::time::Instant::now();
        
        for _ in 0..600 { // 10 seconds at 60fps = 600 iterations
            // Try to consume available chunks
            while let Ok(chunk) = consumer.pop() {
                // Send via Tauri channel
                if let Err(e) = app_clone.emit("viz_test_chunk", &chunk) {
                    println!("⚠️ Failed to emit chunk: {}", e);
                } else {
                    chunks_sent += 1;
                }
            }
            
            // 60fps timing
            std::thread::sleep(Duration::from_millis(16));
        }
        
        let elapsed = start_time.elapsed();
        println!("✅ Consumer finished: {} chunks sent in {:.2}s", chunks_sent, elapsed.as_secs_f32());
        chunks_sent
    });
    
    // Wait for both threads
    producer_handle.join().unwrap();
    let chunks_sent = consumer_handle.join().unwrap();
    
    let result = format!("Throughput test completed: {} chunks sent via Tauri channels", chunks_sent);
    println!("🧪 {}", result);
    Ok(result)
}

/// Cancel an ongoing recording
#[tauri::command]
fn cancel_recording() -> Result<String, String> {
    println!("🛑 Cancelling recording...");
    RECORDING_CANCELLED.store(true, std::sync::atomic::Ordering::Relaxed);
    Ok("Recording cancelled".to_string())
}

/// Save a recording session for later recovery
#[tauri::command]
fn save_recording_session(
    start_note: u8,
    end_note: u8,
    velocity_layers: Vec<u8>,
    duration: u32,
    output_directory: String,
    sample_name: String,
    export_format: String,
    creator_name: Option<String>,
    instrument_description: Option<String>,
    note_to_note_delay: u32,
    layer_to_layer_delay: u32,
) -> Result<String, String> {
    let session = RecordingSession::new(
        start_note,
        end_note,
        velocity_layers,
        duration,
        output_directory,
        sample_name,
        export_format,
        creator_name,
        instrument_description,
        note_to_note_delay,
        layer_to_layer_delay,
    );
    
    session.save_to_file()?;
    Ok(session.session_id)
}

/// Update session with completed recording
#[tauri::command]
fn update_session_progress(
    session_id: String,
    note: u8,
    velocity: u8,
    file_path: String,
) -> Result<f32, String> {
    let mut session = RecordingSession::load_from_file(&session_id)?;
    session.add_completed_recording(note, velocity, file_path);
    session.save_to_file()?;
    Ok(session.get_progress())
}

/// Get list of recoverable sessions
#[tauri::command]
fn get_recoverable_sessions() -> Result<Vec<SessionInfo>, String> {
    session::list_sessions()
}

/// Resume a recording session from where it left off
#[tauri::command]
fn resume_recording_session(session_id: String) -> Result<serde_json::Value, String> {
    let session = RecordingSession::load_from_file(&session_id)?;
    
    // Return session data as JSON for frontend to use
    serde_json::to_value(&session)
        .map_err(|e| format!("Failed to serialize session: {}", e))
}

/// Delete a recording session
#[tauri::command]
fn delete_recording_session(session_id: String) -> Result<String, String> {
    let session = RecordingSession::load_from_file(&session_id)?;
    session.delete_session_file()?;
    Ok("Session deleted".to_string())
}

/// Record a range of notes with multiple velocity layers
/// Epic 4.1: Intelligent Velocity Layer Recording System
#[tauri::command]
async fn record_range_with_velocity_layers(
    app: tauri::AppHandle,
    start_note: u8, 
    end_note: u8, 
    velocity_layers: Vec<u8>,  // e.g., [32, 64, 96, 127] for 4 layers
    duration: u32, 
    output_directory: Option<String>, 
    sample_name: Option<String>, 
    export_format: Option<String>, 
    creator_name: Option<String>, 
    instrument_description: Option<String>,
    note_to_note_delay: Option<u32>,  // Delay between notes in ms (default: 200)
    layer_to_layer_delay: Option<u32>  // Delay between velocity layers in ms (default: 500)
) -> Result<String, String> {
    use batcherbird_core::export::{SampleExporter, ExportConfig, AudioFormat};
    use batcherbird_core::detection::DetectionConfig;
    
    println!("🎹 GUI: Recording range with velocity layers");
    println!("   Notes: {}-{}, Velocities: {:?}, Duration: {}ms", start_note, end_note, velocity_layers, duration);
    
    // Validate inputs
    if velocity_layers.is_empty() {
        return Err("No velocity layers specified".to_string());
    }
    if start_note > end_note {
        return Err("Invalid note range".to_string());
    }
    
    // Get MIDI connection
    let mut connection = {
        let mut connection_guard = MIDI_CONNECTION.lock().unwrap();
        match connection_guard.take() {
            Some(conn) => conn,
            None => return Err("No MIDI connection established. Please select a MIDI device first.".to_string()),
        }
    };
    
    // Calculate total samples to record
    let note_count = (end_note - start_note + 1) as usize;
    let velocity_count = velocity_layers.len();
    let total_samples = note_count * velocity_count;
    println!("📊 Total samples to record: {} notes × {} velocities = {}", note_count, velocity_count, total_samples);
    
    // Range sampling with velocity layers in dedicated thread
    println!("📡 GUI: Starting multi-velocity range recording...");
    
    // Reset cancellation flag
    RECORDING_CANCELLED.store(false, std::sync::atomic::Ordering::Relaxed);
    
    // Clone app handle for thread
    let app_handle = app.clone();
    
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        println!("🧵 Multi-velocity range thread started");
        
        // Create sampling engine
        let sampling_config = SamplingConfig {
            note_duration_ms: duration as u64,
            release_time_ms: 500,  // Professional standard
            pre_delay_ms: 100,     
            post_delay_ms: 100,    
            midi_channel: 0,       
            velocity: 127,         // Will be overridden per layer
            ..SamplingConfig::default()
        };
        
        let mut sampling_engine = match SamplingEngine::new(sampling_config) {
            Ok(engine) => {
                println!("✅ SamplingEngine created for multi-velocity recording");
                engine
            },
            Err(e) => {
                println!("❌ Failed to create SamplingEngine: {}", e);
                let _ = tx.send((Err(e), connection));
                return;
            }
        };
        
        // Collect all samples
        let mut all_samples = Vec::new();
        let mut completed = 0;
        
        // Record each velocity layer
        for (layer_idx, &velocity) in velocity_layers.iter().enumerate() {
            // Check for cancellation
            if RECORDING_CANCELLED.load(std::sync::atomic::Ordering::Relaxed) {
                println!("⚠️ Recording cancelled by user");
                break;
            }
            
            println!("🎼 Recording velocity layer {} of {} (velocity: {})", layer_idx + 1, velocity_count, velocity);
            
            // Create new config with updated velocity
            let sampling_config = SamplingConfig {
                note_duration_ms: duration as u64,
                release_time_ms: 500,
                pre_delay_ms: 100,
                post_delay_ms: 100,
                midi_channel: 0,
                velocity,
                ..SamplingConfig::default()
            };
            
            // Recreate engine with new velocity
            sampling_engine = match SamplingEngine::new(sampling_config) {
                Ok(engine) => engine,
                Err(e) => {
                    println!("❌ Failed to create SamplingEngine for velocity {}: {}", velocity, e);
                    let _ = tx.send((Err(e), connection));
                    return;
                }
            };
            
            // Record this velocity layer for all notes
            for note in start_note..=end_note {
                // Check for cancellation
                if RECORDING_CANCELLED.load(std::sync::atomic::Ordering::Relaxed) {
                    println!("⚠️ Recording cancelled by user");
                    break;
                }
                
                println!("   🎵 Recording note {} at velocity {}", note, velocity);
                
                match sampling_engine.sample_single_note_blocking(&mut connection, note) {
                    Ok(mut sample) => {
                        // Override velocity in case it wasn't set correctly
                        sample.velocity = velocity;
                        all_samples.push(sample);
                        completed += 1;
                        
                        // Progress update
                        let progress = (completed as f32 / total_samples as f32) * 100.0;
                        println!("   ✅ Sample {}/{} recorded ({:.1}%)", completed, total_samples, progress);
                        
                        // Emit progress event to frontend
                        let progress_data = serde_json::json!({
                            "current": completed,
                            "total": total_samples,
                            "percent": progress,
                            "note": note,
                            "velocity": velocity,
                            "layer": layer_idx + 1,
                            "totalLayers": velocity_count,
                            "noteName": note_to_name(note)
                        });
                        let _ = app_handle.emit("recording_progress", progress_data);
                    },
                    Err(e) => {
                        println!("   ❌ Failed to record note {} at velocity {}: {}", note, velocity, e);
                        // Continue with other samples instead of failing completely
                    }
                }
                
                // Small delay between notes to avoid MIDI/audio issues
                let delay_ms = note_to_note_delay.unwrap_or(200);
                std::thread::sleep(std::time::Duration::from_millis(delay_ms as u64));
            }
            
            // Delay between velocity layers
            if layer_idx < velocity_count - 1 {
                let delay_ms = layer_to_layer_delay.unwrap_or(500);
                println!("⏸️ Pausing {}ms before next velocity layer...", delay_ms);
                std::thread::sleep(std::time::Duration::from_millis(delay_ms as u64));
            }
        }
        
        println!("✅ Multi-velocity recording complete: {} samples collected", all_samples.len());
        
        // Send results back
        if all_samples.is_empty() {
            let _ = tx.send((Err(BatcherbirdError::Audio("No samples recorded".to_string())), connection));
        } else {
            let _ = tx.send((Ok(all_samples), connection));
        }
    });
    
    // Wait for recording to complete
    match rx.recv() {
        Ok((result, mut conn)) => {
            // Restore MIDI connection for future use
            {
                let mut connection_guard = MIDI_CONNECTION.lock().unwrap();
                *connection_guard = Some(conn);
            }
            
            // Process results
            match result {
                Ok(samples) => {
                    println!("✅ GUI: Received {} samples from multi-velocity recording", samples.len());
                    
                    // Export samples
                    let output_path = if let Some(dir) = output_directory {
                        std::path::PathBuf::from(dir)
                    } else {
                        dirs::desktop_dir()
                            .unwrap_or_else(|| std::path::PathBuf::from("."))
                            .join("Batcherbird Samples")
                    };
                    
                    // Create export configuration
                    let naming_pattern = if let Some(name) = sample_name.as_ref() {
                        format!("{}_{{note_name}}_{{note}}_vel{{velocity:03}}.wav", name)
                    } else {
                        "Sample_{note_name}_{note}_vel{velocity:03}.wav".to_string()
                    };
                    
                    let export_config = ExportConfig {
                        output_directory: output_path.clone(),
                        naming_pattern,
                        sample_format: match export_format.as_deref() {
                            Some("decentsampler") => AudioFormat::DecentSampler,
                            Some("sfz") => AudioFormat::SFZ,
                            _ => AudioFormat::Wav24Bit,
                        },
                        normalize: false,
                        fade_in_ms: 0.0,
                        fade_out_ms: 10.0,
                        apply_detection: true,
                        detection_config: DetectionConfig::default(),
                        creator_name: creator_name.clone(),
                        instrument_description: instrument_description.clone(),
                    };
                    
                    // Export all samples
                    let exporter = SampleExporter::new(export_config).map_err(|e| {
                        format!("Failed to create exporter: {}", e)
                    })?;
                    
                    let exported_paths = exporter.export_samples(&samples).map_err(|e| {
                        format!("Failed to export samples: {}", e)
                    })?;
                    
                    println!("✅ Exported {} files to {}", exported_paths.len(), output_path.display());
                    
                    Ok(format!("Multi-velocity range recording complete! {} samples saved to {}", 
                        exported_paths.len(), output_path.display()))
                },
                Err(e) => {
                    println!("❌ Multi-velocity recording error: {}", e);
                    Err(format!("Multi-velocity recording failed: {}", e))
                }
            }
        },
        Err(e) => {
            println!("❌ GUI: Thread communication error: {}", e);
            Err("Recording thread communication failed".to_string())
        }
    }
}

#[tauri::command]
fn record_range(start_note: u8, end_note: u8, velocity: u8, duration: u32, output_directory: Option<String>, sample_name: Option<String>, export_format: Option<String>, creator_name: Option<String>, instrument_description: Option<String>) -> Result<String, String> {
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
            ..SamplingConfig::default() // Use defaults for input_mode and input_channels
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
            
            // Filter out empty samples
            let valid_samples: Vec<_> = samples.into_iter()
                .filter(|sample| {
                    if sample.audio_data.is_empty() {
                        println!("⚠️ GUI: Warning - Sample (note {}) has no audio data, skipping", sample.note);
                        false
                    } else {
                        true
                    }
                })
                .collect();
            
            if valid_samples.is_empty() {
                return Err("No valid samples to export".to_string());
            }
            
            // Build naming pattern with optional sample name prefix (consistent with single sample recording)
            let naming_pattern = if let Some(name) = sample_name.as_ref().filter(|n| !n.trim().is_empty()) {
                format!("{}_{{note_name}}_{{note}}_vel{{velocity:03}}.wav", name.trim())
            } else {
                "{note_name}_{note}_vel{velocity:03}.wav".to_string()
            };
            
            // Determine sample format based on frontend selection
            let sample_format = match export_format.as_deref() {
                Some("decentsampler") => AudioFormat::DecentSampler,
                Some("sfz") => AudioFormat::SFZ,
                Some("kontakt") => AudioFormat::Wav24Bit, // For future Kontakt export
                Some("all") => AudioFormat::Wav24Bit, // Default for "all formats" 
                _ => AudioFormat::Wav32BitFloat, // Default: high-quality WAV
            };
            
            // Create single exporter for all samples - this enables .dspreset/.sfz generation
            let export_config = ExportConfig {
                output_directory: output_path.clone(),
                naming_pattern,
                sample_format,
                normalize: false,
                fade_in_ms: 0.0,
                fade_out_ms: 10.0,
                apply_detection: true, // Enable detection by default
                detection_config: Default::default(),
                creator_name: creator_name.clone(),
                instrument_description: instrument_description.clone(),
            };
            
            println!("🔧 GUI: Creating batch exporter for {} samples...", valid_samples.len());
            let exporter = SampleExporter::new(export_config).map_err(|e| {
                println!("❌ GUI: Failed to create batch exporter: {}", e);
                format!("Failed to create sample exporter: {}", e)
            })?;
            
            // Export all samples as a batch - this will create .dspreset/.sfz files automatically
            println!("💾 GUI: Batch exporting {} samples...", valid_samples.len());
            let exported_file_paths = exporter.export_samples(&valid_samples).map_err(|e| {
                println!("❌ GUI: Batch export failed: {}", e);
                format!("Failed to export samples: {}", e)
            })?;
            
            // Convert paths to filenames for display
            let exported_files: Vec<String> = exported_file_paths.iter()
                .map(|path| path.file_name().unwrap().to_string_lossy().to_string())
                .collect();
            
            println!("✅ GUI: Successfully batch exported {} files:", exported_files.len());
            for filename in &exported_files {
                println!("   📄 {}", filename);
            }
            
            let success_message = format!("Range recording complete! {} files saved to:\n{}", 
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

/// Apply loop detection to a sample file
#[tauri::command]
fn detect_loop_points(file_path: String, min_loop_length: Option<f32>, max_loop_length: Option<f32>, correlation_threshold: Option<f32>) -> Result<String, String> {
    println!("🔄 GUI: Detecting loop points for: {}", file_path);
    
    use batcherbird_core::sampler::Sample;
    
    // Validate and load the audio file
    let path = validate_file_path(&file_path)?;
    if !path.exists() {
        let response = LoopDetectionResponse {
            success: false,
            sample_rate: 44100, // Default fallback
            candidates: vec![],
            best_candidate: None,
            failure_reason: Some(format!("File not found: {}", file_path)),
        };
        return Ok(serde_json::to_string(&response).map_err(|e| format!("JSON serialization error: {}", e))?);
    }
    
    // Read the WAV file
    match hound::WavReader::open(&path) {
        Ok(mut reader) => {
            let spec = reader.spec();
            println!("   📊 Audio specs: {}Hz, {} channels, {} bits", 
                    spec.sample_rate, spec.channels, spec.bits_per_sample);
            
            // Read samples based on bit depth
            let samples: Result<Vec<f32>, _> = match spec.sample_format {
                hound::SampleFormat::Float => {
                    reader.samples::<f32>().collect()
                },
                hound::SampleFormat::Int => {
                    match spec.bits_per_sample {
                        16 => {
                            reader.samples::<i16>()
                                .map(|s| s.map(|sample| sample as f32 / i16::MAX as f32))
                                .collect()
                        },
                        24 => {
                            reader.samples::<i32>()
                                .map(|s| s.map(|sample| sample as f32 / 8_388_607.0)) // 24-bit max
                                .collect()
                        },
                        32 => {
                            reader.samples::<i32>()
                                .map(|s| s.map(|sample| sample as f32 / i32::MAX as f32))
                                .collect()
                        },
                        _ => {
                            let response = LoopDetectionResponse {
                                success: false,
                                sample_rate: spec.sample_rate,
                                candidates: vec![],
                                best_candidate: None,
                                failure_reason: Some(format!("Unsupported bit depth: {}", spec.bits_per_sample)),
                            };
                            return Ok(serde_json::to_string(&response).map_err(|e| format!("JSON serialization error: {}", e))?);
                        }
                    }
                }
            };
            
            let audio_data = match samples {
                Ok(data) => data,
                Err(e) => {
                    let response = LoopDetectionResponse {
                        success: false,
                        sample_rate: spec.sample_rate,
                        candidates: vec![],
                        best_candidate: None,
                        failure_reason: Some(format!("Failed to read audio data: {}", e)),
                    };
                    return Ok(serde_json::to_string(&response).map_err(|e| format!("JSON serialization error: {}", e))?);
                }
            };
            
            if audio_data.is_empty() {
                let response = LoopDetectionResponse {
                    success: false,
                    sample_rate: spec.sample_rate,
                    candidates: vec![],
                    best_candidate: None,
                    failure_reason: Some("No audio data found in file".to_string()),
                };
                return Ok(serde_json::to_string(&response).map_err(|e| format!("JSON serialization error: {}", e))?);
            }
            
            println!("   📄 Loaded {} samples ({:.2}s)", 
                    audio_data.len(), 
                    audio_data.len() as f32 / spec.sample_rate as f32);
            
            // Create a temporary sample for loop detection
            let mut sample = Sample {
                note: 60, // Middle C - not used for loop detection
                velocity: 127,
                audio_data,
                sample_rate: spec.sample_rate,
                channels: spec.channels,
                recorded_at: std::time::SystemTime::now(),
                midi_timing: std::time::Duration::from_millis(100),
                audio_timing: std::time::Duration::from_millis(2000),
            };
            
            // Configure loop detection
            let mut config = LoopDetectionConfig::default();
            if let Some(min_len) = min_loop_length {
                config.min_loop_length_sec = min_len;
            }
            if let Some(max_len) = max_loop_length {
                config.max_loop_length_sec = max_len;
            }
            if let Some(threshold) = correlation_threshold {
                config.correlation_threshold = threshold;
            }
            
            println!("   🔧 Loop detection config: {:.1}s-{:.1}s, threshold: {:.2}", 
                    config.min_loop_length_sec, config.max_loop_length_sec, config.correlation_threshold);
            
            // Apply loop detection
            match sample.apply_loop_detection(config) {
                Ok(result) => {
                    // Convert candidates to response format
                    let candidates_response: Vec<LoopCandidateResponse> = result.all_candidates
                        .into_iter()
                        .map(|candidate| LoopCandidateResponse {
                            start_sample: candidate.start_sample,
                            end_sample: candidate.end_sample,
                            length_samples: candidate.length_samples,
                            quality_score: candidate.quality_score,
                            zero_crossing_aligned: candidate.zero_crossing_aligned,
                            correlation: candidate.correlation,
                        })
                        .collect();
                    
                    let best_candidate_response = result.best_candidate.map(|candidate| LoopCandidateResponse {
                        start_sample: candidate.start_sample,
                        end_sample: candidate.end_sample,
                        length_samples: candidate.length_samples,
                        quality_score: candidate.quality_score,
                        zero_crossing_aligned: candidate.zero_crossing_aligned,
                        correlation: candidate.correlation,
                    });
                    
                    let response = LoopDetectionResponse {
                        success: result.success,
                        sample_rate: sample.sample_rate,
                        candidates: candidates_response,
                        best_candidate: best_candidate_response,
                        failure_reason: result.failure_reason,
                    };
                    
                    if result.success {
                        if let Some(ref candidate) = response.best_candidate {
                            println!("   ✅ Loop detected: {:.3}s-{:.3}s, quality: {:.3}", 
                                    candidate.start_sample as f32 / sample.sample_rate as f32,
                                    candidate.end_sample as f32 / sample.sample_rate as f32,
                                    candidate.quality_score);
                        }
                    } else {
                        println!("   ❌ Loop detection failed: {}", 
                                response.failure_reason.as_ref().unwrap_or(&"Unknown reason".to_string()));
                    }
                    
                    Ok(serde_json::to_string(&response).map_err(|e| format!("JSON serialization error: {}", e))?)
                },
                Err(e) => {
                    let response = LoopDetectionResponse {
                        success: false,
                        sample_rate: sample.sample_rate,
                        candidates: vec![],
                        best_candidate: None,
                        failure_reason: Some(format!("Loop detection error: {}", e)),
                    };
                    println!("   ❌ Loop detection error: {}", e);
                    Ok(serde_json::to_string(&response).map_err(|e| format!("JSON serialization error: {}", e))?)
                }
            }
        },
        Err(e) => {
            let response = LoopDetectionResponse {
                success: false,
                sample_rate: 44100, // Default fallback
                candidates: vec![],
                best_candidate: None,
                failure_reason: Some(format!("Failed to open WAV file: {}", e)),
            };
            println!("   ❌ Failed to open WAV file: {}", e);
            Ok(serde_json::to_string(&response).map_err(|e| format!("JSON serialization error: {}", e))?)
        }
    }
}

/// Apply loop metadata to a WAV file
#[tauri::command]
fn apply_loop_metadata(file_path: String, start_sample: usize, end_sample: usize, sample_rate: u32) -> Result<String, String> {
    println!("🔄 GUI: Applying loop metadata to: {}", file_path);
    println!("   Loop: samples {}-{} (rate: {}Hz)", start_sample, end_sample, sample_rate);
    
    // Validate path for security
    let path = validate_file_path(&file_path)?;
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }
    
    // For now, we'll store the loop metadata in a companion file
    // TODO: Implement actual WAV metadata embedding
    let metadata_path = path.with_extension("loop.json");
    
    let loop_metadata = serde_json::json!({
        "version": "1.0",
        "loop_start_sample": start_sample,
        "loop_end_sample": end_sample,
        "loop_start_time": start_sample as f64 / sample_rate as f64,
        "loop_end_time": end_sample as f64 / sample_rate as f64,
        "sample_rate": sample_rate,
        "applied_at": chrono::Utc::now().to_rfc3339()
    });
    
    match std::fs::write(&metadata_path, serde_json::to_string_pretty(&loop_metadata).unwrap()) {
        Ok(_) => {
            println!("✅ GUI: Loop metadata saved to: {}", metadata_path.display());
            Ok(format!("Loop metadata applied and saved to: {}", metadata_path.display()))
        },
        Err(e) => {
            println!("❌ GUI: Failed to save loop metadata: {}", e);
            Err(format!("Failed to save loop metadata: {}", e))
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
async fn select_audio_file(app: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    use std::sync::mpsc;
    
    println!("🎵 Opening native file picker for audio files...");
    
    let (tx, rx) = mpsc::channel();
    
    app.dialog()
        .file()
        .set_title("Select Audio File")
        .add_filter("Audio Files", &["wav", "mp3", "flac", "aiff", "m4a", "ogg"])
        .add_filter("WAV Files", &["wav"])
        .add_filter("All Files", &["*"])
        .pick_file(move |file_path| {
            let _ = tx.send(file_path);
        });
    
    match rx.recv() {
        Ok(Some(path)) => {
            let path_str = path.to_string();
            println!("✅ User selected audio file: {}", path_str);
            Ok(path_str)
        }
        Ok(None) => {
            println!("❌ User cancelled file selection");
            Err("File selection cancelled".to_string())
        }
        Err(e) => {
            println!("❌ File picker error: {}", e);
            Err(format!("File picker error: {}", e))
        }
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

/// Get the path of the most recently recorded sample file
#[tauri::command]
fn get_last_recorded_sample_path(output_directory: Option<String>, sample_name: Option<String>) -> Result<String, String> {
    println!("🔍 GUI: Finding last recorded sample path");
    println!("   📁 Output directory param: {:?}", output_directory);
    println!("   📁 Sample name param: {:?}", sample_name);
    
    use std::path::PathBuf;
    use std::fs;
    use std::time::SystemTime;
    
    // Determine search directory
    let search_dir = if let Some(dir) = output_directory {
        if dir.trim().is_empty() {
            // Use Desktop/Batcherbird Samples when field is empty
            dirs::desktop_dir()
                .map(|desktop| desktop.join("Batcherbird Samples"))
                .unwrap_or_else(|| PathBuf::from("samples"))
        } else {
            PathBuf::from(dir)
        }
    } else {
        // Default to Desktop/Batcherbird Samples
        dirs::desktop_dir()
            .map(|desktop| desktop.join("Batcherbird Samples"))
            .unwrap_or_else(|| PathBuf::from("samples"))
    };
    
    // Add subdirectory if sample name is provided
    let mut search_path = search_dir;
    if let Some(name) = sample_name.as_ref().filter(|n| !n.trim().is_empty()) {
        search_path = search_path.join(name.trim());
    }
    
    println!("   📁 Searching in: {}", search_path.display());
    
    if !search_path.exists() {
        return Err(format!("Directory does not exist: {}", search_path.display()));
    }
    
    // Find all WAV files in the directory
    let entries = fs::read_dir(&search_path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    
    let mut wav_files: Vec<(PathBuf, SystemTime)> = Vec::new();
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        println!("   🔍 Checking file: {}", path.display());
        
        // Check if it's a WAV file
        if path.extension().and_then(|ext| ext.to_str()) == Some("wav") {
            println!("   ✓ Found WAV file: {}", path.display());
            // Get modification time
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    wav_files.push((path, modified));
                }
            }
        }
    }
    
    if wav_files.is_empty() {
        return Err("No WAV files found in directory".to_string());
    }
    
    // Sort by modification time (most recent first)
    wav_files.sort_by(|a, b| b.1.cmp(&a.1));
    
    let latest_file = &wav_files[0].0;
    println!("   ✅ Found latest sample: {}", latest_file.display());
    
    Ok(latest_file.to_string_lossy().to_string())
}

/// Extract waveform data from an audio file for visualization
#[tauri::command]
async fn get_waveform_data(file_path: String, resolution: Option<u32>) -> Result<WaveformData, String> {
    println!("🌊 GUI: Extracting waveform data from: {}", file_path);
    
    use hound::WavReader;
    
    // Default resolution: 800 points (matches UI width)
    let target_resolution = resolution.unwrap_or(800);
    
    // Validate and verify file exists
    let path = validate_file_path(&file_path)?;
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }
    
    // Open WAV file
    let mut reader = WavReader::open(&file_path)
        .map_err(|e| format!("Failed to open WAV file: {}", e))?;
    
    let spec = reader.spec();
    let sample_rate = spec.sample_rate;
    let channels = spec.channels as u8;
    let total_samples = reader.len() as usize;
    let samples_per_channel = total_samples / channels as usize;
    
    // Calculate duration
    let duration = samples_per_channel as f64 / sample_rate as f64;
    
    // Calculate downsampling factor
    let samples_per_pixel = (samples_per_channel as f64 / target_resolution as f64).max(1.0);
    let chunk_size = samples_per_pixel as usize;
    
    println!("   📊 Sample rate: {} Hz, Channels: {}, Duration: {:.2}s", sample_rate, channels, duration);
    println!("   📊 Total samples: {}, Samples per pixel: {}", total_samples, chunk_size);
    
    // Read and process samples
    let mut positive_peaks = Vec::with_capacity(target_resolution as usize);
    let mut negative_peaks = Vec::with_capacity(target_resolution as usize);
    
    // Convert samples to f32 based on bit depth
    let samples: Vec<f32> = match spec.bits_per_sample {
        16 => {
            reader.samples::<i16>()
                .filter_map(Result::ok)
                .map(|s| s as f32 / i16::MAX as f32)
                .collect()
        },
        24 => {
            reader.samples::<i32>()
                .filter_map(Result::ok)
                .map(|s| (s >> 8) as f32 / (1 << 23) as f32)
                .collect()
        },
        32 => {
            reader.samples::<f32>()
                .filter_map(Result::ok)
                .collect()
        },
        _ => return Err(format!("Unsupported bit depth: {}", spec.bits_per_sample)),
    };
    
    // Process in chunks to find peaks
    for chunk_start in (0..samples_per_channel).step_by(chunk_size) {
        let chunk_end = (chunk_start + chunk_size).min(samples_per_channel);
        
        let mut max_positive = 0.0f32;
        let mut max_negative = 0.0f32;
        
        // For stereo, we'll take the maximum of both channels
        for i in chunk_start..chunk_end {
            if channels == 1 {
                let sample = samples.get(i).copied().unwrap_or(0.0);
                if sample > max_positive {
                    max_positive = sample;
                }
                if sample < max_negative {
                    max_negative = sample;
                }
            } else {
                // For stereo, check both channels
                for ch in 0..channels as usize {
                    let idx = i * channels as usize + ch;
                    if let Some(&sample) = samples.get(idx) {
                        if sample > max_positive {
                            max_positive = sample;
                        }
                        if sample < max_negative {
                            max_negative = sample;
                        }
                    }
                }
            }
        }
        
        positive_peaks.push(max_positive);
        negative_peaks.push(max_negative.abs()); // Store as positive for easier rendering
    }
    
    println!("   ✅ Generated {} waveform points", positive_peaks.len());
    
    Ok(WaveformData {
        peaks: WaveformPeaks {
            positive: positive_peaks,
            negative: negative_peaks,
        },
        sample_rate,
        duration,
        channels,
        format: if channels == 1 { "mono".to_string() } else { "stereo".to_string() },
    })
}

/// Load audio file for playback
#[tauri::command]
async fn load_sample_for_playback(file_path: String) -> Result<String, String> {
    println!("🎵 GUI: Loading sample for playback: {}", file_path);
    
    // Validate file path for security
    let validated_path = validate_file_path(&file_path)?;
    
    // Initialize playback engine if needed
    let mut playback_guard = AUDIO_PLAYBACK.lock().unwrap();
    if playback_guard.is_none() {
        println!("   🔧 Creating audio playback instance");
        match AudioPlayback::new() {
            Ok(playback) => {
                let playback_arc = Arc::new(playback);
                // Initialize the audio engine (starts the heartbeat thread)
                playback_arc.initialize_audio_engine()
                    .map_err(|e| format!("Failed to initialize audio engine: {}", e))?;
                *playback_guard = Some(playback_arc);
            }
            Err(e) => {
                return Err(format!("Failed to create playback engine: {}", e));
            }
        }
    }
    
    // Load the sample
    if let Some(playback) = playback_guard.as_ref() {
        playback.load_sample(&validated_path.to_string_lossy())
            .map_err(|e| format!("Failed to load sample: {}", e))
    } else {
        Err("Playback engine not initialized".to_string())
    }
}

/// Start audio playback
#[tauri::command]
async fn start_playback() -> Result<String, String> {
    println!("▶️ GUI: Starting playback");
    
    let playback_guard = AUDIO_PLAYBACK.lock().unwrap();
    if let Some(playback) = playback_guard.as_ref() {
        playback.start_playback()
            .map_err(|e| format!("Failed to start playback: {}", e))
    } else {
        Err("Playback engine not initialized".to_string())
    }
}

/// Stop audio playback
#[tauri::command]
async fn stop_playback() -> Result<String, String> {
    println!("⏹️ GUI: Stopping playback");
    
    let playback_guard = AUDIO_PLAYBACK.lock().unwrap();
    if let Some(playback) = playback_guard.as_ref() {
        playback.stop_playback()
            .map_err(|e| format!("Failed to stop playback: {}", e))
    } else {
        Err("Playback engine not initialized".to_string())
    }
}

/// Pause audio playback
#[tauri::command]
async fn pause_playback() -> Result<String, String> {
    println!("⏸️ GUI: Pausing playback");
    
    let playback_guard = AUDIO_PLAYBACK.lock().unwrap();
    if let Some(playback) = playback_guard.as_ref() {
        playback.pause_playback()
            .map_err(|e| format!("Failed to pause playback: {}", e))
    } else {
        Err("Playback engine not initialized".to_string())
    }
}

/// Seek to position in audio (0.0 to 1.0)
#[tauri::command]
async fn seek_playback(position: f64) -> Result<String, String> {
    println!("⏩ GUI: Seeking to position: {:.1}%", position * 100.0);
    
    let playback_guard = AUDIO_PLAYBACK.lock().unwrap();
    if let Some(playback) = playback_guard.as_ref() {
        playback.seek_to_position(position)
            .map_err(|e| format!("Failed to seek: {}", e))
    } else {
        Err("Playback engine not initialized".to_string())
    }
}

/// Get current playback position (0.0 to 1.0)
#[tauri::command]
async fn get_playback_position() -> Result<f64, String> {
    let playback_guard = AUDIO_PLAYBACK.lock().unwrap();
    if let Some(playback) = playback_guard.as_ref() {
        Ok(playback.get_playback_position())
    } else {
        Ok(0.0)
    }
}

/// Check if audio is playing
#[tauri::command]
async fn is_playing() -> Result<bool, String> {
    let playback_guard = AUDIO_PLAYBACK.lock().unwrap();
    if let Some(playback) = playback_guard.as_ref() {
        Ok(playback.is_playing())
    } else {
        Ok(false)
    }
}

/// Get audio device channel information
#[tauri::command]
fn get_audio_device_info(device_index: usize) -> Result<AudioDeviceInfo, String> {
    println!("🎤 Getting info for audio device index: {}", device_index);
    
    use cpal::traits::{DeviceTrait, HostTrait};
    
    let host = cpal::default_host();
    let devices: Vec<_> = host.input_devices()
        .map_err(|e| format!("Failed to enumerate input devices: {}", e))?
        .collect();
    
    let device = devices.get(device_index)
        .ok_or_else(|| format!("Device index {} not found", device_index))?;
    
    let device_name = device.name()
        .unwrap_or_else(|_| "Unknown".to_string());
    
    // Get default input config to determine channels
    let config = device.default_input_config()
        .map_err(|e| format!("Failed to get device config: {}", e))?;
    
    let total_channels = config.channels();
    let sample_rate = config.sample_rate().0;
    
    // Generate channel names
    let channel_names: Vec<String> = (1..=total_channels)
        .map(|ch| format!("Input {}", ch))
        .collect();
    
    println!("   📊 Device: {}, Channels: {}, Sample Rate: {} Hz", 
             device_name, total_channels, sample_rate);
    
    Ok(AudioDeviceInfo {
        device_name,
        total_channels,
        sample_rate,
        channel_names,
    })
}

/// Initialize professional session management
#[tauri::command]
fn initialize_session_manager() -> Result<String, String> {
    println!("🎛️ Initializing professional session manager");
    
    let mut manager_guard = SESSION_MANAGER.lock().unwrap();
    if manager_guard.is_some() {
        return Ok("Session manager already initialized".to_string());
    }
    
    match SessionManager::new() {
        Ok(mut manager) => {
            // Update available devices in the validator
            let audio_inputs = list_audio_input_devices_sync().unwrap_or_default();
            let audio_outputs = list_audio_output_devices_sync().unwrap_or_default(); 
            let midi_devices = list_midi_devices_sync().unwrap_or_default();
            
            manager.config_validator.update_available_devices(
                audio_inputs,
                audio_outputs, 
                midi_devices
            );
            
            *manager_guard = Some(manager);
            println!("✅ Session manager initialized successfully");
            Ok("Session manager initialized".to_string())
        }
        Err(e) => {
            println!("❌ Failed to initialize session manager: {}", e);
            Err(format!("Failed to initialize session manager: {}", e))
        }
    }
}

/// Validate session configuration before initialization
#[tauri::command]
fn validate_session_config(config: SessionConfig) -> Result<ValidationReport, String> {
    println!("🔍 Validating session configuration: {}", config.project_name);
    
    let manager_guard = SESSION_MANAGER.lock().unwrap();
    let manager = manager_guard.as_ref()
        .ok_or_else(|| "Session manager not initialized".to_string())?;
    
    match manager.config_validator.validate_session_config(&config) {
        Ok(report) => {
            println!("📋 Validation complete: {} errors, {} warnings", report.errors.len(), report.warnings.len());
            Ok(report)
        }
        Err(e) => {
            println!("❌ Validation failed: {}", e);
            Err(format!("Validation failed: {}", e))
        }
    }
}

/// Test device connectivity before session initialization
#[tauri::command]
fn test_device_connectivity(config: SessionConfig) -> Result<DeviceTestResult, String> {
    println!("🔍 Testing device connectivity for session: {}", config.project_name);
    
    let manager_guard = SESSION_MANAGER.lock().unwrap();
    let manager = manager_guard.as_ref()
        .ok_or_else(|| "Session manager not initialized".to_string())?;
    
    match manager.test_device_connectivity(&config) {
        Ok(test_result) => {
            println!("🧪 Device test complete: overall success = {}", test_result.overall_success);
            Ok(test_result)
        }
        Err(e) => {
            println!("❌ Device test failed: {}", e);
            Err(format!("Device test failed: {}", e))
        }
    }
}

/// Initialize a new professional session
#[tauri::command]
fn initialize_session(config: SessionConfig) -> Result<String, String> {
    println!("🎛️ Initializing professional session: {}", config.project_name);
    
    let mut manager_guard = SESSION_MANAGER.lock().unwrap();
    let manager = manager_guard.as_mut()
        .ok_or_else(|| "Session manager not initialized".to_string())?;
    
    match manager.initialize_session(config.clone()) {
        Ok(_) => {
            println!("✅ Session '{}' initialized successfully", config.project_name);
            Ok(format!("Session '{}' ready for recording", config.project_name))
        }
        Err(e) => {
            println!("❌ Session initialization failed: {}", e);
            Err(format!("Session initialization failed: {}", e))
        }
    }
}

/// Get current session state
#[tauri::command]
fn get_session_state() -> Result<String, String> {
    let manager_guard = SESSION_MANAGER.lock().unwrap();
    if let Some(manager) = manager_guard.as_ref() {
        let state = manager.get_session_state();
        Ok(format!("{:?}", state))
    } else {
        Ok("Uninitialized".to_string())
    }
}

/// Check if session is ready for recording
#[tauri::command]
fn can_record() -> Result<bool, String> {
    let manager_guard = SESSION_MANAGER.lock().unwrap();
    if let Some(manager) = manager_guard.as_ref() {
        match manager.validate_recording_state() {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    } else {
        Ok(false)
    }
}

/// Get default session configuration
#[tauri::command]
fn get_default_session_config() -> Result<SessionConfig, String> {
    use batcherbird_core::session::*;
    use std::time::SystemTime;
    
    let default_config = SessionConfig {
        project_name: format!("New Project {}", chrono::Utc::now().format("%Y-%m-%d %H-%M")),
        project_directory: dirs::document_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")))
            .join("BatcherBird Projects"),
        audio: AudioSessionConfig::default(),
        midi: MidiSessionConfig::default(),
        recording: RecordingSessionConfig::default(),
        export: ExportSessionConfig::default(),
        created_at: SystemTime::now(),
    };
    
    Ok(default_config)
}

/// Save session configuration as template
#[tauri::command]
fn save_session_template(name: String, config: SessionConfig) -> Result<String, String> {
    println!("💾 Saving session template: {}", name);
    
    let mut manager_guard = SESSION_MANAGER.lock().unwrap();
    let manager = manager_guard.as_mut()
        .ok_or_else(|| "Session manager not initialized".to_string())?;
    
    match manager.save_session_template(name.clone(), config) {
        Ok(_) => Ok(format!("Template '{}' saved successfully", name)),
        Err(e) => Err(format!("Failed to save template: {}", e)),
    }
}

/// Load session template
#[tauri::command]
fn load_session_template(name: String) -> Result<SessionConfig, String> {
    println!("📂 Loading session template: {}", name);
    
    let manager_guard = SESSION_MANAGER.lock().unwrap();
    let manager = manager_guard.as_ref()
        .ok_or_else(|| "Session manager not initialized".to_string())?;
    
    match manager.load_session_template(&name) {
        Some(config) => Ok(config.clone()),
        None => Err(format!("Template '{}' not found", name)),
    }
}

/// List available session templates
#[tauri::command]
fn list_session_templates() -> Result<Vec<String>, String> {
    let manager_guard = SESSION_MANAGER.lock().unwrap();
    let manager = manager_guard.as_ref()
        .ok_or_else(|| "Session manager not initialized".to_string())?;
    
    Ok(manager.list_session_templates())
}

// Helper functions for device listing (synchronous versions)
fn list_audio_input_devices_sync() -> Result<Vec<String>, String> {
    let audio_manager = AudioManager::new().map_err(|e| e.to_string())?;
    audio_manager.list_input_devices().map_err(|e| e.to_string())
}

fn list_audio_output_devices_sync() -> Result<Vec<String>, String> {
    let audio_manager = AudioManager::new().map_err(|e| e.to_string())?;
    audio_manager.list_output_devices().map_err(|e| e.to_string())
}

fn list_midi_devices_sync() -> Result<Vec<String>, String> {
    let mut midi_manager = MidiManager::new().map_err(|e| e.to_string())?;
    midi_manager.list_output_devices().map_err(|e| e.to_string())
}

/// Get available synthesizer profiles for intelligent detection
#[tauri::command]
fn get_synthesizer_profiles() -> Result<Vec<String>, String> {
    Ok(vec![
        "General".to_string(),
        "Leads".to_string(),
        "Pads".to_string(),
        "Percussive".to_string(),
        "Ambient".to_string(),
    ])
}

/// Get intelligent detection configuration for a specific profile
#[tauri::command]
fn get_detection_config(profile: String) -> Result<String, String> {
    let config = match profile.as_str() {
        "General" => IntelligentDetectionConfig::default(),
        "Leads" => IntelligentDetectionConfig::for_leads(),
        "Pads" => IntelligentDetectionConfig::for_pads(),
        "Percussive" => IntelligentDetectionConfig::for_percussive(),
        "Ambient" => IntelligentDetectionConfig::for_ambient(),
        _ => return Err(format!("Unknown profile: {}", profile)),
    };
    serde_json::to_string(&config).map_err(|e| format!("Failed to serialize config: {}", e))
}

/// Perform intelligent detection on an audio file
#[tauri::command]
async fn detect_sample_boundaries(
    file_path: String,
    profile: String,
    custom_config: Option<String>,
) -> Result<String, String> {
    println!("🔍 Performing intelligent detection on: {}", file_path);
    println!("   Profile: {}", profile);
    
    // Validate file path
    let validated_path = validate_file_path(&file_path)?;
    
    // Create detection config
    let config = if let Some(custom_config_str) = custom_config {
        serde_json::from_str::<IntelligentDetectionConfig>(&custom_config_str)
            .map_err(|e| format!("Failed to parse custom config: {}", e))?
    } else {
        match profile.as_str() {
            "General" => IntelligentDetectionConfig::default(),
            "Leads" => IntelligentDetectionConfig::for_leads(),
            "Pads" => IntelligentDetectionConfig::for_pads(),
            "Percussive" => IntelligentDetectionConfig::for_percussive(),
            "Ambient" => IntelligentDetectionConfig::for_ambient(),
            _ => return Err(format!("Unknown profile: {}", profile)),
        }
    };
    
    // Create detector
    let mut detector = IntelligentSampleDetector::new(config);
    
    // Load audio file
    let mut reader = hound::WavReader::open(&validated_path)
        .map_err(|e| format!("Failed to open audio file: {}", e))?;
    
    let spec = reader.spec();
    let samples: Result<Vec<f32>, _> = reader.samples::<i16>()
        .map(|s| s.map(|sample| sample as f32 / i16::MAX as f32))
        .collect();
    
    let audio_data = samples.map_err(|e| format!("Failed to read audio data: {}", e))?;
    
    // Perform detection
    let result = detector.detect_intelligent_boundaries(&audio_data, spec.sample_rate)
        .map_err(|e| format!("Detection failed: {}", e))?;
    
    // Serialize result
    serde_json::to_string(&result).map_err(|e| format!("Failed to serialize result: {}", e))
}

/// Apply professional trimming to an audio file
#[tauri::command]
async fn apply_professional_trimming(
    file_path: String,
    detection_result: String,
    output_path: Option<String>,
) -> Result<String, String> {
    println!("✂️ Applying professional trimming to: {}", file_path);
    
    // Validate file path
    let validated_path = validate_file_path(&file_path)?;
    
    // Parse detection result
    let detection: IntelligentDetectionResult = serde_json::from_str(&detection_result)
        .map_err(|e| format!("Failed to parse detection result: {}", e))?;
    
    // Determine output path
    let output_file = if let Some(out_path) = output_path {
        validate_file_path(&out_path)?
    } else {
        // Create output path with "_trimmed" suffix
        let input_path = std::path::Path::new(&validated_path);
        let stem = input_path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid file name")?;
        let ext = input_path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("wav");
        let parent = input_path.parent()
            .ok_or("Invalid file path")?;
        
        parent.join(format!("{}_trimmed.{}", stem, ext))
    };
    
    // Load audio file
    let mut reader = hound::WavReader::open(&validated_path)
        .map_err(|e| format!("Failed to open audio file: {}", e))?;
    
    let spec = reader.spec();
    let samples: Result<Vec<f32>, _> = reader.samples::<i16>()
        .map(|s| s.map(|sample| sample as f32 / i16::MAX as f32))
        .collect();
    
    let audio_data = samples.map_err(|e| format!("Failed to read audio data: {}", e))?;
    
    // Apply trimming
    let trimmer = ProfessionalTrimmer::default();
    let trimming_result = trimmer.trim_audio(&audio_data, &detection, spec.sample_rate)
        .map_err(|e| format!("Trimming failed: {}", e))?;
    
    // Write trimmed audio
    let output_spec = hound::WavSpec {
        channels: spec.channels,
        sample_rate: spec.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = hound::WavWriter::create(&output_file, output_spec)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    for &sample in &trimming_result.audio_data {
        let sample_i16 = (sample * i16::MAX as f32) as i16;
        writer.write_sample(sample_i16)
            .map_err(|e| format!("Failed to write sample: {}", e))?;
    }
    
    writer.finalize()
        .map_err(|e| format!("Failed to finalize output file: {}", e))?;
    
    println!("✅ Trimmed audio saved to: {}", output_file.display());
    Ok(output_file.to_string_lossy().to_string())
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
      start_recording_with_viz,
      test_viz_throughput,
      record_range,
      record_range_with_velocity_layers,
      cancel_recording,
      save_recording_session,
      update_session_progress,
      get_recoverable_sessions,
      resume_recording_session,
      delete_recording_session,
      generate_instrument_files,
      create_directory,
      select_output_directory,
      select_audio_file,
      show_samples_in_finder,
      send_midi_panic,
      start_input_monitoring,
      start_input_monitoring_with_playthrough,
      stop_input_monitoring,
      get_midi_connection_status,
      get_audio_levels,
      start_realtime_meter_stream,
      get_professional_meter_readings,
      get_gain_staging_analysis,
      detect_loop_points,
      get_last_recorded_sample_path,
      apply_loop_metadata,
      get_waveform_data,
      load_sample_for_playback,
      start_playback,
      stop_playback,
      pause_playback,
      seek_playback,
      get_playback_position,
      is_playing,
      get_audio_device_info,
      // Professional session management commands
      initialize_session_manager,
      validate_session_config,
      test_device_connectivity,
      initialize_session,
      get_session_state,
      can_record,
      get_default_session_config,
      save_session_template,
      load_session_template,
      list_session_templates,
      // Intelligent detection commands
      get_synthesizer_profiles,
      get_detection_config,
      detect_sample_boundaries,
      apply_professional_trimming
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
