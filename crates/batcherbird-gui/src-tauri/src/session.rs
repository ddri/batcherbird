// Session recovery for Epic 4: Advanced Sampling Features
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSession {
    pub session_id: String,
    pub start_note: u8,
    pub end_note: u8,
    pub velocity_layers: Vec<u8>,
    pub duration: u32,
    pub output_directory: String,
    pub sample_name: String,
    pub export_format: String,
    pub creator_name: Option<String>,
    pub instrument_description: Option<String>,
    pub note_to_note_delay: u32,
    pub layer_to_layer_delay: u32,
    pub completed_recordings: Vec<CompletedRecording>,
    pub last_updated: std::time::SystemTime,
    pub total_recordings: usize,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedRecording {
    pub note: u8,
    pub velocity: u8,
    pub file_path: String,
    pub recorded_at: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    InProgress,
    Interrupted,
    Completed,
    Cancelled,
}

impl RecordingSession {
    pub fn new(
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
    ) -> Self {
        let total_notes = (end_note - start_note + 1) as usize;
        let total_recordings = total_notes * velocity_layers.len();
        
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
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
            completed_recordings: Vec::new(),
            last_updated: std::time::SystemTime::now(),
            total_recordings,
            status: SessionStatus::InProgress,
        }
    }
    
    pub fn add_completed_recording(&mut self, note: u8, velocity: u8, file_path: String) {
        self.completed_recordings.push(CompletedRecording {
            note,
            velocity,
            file_path,
            recorded_at: std::time::SystemTime::now(),
        });
        self.last_updated = std::time::SystemTime::now();
    }
    
    pub fn get_next_recording(&self) -> Option<(u8, u8)> {
        // Find the next note and velocity to record
        for note in self.start_note..=self.end_note {
            for velocity in &self.velocity_layers {
                let is_completed = self.completed_recordings.iter()
                    .any(|rec| rec.note == note && rec.velocity == *velocity);
                    
                if !is_completed {
                    return Some((note, *velocity));
                }
            }
        }
        None
    }
    
    pub fn get_progress(&self) -> f32 {
        (self.completed_recordings.len() as f32 / self.total_recordings as f32) * 100.0
    }
    
    pub fn is_complete(&self) -> bool {
        self.completed_recordings.len() == self.total_recordings
    }
    
    pub fn save_to_file(&self) -> Result<(), String> {
        let session_path = get_session_path(&self.session_id)?;
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;
        fs::write(session_path, json)
            .map_err(|e| format!("Failed to write session file: {}", e))?;
        Ok(())
    }
    
    pub fn load_from_file(session_id: &str) -> Result<Self, String> {
        let session_path = get_session_path(session_id)?;
        let json = fs::read_to_string(session_path)
            .map_err(|e| format!("Failed to read session file: {}", e))?;
        let session: Self = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize session: {}", e))?;
        Ok(session)
    }
    
    pub fn delete_session_file(&self) -> Result<(), String> {
        let session_path = get_session_path(&self.session_id)?;
        if session_path.exists() {
            fs::remove_file(session_path)
                .map_err(|e| format!("Failed to delete session file: {}", e))?;
        }
        Ok(())
    }
}

fn get_session_dir() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| "Could not find home directory".to_string())?;
    let session_dir = home_dir.join(".batcherbird").join("sessions");
    
    if !session_dir.exists() {
        fs::create_dir_all(&session_dir)
            .map_err(|e| format!("Failed to create session directory: {}", e))?;
    }
    
    Ok(session_dir)
}

fn get_session_path(session_id: &str) -> Result<PathBuf, String> {
    let session_dir = get_session_dir()?;
    Ok(session_dir.join(format!("{}.json", session_id)))
}

pub fn list_sessions() -> Result<Vec<SessionInfo>, String> {
    let session_dir = get_session_dir()?;
    let mut sessions = Vec::new();
    
    if let Ok(entries) = fs::read_dir(session_dir) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".json") {
                    let session_id = file_name.trim_end_matches(".json");
                    if let Ok(session) = RecordingSession::load_from_file(session_id) {
                        sessions.push(SessionInfo {
                            session_id: session.session_id.clone(),
                            sample_name: session.sample_name.clone(),
                            progress: session.get_progress(),
                            status: session.status.clone(),
                            last_updated: session.last_updated,
                        });
                    }
                }
            }
        }
    }
    
    // Sort by last updated, most recent first
    sessions.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));
    
    Ok(sessions)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub sample_name: String,
    pub progress: f32,
    pub status: SessionStatus,
    pub last_updated: std::time::SystemTime,
}