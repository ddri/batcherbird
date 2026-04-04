use crate::app_event::AppEvent;
use batcherbird_core::export::AudioFormat;
use batcherbird_core::lock_free_recording::RealtimeMeterData;
use batcherbird_core::sampler::{SamplingConfig, SamplingEngine, VizChunk};
use rtrb::Consumer;
use std::path::PathBuf;
use vizia::prelude::*;

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
    pub error_message: Option<String>,

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

    // Engine handles
    #[lens(ignore)]
    pub meter_consumer: Option<Consumer<RealtimeMeterData>>,
    #[lens(ignore)]
    pub sampling_engine: Option<SamplingEngine>,
    #[lens(ignore)]
    pub monitoring_stream: Option<cpal::Stream>,

    // Waveform data
    #[lens(ignore)]
    pub viz_chunks: Vec<VizChunk>,
    /// Peak values (0.0-1.0) extracted from viz_chunks for waveform display.
    /// Updated alongside viz_chunks. This field is lensable.
    pub viz_peaks: Vec<f32>,
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

            start_note: 36, // C2
            end_note: 84,   // C6
            velocity_layers: 1,
            note_duration_ms: 2000,

            export_format: AudioFormat::Wav24Bit,
            output_directory: dirs::document_dir().unwrap_or_else(|| PathBuf::from(".")),

            app_state: AppState::Idle,
            error_message: None,

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

            meter_consumer: None,
            sampling_engine: None,
            monitoring_stream: None,

            viz_chunks: Vec::new(),
            viz_peaks: Vec::new(),
        }
    }
}

impl AppData {
    fn build_sampling_config(&self) -> SamplingConfig {
        SamplingConfig {
            note_duration_ms: self.note_duration_ms as u64,
            release_time_ms: 1000,
            pre_delay_ms: 100,
            post_delay_ms: 100,
            midi_channel: 0,
            velocity: 100,
        }
    }

    pub fn total_samples(&self) -> u32 {
        let num_notes = (self.end_note as u32).saturating_sub(self.start_note as u32) + 1;
        num_notes * self.velocity_layers as u32
    }

    pub fn note_name(note: u8) -> String {
        let names = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
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
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event: &AppEvent, _| match app_event {
            AppEvent::RefreshDevices => {
                if let Ok(mut manager) = batcherbird_core::midi::MidiManager::new() {
                    if let Ok(devices) = manager.list_output_devices() {
                        self.midi_devices = devices;
                    }
                }
                if let Ok(manager) = batcherbird_core::audio::AudioManager::new() {
                    if let Ok(devices) = manager.list_input_devices() {
                        self.audio_input_devices = devices;
                    }
                }
            }
            AppEvent::SelectMidiDevice(idx) => {
                self.selected_midi_device = Some(*idx);
            }
            AppEvent::SelectAudioInput(idx) => {
                self.selected_audio_input = Some(*idx);
            }
            AppEvent::Tick => {
                if let Some(consumer) = &mut self.meter_consumer {
                    let mut latest: Option<RealtimeMeterData> = None;
                    while let Ok(data) = consumer.pop() {
                        latest = Some(data);
                    }
                    if let Some(data) = latest {
                        self.meter_left = data.peak_left;
                        self.meter_right = data.peak_right;
                        self.meter_left_db = if data.peak_left > 0.0 {
                            20.0 * data.peak_left.log10()
                        } else {
                            -60.0
                        };
                        self.meter_right_db = if data.peak_right > 0.0 {
                            20.0 * data.peak_right.log10()
                        } else {
                            -60.0
                        };
                        self.is_clipping = data.is_clipping;
                    }
                }
                // Fallback: poll engine levels when monitoring (no ring buffer consumer)
                if self.meter_consumer.is_none() {
                    if let Some(engine) = &self.sampling_engine {
                        let levels = engine.get_audio_levels();
                        self.meter_left = levels.peak;
                        self.meter_right = levels.peak; // mono for now
                        self.meter_left_db = levels.peak_db;
                        self.meter_right_db = levels.peak_db;
                    }
                }
            }
            AppEvent::SetStartNote(n) => self.start_note = *n,
            AppEvent::SetEndNote(n) => self.end_note = *n,
            AppEvent::SetVelocityLayers(n) => self.velocity_layers = *n,
            AppEvent::SetDuration(ms) => self.note_duration_ms = *ms,
            AppEvent::SetExportFormat(fmt) => self.export_format = fmt.clone(),
            AppEvent::SetOutputDirectory(path) => self.output_directory = path.clone(),

            AppEvent::Arm => {
                if self.app_state == AppState::Idle {
                    let config = self.build_sampling_config();
                    match SamplingEngine::new(config) {
                        Ok(engine) => match engine.start_monitoring_stream() {
                            Ok(stream) => {
                                self.monitoring_stream = Some(stream);
                                self.sampling_engine = Some(engine);
                                self.app_state = AppState::Armed;
                            }
                            Err(e) => {
                            self.error_message = Some(format!("Failed to start monitoring: {}", e));
                        }
                        },
                        Err(e) => {
                            self.error_message = Some(format!("Failed to create engine: {}", e));
                        }
                    }
                }
            }
            AppEvent::Disarm => {
                if self.app_state == AppState::Armed || self.app_state == AppState::Review {
                    self.monitoring_stream = None;
                    self.sampling_engine = None;
                    self.app_state = AppState::Idle;
                }
            }
            AppEvent::StartRecording => {
                if self.app_state == AppState::Armed {
                    self.app_state = AppState::Recording;
                    self.notes_total = self.total_samples();
                    self.notes_completed = 0;
                    self.viz_chunks.clear();
                    self.viz_peaks.clear();

                    // Stop monitoring before recording
                    self.monitoring_stream = None;
                    self.sampling_engine = None;

                    let config = self.build_sampling_config();

                    let start_note = self.start_note;
                    let end_note = self.end_note;
                    let midi_device_idx = self.selected_midi_device.unwrap_or(0);
                    let mut proxy = cx.get_proxy();

                    std::thread::spawn(move || {
                        // Create MIDI connection in the recording thread
                        let midi_conn_result =
                            (|| -> std::result::Result<midir::MidiOutputConnection, String> {
                                let mut midi_mgr = batcherbird_core::midi::MidiManager::new()
                                    .map_err(|e| format!("Failed to create MIDI manager: {}", e))?;
                                midi_mgr
                                    .connect_output(midi_device_idx)
                                    .map_err(|e| format!("Failed to connect MIDI output: {}", e))
                            })();

                        let mut midi_conn = match midi_conn_result {
                            Ok(conn) => conn,
                            Err(e) => {
                                let _ = proxy.emit(AppEvent::RecordingError(e));
                                return;
                            }
                        };

                        match SamplingEngine::new(config) {
                            Ok(engine) => {
                                if start_note == end_note {
                                    match engine
                                        .sample_single_note_blocking(&mut midi_conn, start_note)
                                    {
                                        Ok(_) => {
                                            let _ = proxy.emit(AppEvent::RecordingComplete);
                                        }
                                        Err(e) => {
                                            let _ =
                                                proxy.emit(AppEvent::RecordingError(e.to_string()));
                                        }
                                    }
                                } else {
                                    match engine.sample_note_range_blocking(
                                        &mut midi_conn,
                                        start_note,
                                        end_note,
                                    ) {
                                        Ok(_) => {
                                            let _ = proxy.emit(AppEvent::RecordingComplete);
                                        }
                                        Err(e) => {
                                            let _ =
                                                proxy.emit(AppEvent::RecordingError(e.to_string()));
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = proxy.emit(AppEvent::RecordingError(e.to_string()));
                            }
                        }
                    });
                }
            }
            AppEvent::PushVizChunk(chunk) => {
                self.viz_peaks.push(chunk.peak);
                self.viz_chunks.push(chunk.clone());
            }
            AppEvent::CancelRecording => {
                self.app_state = AppState::Idle;
            }
            AppEvent::RecordingProgress {
                note,
                velocity,
                layer,
                completed,
                total,
            } => {
                self.current_note = *note;
                self.current_velocity = *velocity;
                self.current_layer = *layer;
                self.notes_completed = *completed;
                self.notes_total = *total;
            }
            AppEvent::RecordingComplete => {
                self.app_state = AppState::Review;
            }
            AppEvent::RecordingError(msg) => {
                self.app_state = AppState::Idle;
                self.error_message = Some(msg.clone());
            }
            AppEvent::DismissError => {
                self.error_message = None;
            }

            AppEvent::SelectOutputDirectory => {
                let current_dir = self.output_directory.clone();
                let mut proxy = cx.get_proxy();

                std::thread::spawn(move || {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_directory(&current_dir)
                        .pick_folder()
                    {
                        let _ = proxy.emit(AppEvent::SetOutputDirectory(path));
                    }
                });
            }

            _ => {}
        });
    }
}
