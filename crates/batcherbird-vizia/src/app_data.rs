use crate::app_event::AppEvent;
use batcherbird_core::export::AudioFormat;
use batcherbird_core::lock_free_recording::RealtimeMeterData;
use batcherbird_core::preview_player::PreviewPlayer;
use batcherbird_core::export::{ExportConfig, SampleExporter};
use batcherbird_core::sampler::{Sample, SamplingConfig, SamplingEngine, VizChunk};
use rtrb::Consumer;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use vizia::prelude::*;

/// Downsample raw interleaved/mono audio into `buckets` peak values normalized
/// to 0.0..=1.0 by taking the max absolute amplitude within each bucket.
///
/// - Empty input (or `buckets == 0`) yields an empty vec.
/// - The returned vec has at most `buckets` entries (fewer if `audio` is shorter
///   than `buckets`).
/// - All values are clamped to `<= 1.0`.
pub fn samples_to_peaks(audio: &[f32], buckets: usize) -> Vec<f32> {
    if audio.is_empty() || buckets == 0 {
        return Vec::new();
    }
    let n = buckets.min(audio.len());
    let chunk = audio.len().div_ceil(n);
    let mut peaks = Vec::with_capacity(n);
    for chunk_slice in audio.chunks(chunk) {
        let peak = chunk_slice
            .iter()
            .fold(0.0f32, |acc, &s| acc.max(s.abs()))
            .min(1.0);
        peaks.push(peak);
    }
    peaks
}

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
    pub selected_midi_device: usize,
    pub selected_audio_input: usize,
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
    pub export_format_display: String,
    pub format_options: Vec<String>,
    pub selected_format_index: usize,
    pub output_directory: PathBuf,

    // App state
    pub app_state: AppState,
    pub error_message: Option<String>,
    /// Transient success / status banner (e.g. export complete).
    pub info_message: Option<String>,

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

    // Review state
    pub recorded_count: u32,
    pub is_playing: bool,
    pub playback_position: f64,
    /// Active one-shot preview player for the Review screen, if any. Holds the
    /// live cpal output stream; dropping it stops audio.
    #[lens(ignore)]
    pub preview_player: Option<PreviewPlayer>,

    // Recorded samples + worker hand-off
    /// Samples captured by the most recent finished recording.
    #[lens(ignore)]
    pub recorded_samples: Vec<Sample>,
    /// Hand-off slot: the recording worker writes its returned samples here so
    /// they don't have to travel by value through an `AppEvent` (which would
    /// deep-clone all the audio). The UI moves them out on `RecordingFinished`.
    #[lens(ignore)]
    pub recorded_slot: Arc<Mutex<Vec<Sample>>>,
    /// Cooperative cancellation flag for the active recording worker.
    #[lens(ignore)]
    pub cancel_flag: Option<Arc<AtomicBool>>,
    /// Bumped on every `StartRecording`. Worker→UI events carry the generation
    /// they were started under; the handler ignores any event whose generation
    /// no longer matches, which is what makes cancellation/restart correct.
    #[lens(ignore)]
    pub recording_generation: u64,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            midi_devices: Vec::new(),
            audio_input_devices: Vec::new(),
            selected_midi_device: 0,
            selected_audio_input: 0,
            midi_connected: false,
            audio_connected: false,

            start_note: 36, // C2
            end_note: 84,   // C6
            velocity_layers: 1,
            note_duration_ms: 2000,

            export_format: AudioFormat::Wav24Bit,
            export_format_display: "Wav24Bit".to_string(),
            format_options: vec![
                "WAV 16-bit".to_string(),
                "WAV 24-bit".to_string(),
                "WAV 32-float".to_string(),
                "DecentSampler".to_string(),
                "SFZ".to_string(),
            ],
            selected_format_index: 1, // Wav24Bit
            output_directory: dirs::document_dir().unwrap_or_else(|| PathBuf::from(".")),

            app_state: AppState::Idle,
            error_message: None,
            info_message: None,

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

            recorded_count: 0,
            is_playing: false,
            playback_position: 0.0,
            preview_player: None,

            recorded_samples: Vec::new(),
            recorded_slot: Arc::new(Mutex::new(Vec::new())),
            cancel_flag: None,
            recording_generation: 0,
        }
    }
}

impl AppData {
    fn build_sampling_config(&self) -> SamplingConfig {
        // Resolve the user's selected audio input device (if any) to a name so
        // recording uses the chosen device rather than the system default.
        let input_device_name = self
            .audio_input_devices
            .as_slice()
            .get(self.selected_audio_input)
            .cloned();

        SamplingConfig {
            note_duration_ms: self.note_duration_ms as u64,
            release_time_ms: 1000,
            pre_delay_ms: 100,
            post_delay_ms: 100,
            midi_channel: 0,
            velocity: 100,
            input_device_name,
        }
    }

    /// Whether `generation` matches the current recording session. Worker→UI
    /// events carry the generation they were spawned under; once the generation
    /// is bumped (new recording or cancel), the in-flight worker's events become
    /// stale and are ignored.
    pub fn is_current_recording(&self, generation: u64) -> bool {
        generation == self.recording_generation
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

    pub fn format_display(fmt: &AudioFormat) -> &'static str {
        match fmt {
            AudioFormat::Wav16Bit => "Wav16Bit",
            AudioFormat::Wav24Bit => "Wav24Bit",
            AudioFormat::Wav32BitFloat => "Wav32Float",
            AudioFormat::DecentSampler => "DecentSampler",
            AudioFormat::SFZ => "SFZ",
        }
    }

    pub fn next_format(fmt: &AudioFormat) -> AudioFormat {
        match fmt {
            AudioFormat::Wav16Bit => AudioFormat::Wav24Bit,
            AudioFormat::Wav24Bit => AudioFormat::Wav32BitFloat,
            AudioFormat::Wav32BitFloat => AudioFormat::DecentSampler,
            AudioFormat::DecentSampler => AudioFormat::SFZ,
            AudioFormat::SFZ => AudioFormat::Wav16Bit,
        }
    }

    pub fn prev_format(fmt: &AudioFormat) -> AudioFormat {
        match fmt {
            AudioFormat::Wav16Bit => AudioFormat::SFZ,
            AudioFormat::Wav24Bit => AudioFormat::Wav16Bit,
            AudioFormat::Wav32BitFloat => AudioFormat::Wav24Bit,
            AudioFormat::DecentSampler => AudioFormat::Wav32BitFloat,
            AudioFormat::SFZ => AudioFormat::DecentSampler,
        }
    }

    pub fn estimated_duration_secs(&self) -> f32 {
        let total = self.total_samples() as f32;
        let per_note_secs = self.note_duration_ms as f32 / 1000.0 + 1.5;
        total * per_note_secs
    }

    /// Start a one-shot preview of `recorded_samples[idx]`, replacing any
    /// currently-playing preview. On success the player is stored and
    /// `is_playing` is set; on failure `error_message` is set and `is_playing`
    /// is left false. Out-of-range indices are ignored.
    fn start_preview(&mut self, idx: usize) {
        // Drop any existing player first so its stream stops cleanly.
        self.stop_preview();

        let Some(sample) = self.recorded_samples.get(idx) else {
            return;
        };
        let audio: Arc<[f32]> = Arc::from(sample.audio_data.as_slice());
        match PreviewPlayer::play(audio, sample.sample_rate, sample.channels) {
            Ok(player) => {
                self.preview_player = Some(player);
                self.is_playing = true;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to play preview: {}", e));
                self.is_playing = false;
            }
        }
    }

    /// Stop and drop any active preview player, resetting playback UI state.
    fn stop_preview(&mut self) {
        if let Some(player) = self.preview_player.take() {
            player.stop();
        }
        self.is_playing = false;
        self.playback_position = 0.0;
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
                // Reflect one-shot preview completion in the UI: once the
                // player reaches the end (or there's no player), clear playing
                // state so the Play/Stop button updates.
                if self.is_playing
                    && self
                        .preview_player
                        .as_ref()
                        .is_none_or(|p| p.is_finished())
                {
                    self.preview_player = None;
                    self.is_playing = false;
                    self.playback_position = 0.0;
                }
            }
            AppEvent::SetStartNote(n) => self.start_note = *n,
            AppEvent::SetEndNote(n) => self.end_note = *n,
            AppEvent::SetVelocityLayers(n) => self.velocity_layers = *n,
            AppEvent::SetDuration(ms) => self.note_duration_ms = *ms,
            AppEvent::SetExportFormat(fmt) => {
                self.export_format_display = Self::format_display(fmt).to_string();
                self.export_format = fmt.clone();
            }
            AppEvent::SetOutputDirectory(path) => self.output_directory = path.clone(),

            AppEvent::CycleNextMidiDevice => {
                if !self.midi_devices.is_empty() {
                    self.selected_midi_device = (self.selected_midi_device + 1) % self.midi_devices.len();
                }
            }
            AppEvent::CycleNextAudioInput => {
                if !self.audio_input_devices.is_empty() {
                    self.selected_audio_input = (self.selected_audio_input + 1) % self.audio_input_devices.len();
                }
            }
            AppEvent::SelectMidiDevice(idx) => {
                self.selected_midi_device = *idx;
            }
            AppEvent::SelectAudioInput(idx) => {
                self.selected_audio_input = *idx;
            }
            AppEvent::CycleExportFormat => {
                let next = Self::next_format(&self.export_format);
                self.export_format_display = Self::format_display(&next).to_string();
                self.export_format = next;
            }
            AppEvent::SelectFormatByIndex(idx) => {
                let formats = [
                    AudioFormat::Wav16Bit,
                    AudioFormat::Wav24Bit,
                    AudioFormat::Wav32BitFloat,
                    AudioFormat::DecentSampler,
                    AudioFormat::SFZ,
                ];
                if *idx < formats.len() {
                    self.selected_format_index = *idx;
                    self.export_format = formats[*idx].clone();
                    self.export_format_display = Self::format_display(&formats[*idx]).to_string();
                }
            }
            AppEvent::CycleExportFormatBack => {
                let prev = Self::prev_format(&self.export_format);
                self.export_format_display = Self::format_display(&prev).to_string();
                self.export_format = prev;
            }
            AppEvent::CyclePrevMidiDevice => {
                if !self.midi_devices.is_empty() {
                    self.selected_midi_device = if self.selected_midi_device == 0 {
                        self.midi_devices.len() - 1
                    } else {
                        self.selected_midi_device - 1
                    };
                }
            }
            AppEvent::CyclePrevAudioInput => {
                if !self.audio_input_devices.is_empty() {
                    self.selected_audio_input = if self.selected_audio_input == 0 {
                        self.audio_input_devices.len() - 1
                    } else {
                        self.selected_audio_input - 1
                    };
                }
            }

            AppEvent::IncrementStartNote => {
                if self.start_note < 127 && self.start_note < self.end_note {
                    self.start_note += 1;
                }
            }
            AppEvent::DecrementStartNote => {
                if self.start_note > 0 {
                    self.start_note -= 1;
                }
            }
            AppEvent::IncrementEndNote => {
                if self.end_note < 127 {
                    self.end_note += 1;
                }
            }
            AppEvent::DecrementEndNote => {
                if self.end_note > 0 && self.end_note > self.start_note {
                    self.end_note -= 1;
                }
            }
            AppEvent::IncrementVelocityLayers => {
                if self.velocity_layers < 4 {
                    self.velocity_layers += 1;
                }
            }
            AppEvent::DecrementVelocityLayers => {
                if self.velocity_layers > 1 {
                    self.velocity_layers -= 1;
                }
            }
            AppEvent::IncrementDuration => {
                if self.note_duration_ms < 10000 {
                    self.note_duration_ms = (self.note_duration_ms + 500).min(10000);
                }
            }
            AppEvent::DecrementDuration => {
                if self.note_duration_ms > 500 {
                    self.note_duration_ms = self.note_duration_ms.saturating_sub(500).max(500);
                }
            }

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
                    self.stop_preview();
                    self.monitoring_stream = None;
                    self.sampling_engine = None;
                    self.app_state = AppState::Idle;
                }
            }
            AppEvent::StartRecording => {
                if self.app_state == AppState::Armed {
                    self.stop_preview();
                    self.app_state = AppState::Recording;
                    self.notes_total = self.total_samples();
                    self.notes_completed = 0;
                    self.viz_chunks.clear();
                    self.viz_peaks.clear();
                    self.error_message = None;
                    self.info_message = None;

                    // Stop monitoring before recording
                    self.monitoring_stream = None;
                    self.sampling_engine = None;

                    // New recording session: bump generation, create a fresh
                    // cancel flag, and clear the hand-off slot.
                    self.recording_generation += 1;
                    let generation = self.recording_generation;
                    let cancel = Arc::new(AtomicBool::new(false));
                    self.cancel_flag = Some(cancel.clone());
                    let recorded_slot = self.recorded_slot.clone();
                    if let Ok(mut slot) = recorded_slot.lock() {
                        slot.clear();
                    }

                    let config = self.build_sampling_config();

                    let start_note = self.start_note;
                    let end_note = self.end_note;
                    let velocity_layers = self.velocity_layers;
                    let midi_device_idx = self.selected_midi_device;
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
                            Err(message) => {
                                let _ = proxy
                                    .emit(AppEvent::RecordingError { generation, message });
                                return;
                            }
                        };

                        match SamplingEngine::new(config) {
                            Ok(engine) => {
                                let result = engine.sample_note_range_with_progress_blocking(
                                    &mut midi_conn,
                                    start_note,
                                    end_note,
                                    velocity_layers,
                                    &cancel,
                                    |p| {
                                        let _ = proxy.emit(AppEvent::RecordingProgress {
                                            generation,
                                            note: p.note,
                                            velocity: p.velocity,
                                            layer: p.layer,
                                            total_layers: p.total_layers,
                                            completed: p.completed,
                                            total: p.total,
                                        });
                                    },
                                );
                                match result {
                                    Ok(samples) => {
                                        if let Ok(mut slot) = recorded_slot.lock() {
                                            *slot = samples;
                                        }
                                        let _ = proxy
                                            .emit(AppEvent::RecordingFinished { generation });
                                    }
                                    Err(e) => {
                                        let _ = proxy.emit(AppEvent::RecordingError {
                                            generation,
                                            message: e.to_string(),
                                        });
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = proxy.emit(AppEvent::RecordingError {
                                    generation,
                                    message: e.to_string(),
                                });
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
                // Signal the worker to stop. Core's cooperative cancel returns
                // Ok(partial_samples), so the worker still emits
                // RecordingFinished{generation} under its original generation.
                // Bumping the generation here makes that generation stale, so
                // `is_current_recording` rejects the worker's eventual
                // RecordingFinished / RecordingError and the UI stays Idle.
                if let Some(f) = &self.cancel_flag {
                    f.store(true, Ordering::Relaxed);
                }
                self.recording_generation += 1;
                self.cancel_flag = None;
                self.app_state = AppState::Idle;
                self.current_note = 0;
                self.current_velocity = 0;
                self.current_layer = 0;
                self.total_layers = 0;
                self.notes_completed = 0;
            }
            AppEvent::RecordingProgress {
                generation,
                note,
                velocity,
                layer,
                total_layers,
                completed,
                total,
            } => {
                if !self.is_current_recording(*generation) {
                    return;
                }
                self.current_note = *note;
                self.current_velocity = *velocity;
                self.current_layer = *layer;
                self.total_layers = *total_layers;
                self.notes_completed = *completed;
                self.notes_total = *total;
            }
            AppEvent::RecordingFinished { generation } => {
                if !self.is_current_recording(*generation) {
                    return;
                }
                // Move the captured samples out of the hand-off slot.
                let samples = match self.recorded_slot.lock() {
                    Ok(mut slot) => std::mem::take(&mut *slot),
                    Err(_) => Vec::new(),
                };
                // Populate the Review waveform from the first recorded sample.
                self.viz_peaks = samples
                    .first()
                    .map(|s| samples_to_peaks(&s.audio_data, 512))
                    .unwrap_or_default();
                self.recorded_count = samples.len() as u32;
                self.recorded_samples = samples;
                self.cancel_flag = None;
                // Stop/clear any preview from a previous Review session.
                self.stop_preview();
                self.app_state = AppState::Review;
            }
            AppEvent::PlayPreview => {
                // The Review UI has no per-sample selection, so preview the
                // first recorded sample. If selection is added later, route it
                // through PlaySample(idx).
                if !self.recorded_samples.is_empty() {
                    self.start_preview(0);
                }
            }
            AppEvent::PlaySample(idx) => {
                self.start_preview(*idx);
            }
            AppEvent::StopPreview | AppEvent::StopPlayback => {
                self.stop_preview();
            }
            AppEvent::PausePreview => {
                // The one-shot player has no real pause/resume; treat Pause as
                // Stop rather than faking a resumable pause.
                self.stop_preview();
            }
            AppEvent::RecordingError { generation, message } => {
                if !self.is_current_recording(*generation) {
                    return;
                }
                self.app_state = AppState::Idle;
                self.error_message = Some(message.clone());
                self.info_message = None;
                self.cancel_flag = None;
            }
            AppEvent::ExportAll => {
                if self.recorded_samples.is_empty() {
                    self.error_message =
                        Some("No recorded samples to export.".to_string());
                    return;
                }
                self.error_message = None;
                self.info_message = None;

                let cfg = ExportConfig {
                    output_directory: self.output_directory.clone(),
                    sample_format: self.export_format.clone(),
                    ..Default::default()
                };
                // Cloning the samples once into the worker is acceptable; export
                // does file IO + detection and must not block the UI thread.
                let samples = self.recorded_samples.clone();
                let mut proxy = cx.get_proxy();

                std::thread::spawn(move || {
                    let result = SampleExporter::new(cfg.clone())
                        .and_then(|exporter| exporter.export_samples(&samples));
                    match result {
                        Ok(paths) => {
                            let _ = proxy.emit(AppEvent::ExportComplete {
                                count: paths.len(),
                                directory: cfg.output_directory,
                            });
                        }
                        Err(e) => {
                            let _ = proxy.emit(AppEvent::ExportError(e.to_string()));
                        }
                    }
                });
            }
            AppEvent::ExportComplete { count, directory } => {
                self.error_message = None;
                self.info_message = Some(format!(
                    "Exported {} sample(s) to {}",
                    count,
                    directory.display()
                ));
            }
            AppEvent::ExportError(msg) => {
                self.error_message = Some(msg.clone());
                self.info_message = None;
            }
            AppEvent::DismissError => {
                self.error_message = None;
                self.info_message = None;
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
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_current_recording_tracks_generation() {
        let data = AppData {
            recording_generation: 7,
            ..AppData::default()
        };

        // The active generation is current; any other is stale.
        assert!(data.is_current_recording(7));
        assert!(!data.is_current_recording(6));
        assert!(!data.is_current_recording(8));
    }

    #[test]
    fn cancel_invalidates_in_flight_worker_generation() {
        // Simulates the cancel -> stale RecordingFinished path. The full
        // `Model::event` handler can't be exercised in a unit test because it
        // needs a vizia `EventContext`, so this asserts the underlying
        // generation invariant that drives that handler's early-skip:
        //
        // A recording is spawned under generation G. CancelRecording bumps the
        // generation to G+1. When the cancelled worker later emits
        // RecordingFinished{generation: G} (core returns Ok(partial) on
        // cooperative cancel), the handler calls `is_current_recording(G)`,
        // which must now be false so the UI is NOT dragged Idle -> Review.
        let mut data = AppData::default();

        // Worker spawned under generation G (as StartRecording would bump it).
        data.recording_generation += 1;
        let g = data.recording_generation;
        data.app_state = AppState::Recording;
        assert!(data.is_current_recording(g));

        // CancelRecording: bump generation, return to Idle.
        data.recording_generation += 1;
        data.app_state = AppState::Idle;

        // The in-flight worker's RecordingFinished{generation: G} is now stale,
        // so the handler's guard rejects it and the state stays Idle.
        assert!(!data.is_current_recording(g));
        assert_eq!(data.app_state, AppState::Idle);
    }
}
