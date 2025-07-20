use batcherbird_core::{midi::MidiManager, audio::AudioManager};
use midir::MidiOutputConnection;
use std::sync::Mutex;
use std::time::Duration;

// Global state for device managers
static MIDI_MANAGER: Mutex<Option<MidiManager>> = Mutex::new(None);
static MIDI_CONNECTION: Mutex<Option<MidiOutputConnection>> = Mutex::new(None);
static AUDIO_MANAGER: Mutex<Option<AudioManager>> = Mutex::new(None);

#[tauri::command]
async fn list_midi_devices() -> Result<Vec<String>, String> {
    let mut midi_manager = MidiManager::new().map_err(|e| e.to_string())?;
    let devices = midi_manager.list_output_devices().map_err(|e| e.to_string())?;
    
    // Store the manager for later use
    *MIDI_MANAGER.lock().unwrap() = Some(midi_manager);
    
    Ok(devices)
}

#[tauri::command]
async fn list_audio_input_devices() -> Result<Vec<String>, String> {
    let audio_manager = AudioManager::new().map_err(|e| e.to_string())?;
    let devices = audio_manager.list_input_devices().map_err(|e| e.to_string())?;
    
    // Store the manager for later use
    *AUDIO_MANAGER.lock().unwrap() = Some(audio_manager);
    
    Ok(devices)
}

#[tauri::command]
async fn list_audio_output_devices() -> Result<Vec<String>, String> {
    let audio_manager = AudioManager::new().map_err(|e| e.to_string())?;
    audio_manager.list_output_devices().map_err(|e| e.to_string())
}

#[tauri::command]
async fn connect_midi_device(device_index: usize) -> Result<String, String> {
    let mut manager_guard = MIDI_MANAGER.lock().unwrap();
    if let Some(ref mut midi_manager) = manager_guard.as_mut() {
        let connection = midi_manager.connect_output(device_index).map_err(|e| e.to_string())?;
        *MIDI_CONNECTION.lock().unwrap() = Some(connection);
        Ok(format!("Connected to MIDI device {}", device_index))
    } else {
        Err("MIDI manager not initialized. List devices first.".to_string())
    }
}

#[tauri::command]
async fn test_midi_connection() -> Result<String, String> {
    // Extract the connection from the mutex and drop the guard
    let mut connection = {
        let mut connection_guard = MIDI_CONNECTION.lock().unwrap();
        match connection_guard.take() {
            Some(conn) => conn,
            None => return Err("No MIDI connection established".to_string()),
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      list_midi_devices, 
      list_audio_input_devices,
      list_audio_output_devices,
      connect_midi_device, 
      test_midi_connection
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
