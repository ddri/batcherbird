use std::path::PathBuf;
use vizia::prelude::*;
use batcherbird_core::export::AudioFormat;
use batcherbird_core::sampler::VizChunk;
use crate::app_event::AppEvent;

#[derive(Debug, Clone, PartialEq, Data)]
pub enum AppState {
    Idle,
    Armed,
    Recording,
    Review,
}

#[derive(Lens)]
pub struct AppData {
    // Devices
    pub midi_devices: Vec<String>,
    pub audio_input_devices: Vec<String>,
    pub selected_midi_device: Option<usize>,
    pub selected_audio_input: Option<usize>,
    pub midi_connected: bool,
    pub audio_connected: bool,

    // Sampling config
    pub start_note: u8,
    pub end_note: u8,
    pub velocity_layers: u8,
    pub note_duration_ms: u32,

    // Export config
    #[lens(ignore)]
    pub export_format: AudioFormat,
    pub output_directory: PathBuf,

    // App state
    pub app_state: AppState,

    // Real-time meters
    pub meter_left: f32,
    pub meter_right: f32,
    pub meter_left_db: f32,
    pub meter_right_db: f32,
    pub is_clipping: bool,

    // Recording progress
    pub current_note: u8,
    pub current_velocity: u8,
    pub current_layer: u8,
    pub total_layers: u8,
    pub notes_completed: u32,
    pub notes_total: u32,

    // Waveform data
    #[lens(ignore)]
    pub viz_chunks: Vec<VizChunk>,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            midi_devices: Vec::new(),
            audio_input_devices: Vec::new(),
            selected_midi_device: None,
            selected_audio_input: None,
            midi_connected: false,
            audio_connected: false,

            start_note: 36,       // C2
            end_note: 84,         // C6
            velocity_layers: 1,
            note_duration_ms: 2000,

            export_format: AudioFormat::Wav24Bit,
            output_directory: dirs::document_dir()
                .unwrap_or_else(|| PathBuf::from(".")),

            app_state: AppState::Idle,

            meter_left: 0.0,
            meter_right: 0.0,
            meter_left_db: -60.0,
            meter_right_db: -60.0,
            is_clipping: false,

            current_note: 0,
            current_velocity: 0,
            current_layer: 0,
            total_layers: 0,
            notes_completed: 0,
            notes_total: 0,

            viz_chunks: Vec::new(),
        }
    }
}

impl AppData {
    pub fn total_samples(&self) -> u32 {
        let num_notes = (self.end_note as u32)
            .saturating_sub(self.start_note as u32) + 1;
        num_notes * self.velocity_layers as u32
    }

    pub fn note_name(note: u8) -> String {
        let names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let octave = (note as i8 / 12) - 1;
        let index = (note % 12) as usize;
        format!("{}{}", names[index], octave)
    }

    pub fn estimated_duration_secs(&self) -> f32 {
        let total = self.total_samples() as f32;
        let per_note_secs = self.note_duration_ms as f32 / 1000.0 + 1.5;
        total * per_note_secs
    }
}

impl Model for AppData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event: &AppEvent, _| match app_event {
            AppEvent::SetStartNote(n) => self.start_note = *n,
            AppEvent::SetEndNote(n) => self.end_note = *n,
            AppEvent::SetVelocityLayers(n) => self.velocity_layers = *n,
            AppEvent::SetDuration(ms) => self.note_duration_ms = *ms,
            AppEvent::SetExportFormat(fmt) => self.export_format = fmt.clone(),
            AppEvent::SetOutputDirectory(path) => self.output_directory = path.clone(),

            AppEvent::Arm => {
                if self.app_state == AppState::Idle {
                    self.app_state = AppState::Armed;
                }
            }
            AppEvent::Disarm => {
                if self.app_state == AppState::Armed {
                    self.app_state = AppState::Idle;
                }
            }
            AppEvent::StartRecording => {
                if self.app_state == AppState::Armed {
                    self.app_state = AppState::Recording;
                    self.notes_total = self.total_samples();
                    self.notes_completed = 0;
                    self.viz_chunks.clear();
                }
            }
            AppEvent::CancelRecording => {
                self.app_state = AppState::Idle;
            }
            AppEvent::RecordingProgress { note, velocity, layer, completed, total } => {
                self.current_note = *note;
                self.current_velocity = *velocity;
                self.current_layer = *layer;
                self.notes_completed = *completed;
                self.notes_total = *total;
            }
            AppEvent::RecordingComplete => {
                self.app_state = AppState::Review;
            }
            AppEvent::RecordingError(_msg) => {
                self.app_state = AppState::Idle;
            }

            _ => {}
        });
    }
}
