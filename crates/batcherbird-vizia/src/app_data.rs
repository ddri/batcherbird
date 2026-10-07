use crate::app_event::{AppEvent, InstrumentPreset};
use crate::session::{self, SessionSettings};
use batcherbird_core::channel_routing::ChannelRouting;
use batcherbird_core::export::AudioFormat;
use batcherbird_core::export::{ExportConfig, SampleExporter};
use batcherbird_core::lock_free_recording::RealtimeMeterData;
use batcherbird_core::preview_player::PreviewPlayer;
use batcherbird_core::sampler::{Sample, SamplingConfig, SamplingEngine, VizChunk};
use rtrb::Consumer;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use vizia::prelude::*;

fn safe_instrument_name(name: &str) -> String {
    let name: String = name
        .trim()
        .chars()
        .take(120)
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '-' | '_' | ' ') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let name = name.trim_matches([' ', '_', '.']);
    if name.is_empty() {
        "Untitled instrument".into()
    } else {
        name.into()
    }
}

pub struct PendingSession {
    path: PathBuf,
    settings: SessionSettings,
    samples: Vec<Sample>,
}

pub struct AuditionConnection {
    connection: midir::MidiOutputConnection,
    note: Option<u8>,
}
impl std::ops::Deref for AuditionConnection {
    type Target = midir::MidiOutputConnection;
    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}
impl std::ops::DerefMut for AuditionConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.connection
    }
}
impl Drop for AuditionConnection {
    fn drop(&mut self) {
        if let Some(note) = self.note {
            let _ = batcherbird_core::midi::MidiManager::send_note_off(
                &mut self.connection,
                0,
                note,
                0,
            );
        }
        let _ = batcherbird_core::midi::MidiManager::send_channel_panic(&mut self.connection, 0);
    }
}

pub struct RecordingWorker {
    cancel: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Drop for RecordingWorker {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

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
    Stopping,
    Review,
}

#[derive(Lens)]
pub struct AppData {
    pub instrument_name: String,
    pub release_duration_ms: u32,
    pub sample_labels: Vec<String>,
    pub selected_sample: usize,
    pub selected_sample_label: String,
    pub export_in_progress: bool,
    pub session_busy: bool,
    pub pending_session_name: Option<String>,
    #[lens(ignore)]
    pub pending_session: Option<PendingSession>,
    pub controls_busy: bool,
    pub session_status: String,
    pub has_unsaved_changes: bool,
    pub auto_loop: bool,
    pub trim_silence: bool,
    #[lens(ignore)]
    pub settings_revision: u64,
    #[lens(ignore)]
    pub replacement_sample: Option<usize>,
    #[lens(ignore)]
    pub preferences_path: Option<PathBuf>,
    #[lens(ignore)]
    pub preferred_midi_device: Option<String>,
    #[lens(ignore)]
    pub preferred_audio_device: Option<String>,
    #[lens(ignore)]
    pub preferences_dirty: bool,
    #[lens(ignore)]
    pub recording_meter_state: Option<Arc<batcherbird_core::sampler::LevelMeterState>>,
    #[lens(ignore)]
    pub worker_meter_slot: Arc<Mutex<Option<Arc<batcherbird_core::sampler::LevelMeterState>>>>,
    #[lens(ignore)]
    pub recording_worker: Option<RecordingWorker>,
    #[lens(ignore)]
    pub test_worker: Option<RecordingWorker>,
    #[lens(ignore)]
    pub test_generation: u64,
    pub demo_mode: bool,

    // Devices
    pub midi_devices: Vec<String>,
    pub audio_input_devices: Vec<String>,
    pub selected_midi_device: usize,
    pub selected_audio_input: usize,
    pub midi_connected: bool,
    pub audio_connected: bool,

    // Channel routing
    #[lens(ignore)]
    pub channel_routing: ChannelRouting,
    pub channel_routing_display: String,
    pub channel_routing_options: Vec<String>,
    pub selected_channel_routing: usize,

    // Input gain trim
    pub input_gain_db: f32,
    pub input_gain_display: String,

    // Sampling config
    pub start_note: u8,
    pub end_note: u8,
    pub velocity_layers: u8,
    pub note_duration_ms: u32,
    pub note_step: u8,
    pub note_step_options: Vec<String>,
    pub selected_step_index: usize,
    pub session_summary_display: String,

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
    pub is_testing_note: bool,
    pub gain_check_message: Option<String>,
    pub gain_check_status_color: String,
    pub audition_note: Option<u8>,
    #[lens(ignore)]
    pub audition_midi_conn: Option<AuditionConnection>,

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
    #[lens(ignore)]
    pub playthrough_stream: Option<cpal::Stream>,
    pub playthrough_enabled: bool,

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
    pub loop_start: Option<usize>,
    pub loop_end: Option<usize>,
    pub loop_detected: bool,
    pub sample_total_len: usize,
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
            instrument_name: "Untitled instrument".into(),
            release_duration_ms: 1000,
            sample_labels: Vec::new(),
            selected_sample: 0,
            selected_sample_label: "No sample selected".into(),
            export_in_progress: false,
            session_busy: false,
            pending_session_name: None,
            pending_session: None,
            controls_busy: false,
            session_status: "New session".into(),
            has_unsaved_changes: false,
            auto_loop: false,
            trim_silence: true,
            settings_revision: 0,
            replacement_sample: None,
            preferences_path: None,
            preferred_midi_device: None,
            preferred_audio_device: None,
            preferences_dirty: false,
            recording_meter_state: None,
            worker_meter_slot: Arc::new(Mutex::new(None)),
            recording_worker: None,
            test_worker: None,
            test_generation: 0,
            demo_mode: false,
            midi_devices: Vec::new(),
            audio_input_devices: Vec::new(),
            selected_midi_device: 0,
            selected_audio_input: 0,
            midi_connected: false,
            audio_connected: false,

            channel_routing: ChannelRouting::Stereo,
            channel_routing_display: ChannelRouting::Stereo.display_name().to_string(),
            channel_routing_options: vec![
                ChannelRouting::Stereo.display_name().to_string(),
                ChannelRouting::MonoLeft.display_name().to_string(),
                ChannelRouting::MonoRight.display_name().to_string(),
            ],
            selected_channel_routing: 0,

            input_gain_db: 0.0,
            input_gain_display: "0.0 dB".to_string(),

            start_note: 36, // C2
            end_note: 84,   // C6
            velocity_layers: 1,
            note_duration_ms: 2000,
            note_step: 1,
            note_step_options: vec![
                "Every Note".to_string(),
                "Every 3rd Note".to_string(),
                "Every Octave".to_string(),
            ],
            selected_step_index: 0,
            session_summary_display: "49 samples • ~2m 51s".to_string(),

            export_format: AudioFormat::Wav24Bit,
            export_format_display: "WAV 24-bit".to_string(),
            format_options: vec![
                "WAV 16-bit".to_string(),
                "WAV 24-bit".to_string(),
                "WAV 32-float".to_string(),
                "DecentSampler".to_string(),
                "SFZ".to_string(),
                "DecentSampler + SFZ".to_string(),
                "All Formats".to_string(),
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
            is_testing_note: false,
            gain_check_message: None,
            gain_check_status_color: "#888888".to_string(),
            audition_note: None,
            audition_midi_conn: None,

            current_note: 0,
            current_velocity: 0,
            current_layer: 0,
            total_layers: 0,
            notes_completed: 0,
            notes_total: 0,

            meter_consumer: None,
            sampling_engine: None,
            monitoring_stream: None,
            playthrough_stream: None,
            playthrough_enabled: false,

            viz_chunks: Vec::new(),
            viz_peaks: Vec::new(),

            recorded_count: 0,
            is_playing: false,
            playback_position: 0.0,
            loop_start: None,
            loop_end: None,
            loop_detected: false,
            sample_total_len: 0,
            preview_player: None,

            recorded_samples: Vec::new(),
            recorded_slot: Arc::new(Mutex::new(Vec::new())),
            cancel_flag: None,
            recording_generation: 0,
        }
    }
}

impl AppData {
    pub fn load_preferences() -> Self {
        let mut data = Self::default();
        if let Some(root) = dirs::config_dir() {
            let path = root.join("batcherbird/settings.json");
            if path.exists() {
                match session::load_settings(&path) {
                    Ok(settings) => data.apply_session_settings(settings),
                    Err(e) => data.error_message = Some(format!("Could not restore settings: {e}")),
                }
            }
            let recovery = root.join("batcherbird/recovery.batcherbird");
            if recovery.exists() {
                match session::load_session(&recovery) {
                    Ok((settings, samples)) if !samples.is_empty() => {
                        data.apply_session_settings(settings);
                        data.set_recorded_samples(samples);
                        data.app_state = AppState::Review;
                        data.has_unsaved_changes = true;
                        data.session_status = "Recovered capture checkpoint".into();
                        data.info_message = Some(
                            "Last captured batch recovered with its capture settings. Save this session to keep it, or open another saved session to continue your work."
                                .into(),
                        );
                    }
                    Ok(_) => {}
                    Err(e) => {
                        data.error_message = Some(format!("Could not recover recordings: {e}"))
                    }
                }
            }
            data.preferences_path = Some(path);
        }
        data
    }

    /// Hardware-free UI preview. No device enumeration or settings writes.
    pub fn demo() -> Self {
        use std::time::{Duration, SystemTime};
        let mut data = Self {
            instrument_name: "DW6000 · Warm pad".into(),
            demo_mode: true,
            ..Self::default()
        };
        data.start_note = 48;
        data.end_note = 72;
        data.note_step = 3;
        data.selected_step_index = 1;
        data.velocity_layers = 2;
        data.midi_devices = vec!["Demo MIDI output".into()];
        data.audio_input_devices = vec!["Demo audio interface".into()];
        let mut samples = Vec::new();
        for note in (48..=72).step_by(3) {
            for velocity in [64, 127] {
                let rate = 24000;
                let frequency = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);
                let audio_data = (0..rate * 2)
                    .flat_map(|frame| {
                        let t = frame as f32 / rate as f32;
                        let envelope = (t * 5.0).min(1.0) * ((2.0 - t) * 2.0).clamp(0.0, 1.0);
                        let phase = t * frequency * std::f32::consts::TAU;
                        let value =
                            (phase.sin() + (phase * 2.0).sin() * 0.25) * envelope * velocity as f32
                                / 400.0;
                        [value, value * 0.9]
                    })
                    .collect();
                samples.push(Sample {
                    note,
                    velocity,
                    audio_data,
                    sample_rate: rate,
                    channels: 2,
                    recorded_at: SystemTime::now(),
                    midi_timing: Duration::from_secs(1),
                    audio_timing: Duration::from_secs(2),
                });
            }
        }
        data.set_recorded_samples(samples);
        data.app_state = AppState::Review;
        data.session_status = "Demo session".into();
        data.info_message = Some("Demo recordings · no hardware connected".into());
        data.update_summary();
        data
    }

    pub fn is_busy(&self) -> bool {
        matches!(self.app_state, AppState::Recording | AppState::Stopping)
            || self.export_in_progress
            || self.session_busy
            || self.is_testing_note
    }

    pub fn session_settings(&self) -> SessionSettings {
        SessionSettings {
            instrument_name: self.instrument_name.clone(),
            start_note: self.start_note,
            end_note: self.end_note,
            velocity_layers: self.velocity_layers,
            note_step: self.note_step,
            note_duration_ms: self.note_duration_ms,
            release_duration_ms: self.release_duration_ms,
            channel_routing: self.selected_channel_routing,
            input_gain_db: self.input_gain_db,
            export_format: self.selected_format_index,
            output_directory: self.output_directory.clone(),
            midi_device: self
                .midi_devices
                .as_slice()
                .get(self.selected_midi_device)
                .cloned()
                .or_else(|| self.preferred_midi_device.clone()),
            audio_device: self
                .audio_input_devices
                .as_slice()
                .get(self.selected_audio_input)
                .cloned()
                .or_else(|| self.preferred_audio_device.clone()),
            auto_loop: self.auto_loop,
            trim_silence: self.trim_silence,
        }
    }

    pub fn apply_session_settings(&mut self, settings: SessionSettings) {
        self.instrument_name = settings.instrument_name.chars().take(120).collect();
        if self.instrument_name.trim().is_empty() {
            self.instrument_name = "Untitled instrument".into();
        }
        self.start_note = settings.start_note.min(127);
        self.end_note = settings.end_note.min(127).max(self.start_note);
        self.velocity_layers = settings.velocity_layers.clamp(1, 4);
        self.note_duration_ms = settings.note_duration_ms.clamp(500, 10000);
        self.release_duration_ms = settings.release_duration_ms.min(10000);
        self.note_step = match settings.note_step {
            3 => 3,
            12 => 12,
            _ => 1,
        };
        self.selected_step_index = match self.note_step {
            3 => 1,
            12 => 2,
            _ => 0,
        };
        self.set_channel_routing_index(settings.channel_routing.min(2));
        self.set_input_gain_db(if settings.input_gain_db.is_finite() {
            settings.input_gain_db
        } else {
            0.0
        });
        self.selected_format_index = settings.export_format.min(6);
        self.export_format = Self::format_at_index(self.selected_format_index);
        self.export_format_display = Self::format_display(&self.export_format).into();
        self.output_directory = settings.output_directory;
        self.preferred_midi_device = settings.midi_device;
        self.preferred_audio_device = settings.audio_device;
        self.auto_loop = settings.auto_loop;
        self.trim_silence = settings.trim_silence;
        self.update_summary();
    }

    pub fn format_at_index(index: usize) -> AudioFormat {
        match index {
            0 => AudioFormat::Wav16Bit,
            2 => AudioFormat::Wav32BitFloat,
            3 => AudioFormat::DecentSampler,
            4 => AudioFormat::SFZ,
            5 => AudioFormat::DecentSamplerAndSfz,
            6 => AudioFormat::All,
            _ => AudioFormat::Wav24Bit,
        }
    }

    pub fn set_recorded_samples(&mut self, samples: Vec<Sample>) {
        self.recorded_samples = samples;
        self.recorded_count = self.recorded_samples.len() as u32;
        self.sample_labels = self
            .recorded_samples
            .iter()
            .map(|sample| {
                let seconds = sample.audio_data.len() as f32
                    / sample.channels.max(1) as f32
                    / sample.sample_rate.max(1) as f32;
                format!(
                    "{} · velocity {} · {:.1}s",
                    Self::note_name(sample.note),
                    sample.velocity,
                    seconds
                )
            })
            .collect();
        self.select_sample(
            self.selected_sample
                .min(self.recorded_samples.len().saturating_sub(1)),
        );
    }

    pub fn select_sample(&mut self, index: usize) {
        self.stop_preview();
        self.selected_sample = index.min(self.recorded_samples.len().saturating_sub(1));
        self.selected_sample_label = self
            .sample_labels
            .as_slice()
            .get(self.selected_sample)
            .cloned()
            .unwrap_or_else(|| "No sample selected".into());
        self.loop_start = None;
        self.loop_end = None;
        self.loop_detected = false;
        if let Some(sample) = self.recorded_samples.get(self.selected_sample) {
            self.viz_peaks = samples_to_peaks(&sample.audio_data, 512);
            self.sample_total_len = sample.audio_data.len() / sample.channels.max(1) as usize;
            if self.auto_loop {
                let detector =
                    batcherbird_core::loop_detection::LoopDetector::new(Default::default());
                let result = detector.detect_loop_points_channels(
                    &sample.audio_data,
                    sample.sample_rate,
                    sample.channels,
                );
                if result.success {
                    if let Some(candidate) = result.best_candidate {
                        self.loop_start = Some(candidate.start_sample);
                        self.loop_end = Some(candidate.end_sample);
                        self.loop_detected = true;
                    }
                }
            }
        } else {
            self.viz_peaks.clear();
            self.sample_total_len = 0;
        }
    }

    pub fn request_stop(&mut self) {
        if self.app_state == AppState::Recording {
            if let Some(flag) = &self.cancel_flag {
                flag.store(true, Ordering::Release);
            }
            self.app_state = AppState::Stopping;
            self.info_message = Some("Stopping safely and keeping completed samples…".into());
        }
    }

    pub fn accept_captured_samples(&mut self, samples: Vec<Sample>) {
        if let Some(index) = self.replacement_sample.take() {
            if let Some(sample) = samples.into_iter().next() {
                if let Some(old) = self.recorded_samples.get_mut(index) {
                    *old = sample;
                }
                self.mark_changed();
            }
            let updated = std::mem::take(&mut self.recorded_samples);
            self.set_recorded_samples(updated);
        } else if !samples.is_empty() {
            self.set_recorded_samples(samples);
            self.mark_changed();
        } else {
            self.select_sample(self.selected_sample);
        }
        self.app_state = if self.recorded_samples.is_empty() {
            AppState::Idle
        } else {
            AppState::Review
        };
    }

    /// Hold a validated candidate without touching the edited session until the user chooses.
    fn receive_loaded_session(
        &mut self,
        path: PathBuf,
        settings: SessionSettings,
        samples: Vec<Sample>,
    ) {
        let candidate = PendingSession {
            path,
            settings,
            samples,
        };
        if self.has_unsaved_changes {
            self.pending_session_name = Some(
                candidate
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into(),
            );
            self.pending_session = Some(candidate);
            self.session_busy = true;
        } else {
            self.apply_loaded_session(candidate);
        }
    }

    fn resolve_session_replacement(&mut self, replace: bool) {
        let Some(candidate) = self.pending_session.take() else {
            return;
        };
        self.pending_session_name = None;
        if replace {
            self.apply_loaded_session(candidate);
        } else {
            self.session_busy = false;
        }
    }

    fn apply_loaded_session(&mut self, candidate: PendingSession) {
        let PendingSession {
            path,
            settings,
            samples,
        } = candidate;
        self.session_busy = false;
        self.monitoring_stream = None;
        self.playthrough_stream = None;
        self.sampling_engine = None;
        self.silence_audition();
        self.apply_session_settings(settings);
        self.restore_device_selections();
        self.set_recorded_samples(samples);
        self.app_state = if self.recorded_samples.is_empty() {
            AppState::Idle
        } else {
            AppState::Review
        };
        self.settings_revision += 1;
        self.has_unsaved_changes = false;
        self.preferences_dirty = true;
        self.session_status = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into();
        self.info_message = Some(format!("Session opened from {}", path.display()));
        self.error_message = None;
    }

    fn mark_changed(&mut self) {
        self.settings_revision += 1;
        self.has_unsaved_changes = true;
        self.preferences_dirty = true;
    }

    fn restore_device_selections(&mut self) {
        if let Some(name) = &self.preferred_midi_device {
            if let Some(index) = self.midi_devices.iter().position(|device| device == name) {
                self.selected_midi_device = index;
            }
        }
        if let Some(name) = &self.preferred_audio_device {
            if let Some(index) = self
                .audio_input_devices
                .iter()
                .position(|device| device == name)
            {
                self.selected_audio_input = index;
            }
        }
        self.selected_midi_device = self
            .selected_midi_device
            .min(self.midi_devices.len().saturating_sub(1));
        self.selected_audio_input = self
            .selected_audio_input
            .min(self.audio_input_devices.len().saturating_sub(1));
        self.midi_connected = !self.midi_devices.is_empty();
        self.audio_connected = !self.audio_input_devices.is_empty();
    }

    fn silence_audition(&mut self) {
        if let Some(note) = self.audition_note.take() {
            if let Some(conn) = &mut self.audition_midi_conn {
                let _ = batcherbird_core::midi::MidiManager::send_note_off(conn, 0, note, 0);
            }
        }
        if let Some(conn) = &mut self.audition_midi_conn {
            let _ = batcherbird_core::midi::MidiManager::send_channel_panic(conn, 0);
        }
        self.audition_midi_conn = None;
    }

    fn disarm_monitoring(&mut self) {
        self.silence_audition();
        self.monitoring_stream = None;
        self.playthrough_stream = None;
        self.sampling_engine = None;
        self.meter_left = 0.0;
        self.meter_right = 0.0;
        self.meter_left_db = -60.0;
        self.meter_right_db = -60.0;
        self.app_state = AppState::Idle;
    }

    fn begin_test_note(&mut self, cx: &mut EventContext) {
        if self.app_state != AppState::Armed || self.is_busy() || self.demo_mode {
            return;
        }
        self.silence_audition();
        let Some(engine) = &self.sampling_engine else {
            return;
        };
        let meters = engine.get_level_meter_state();
        self.is_testing_note = true;
        self.gain_check_message = Some("Checking input headroom at maximum velocity…".into());
        self.gain_check_status_color = "#79b8ca".into();
        self.test_generation += 1;
        let generation = self.test_generation;
        let (note, midi_index) = (self.start_note, self.selected_midi_device);
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let mut proxy = cx.get_proxy();
        let handle = std::thread::spawn(move || {
            let result = (|| -> Result<(f32, f32), String> {
                let mut manager =
                    batcherbird_core::midi::MidiManager::new().map_err(|e| e.to_string())?;
                let connection = manager
                    .connect_output(midi_index)
                    .map_err(|e| e.to_string())?;
                let mut connection = AuditionConnection {
                    connection,
                    note: Some(note),
                };
                batcherbird_core::midi::MidiManager::send_note_on(&mut connection, 0, note, 127)
                    .map_err(|e| e.to_string())?;
                let mut peak = 0.0_f32;
                for tick in 0..100 {
                    if worker_cancel.load(Ordering::Acquire) {
                        break;
                    }
                    if tick == 70 {
                        batcherbird_core::midi::MidiManager::send_note_off(
                            &mut connection,
                            0,
                            note,
                            0,
                        )
                        .map_err(|e| e.to_string())?;
                        connection.note = None;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    peak = peak.max(meters.get_levels().peak);
                }
                // The connection guard sends Note Off and channel panic on every exit.
                Ok((
                    if peak > 0.0 {
                        20.0 * peak.log10()
                    } else {
                        -60.0
                    },
                    peak,
                ))
            })();
            match result {
                Ok((peak_db, peak_linear)) => {
                    let _ = proxy.emit(AppEvent::TestNoteResult {
                        generation,
                        peak_db,
                        peak_linear,
                    });
                }
                Err(message) => {
                    let _ = proxy.emit(AppEvent::TestNoteError {
                        generation,
                        message,
                    });
                }
            }
        });
        self.test_worker = Some(RecordingWorker {
            cancel,
            handle: Some(handle),
        });
    }

    fn save_recovery(&mut self, cx: &mut EventContext) {
        let Some(path) = self
            .preferences_path
            .as_ref()
            .and_then(|path| path.parent())
            .map(|parent| parent.join("recovery.batcherbird"))
        else {
            return;
        };
        if self.recorded_samples.is_empty() {
            return;
        }
        let settings = self.session_settings();
        let samples = self.recorded_samples.clone();
        self.session_busy = true;
        let mut proxy = cx.get_proxy();
        std::thread::spawn(move || {
            let error = session::save_session(&path, &settings, &samples).err();
            let _ = proxy.emit(AppEvent::RecoveryComplete(error));
        });
    }

    fn begin_recording(&mut self, cx: &mut EventContext, replacement: Option<usize>) {
        if self.is_busy() {
            return;
        }
        if self.demo_mode {
            self.info_message = Some(
                "Demo mode uses generated samples. Launch without --demo to record hardware."
                    .into(),
            );
            return;
        }
        if replacement.is_none() && self.app_state != AppState::Armed {
            return;
        }
        if replacement.is_some() && self.app_state != AppState::Review {
            return;
        }
        if self.midi_devices.is_empty() || self.audio_input_devices.is_empty() {
            self.error_message =
                Some("Connect an audio input and MIDI output, then refresh devices.".into());
            return;
        }
        let replace_note = replacement
            .and_then(|index| self.recorded_samples.get(index))
            .map(|sample| (sample.note, sample.velocity));
        if replacement.is_some() && replace_note.is_none() {
            return;
        }
        self.stop_preview();
        self.silence_audition();
        self.monitoring_stream = None;
        self.playthrough_stream = None;
        self.sampling_engine = None;
        self.app_state = AppState::Recording;
        self.notes_total = if replacement.is_some() {
            1
        } else {
            self.total_samples()
        };
        self.notes_completed = 0;
        self.current_note = replace_note.map_or(self.start_note, |(note, _)| note);
        self.current_velocity = replace_note.map_or(127, |(_, velocity)| velocity);
        self.viz_chunks.clear();
        self.viz_peaks.clear();
        self.loop_start = None;
        self.loop_end = None;
        self.error_message = None;
        self.info_message = None;
        self.replacement_sample = replacement;
        self.recording_generation += 1;
        let generation = self.recording_generation;
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancel_flag = Some(cancel.clone());
        let recorded_slot = Arc::new(Mutex::new(Vec::new()));
        self.recorded_slot = recorded_slot.clone();
        let meter_slot = Arc::new(Mutex::new(None));
        self.worker_meter_slot = meter_slot.clone();
        self.recording_meter_state = None;
        let config = self.build_sampling_config();
        let (start, end, step, layers) = (
            self.start_note,
            self.end_note,
            self.note_step,
            self.velocity_layers,
        );
        let midi_index = self.selected_midi_device;
        let worker_cancel = cancel.clone();
        let mut proxy = cx.get_proxy();
        let handle = std::thread::spawn(move || {
            let result = (|| -> Result<(), String> {
                let mut manager =
                    batcherbird_core::midi::MidiManager::new().map_err(|e| e.to_string())?;
                let mut connection = manager
                    .connect_output(midi_index)
                    .map_err(|e| e.to_string())?;
                let engine = SamplingEngine::new(config).map_err(|e| e.to_string())?;
                if let Ok(mut slot) = meter_slot.lock() {
                    *slot = Some(engine.get_level_meter_state());
                }
                if let Some((note, velocity)) = replace_note {
                    if let Some(sample) = engine
                        .sample_note_velocity_with_cancel_blocking(
                            &mut connection,
                            note,
                            velocity,
                            &worker_cancel,
                        )
                        .map_err(|e| e.to_string())?
                    {
                        if let Ok(mut slot) = recorded_slot.lock() {
                            slot.push(sample);
                        }
                    }
                } else {
                    engine
                        .sample_note_range_stepped_with_capture_blocking(
                            &mut connection,
                            start,
                            end,
                            step,
                            layers,
                            &worker_cancel,
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
                            |sample| {
                                if let Ok(mut slot) = recorded_slot.lock() {
                                    slot.push(sample.clone());
                                }
                            },
                        )
                        .map_err(|e| e.to_string())?;
                }
                Ok(())
            })();
            match result {
                Ok(()) => {
                    let _ = proxy.emit(AppEvent::RecordingFinished { generation });
                }
                Err(message) => {
                    let _ = proxy.emit(AppEvent::RecordingError {
                        generation,
                        message,
                    });
                }
            }
        });
        self.recording_worker = Some(RecordingWorker {
            cancel,
            handle: Some(handle),
        });
    }

    /// The same export choices used by the desktop action and saved sessions.
    pub fn build_export_config(&self) -> ExportConfig {
        let instrument = safe_instrument_name(&self.instrument_name);
        ExportConfig {
            output_directory: self.output_directory.join(&instrument),
            naming_pattern: format!("{instrument}_{{note_name}}_{{note}}_{{velocity}}.wav"),
            instrument_description: Some(self.instrument_name.clone()),
            auto_loop: self.auto_loop,
            apply_detection: self.trim_silence,
            sample_format: self.export_format.clone(),
            ..ExportConfig::default()
        }
    }

    pub fn build_sampling_config(&self) -> SamplingConfig {
        // Resolve the user's selected audio input device (if any) to a name so
        // recording uses the chosen device rather than the system default.
        let input_device_name = self
            .audio_input_devices
            .as_slice()
            .get(self.selected_audio_input)
            .cloned();

        SamplingConfig {
            note_duration_ms: self.note_duration_ms as u64,
            release_time_ms: self.release_duration_ms as u64,
            pre_delay_ms: 100,
            post_delay_ms: 100,
            midi_channel: 0,
            velocity: 100,
            input_device_name,
            channel_routing: self.channel_routing,
            input_gain_db: self.input_gain_db,
        }
    }

    pub fn set_channel_routing_index(&mut self, idx: usize) {
        if idx < self.channel_routing_options.len() {
            self.selected_channel_routing = idx;
            self.channel_routing = match idx {
                0 => ChannelRouting::Stereo,
                1 => ChannelRouting::MonoLeft,
                2 => ChannelRouting::MonoRight,
                _ => ChannelRouting::Stereo,
            };
            self.channel_routing_display = self.channel_routing_options[idx].clone();
            if let Some(engine) = &self.sampling_engine {
                engine.set_channel_routing(self.channel_routing);
            }
        }
    }

    pub fn cycle_channel_routing(&mut self) {
        let next_idx = (self.selected_channel_routing + 1) % self.channel_routing_options.len();
        self.set_channel_routing_index(next_idx);
    }

    pub fn set_input_gain_db(&mut self, db: f32) {
        let clamped = db.clamp(-12.0, 12.0);
        self.input_gain_db = (clamped * 10.0).round() / 10.0;
        self.input_gain_display = if self.input_gain_db > 0.0 {
            format!("+{:.1} dB", self.input_gain_db)
        } else {
            format!("{:.1} dB", self.input_gain_db)
        };
        if let Some(engine) = &self.sampling_engine {
            engine.set_input_gain_db(self.input_gain_db);
        }
    }

    pub fn adjust_input_gain_db(&mut self, delta: f32) {
        self.set_input_gain_db(self.input_gain_db + delta);
    }

    pub fn reset_input_gain_db(&mut self) {
        self.set_input_gain_db(0.0);
    }

    /// Whether `generation` matches the current recording session. Worker→UI
    /// events carry the generation they were spawned under; once the generation
    /// is bumped (new recording or cancel), the in-flight worker's events become
    /// stale and are ignored.
    pub fn is_current_recording(&self, generation: u64) -> bool {
        generation == self.recording_generation
    }

    pub fn total_samples(&self) -> u32 {
        let step = (self.note_step as u32).max(1);
        let span = (self.end_note as u32).saturating_sub(self.start_note as u32);
        let num_notes = (span / step) + 1;
        num_notes * self.velocity_layers as u32
    }

    pub fn estimated_duration_display(&self) -> String {
        let secs = self.estimated_duration_secs().round() as u32;
        if secs < 60 {
            format!("~{}s", secs)
        } else {
            format!("~{}m {}s", secs / 60, secs % 60)
        }
    }

    pub fn update_summary(&mut self) {
        self.session_summary_display = format!(
            "{} samples • {}",
            self.total_samples(),
            self.estimated_duration_display()
        );
    }

    pub fn apply_octave_preset(&mut self, octaves: u8) {
        match octaves {
            1 => {
                self.start_note = 48; // C3
                self.end_note = 60; // C4
            }
            2 => {
                self.start_note = 36; // C2
                self.end_note = 60; // C4
            }
            4 => {
                self.start_note = 36; // C2
                self.end_note = 84; // C6
            }
            _ => {}
        }
        self.update_summary();
    }

    pub fn apply_instrument_preset(&mut self, preset: InstrumentPreset) {
        match preset {
            InstrumentPreset::Lead => {
                self.start_note = 48; // C3
                self.end_note = 72; // C5
                self.note_step = 1; // Every Note
                self.selected_step_index = 0;
                self.velocity_layers = 2; // Soft, Loud
                self.note_duration_ms = 2000;
            }
            InstrumentPreset::Pad => {
                self.start_note = 36; // C2
                self.end_note = 84; // C6
                self.note_step = 3; // Every 3rd Note
                self.selected_step_index = 1;
                self.velocity_layers = 2;
                self.note_duration_ms = 4000;
            }
            InstrumentPreset::Bass => {
                self.start_note = 24; // C1
                self.end_note = 48; // C3
                self.note_step = 1; // Every Note
                self.selected_step_index = 0;
                self.velocity_layers = 2;
                self.note_duration_ms = 1500;
            }
            InstrumentPreset::Pluck => {
                self.start_note = 36; // C2
                self.end_note = 60; // C4
                self.note_step = 1; // Every Note
                self.selected_step_index = 0;
                self.velocity_layers = 4; // Expressive dynamic velocities
                self.note_duration_ms = 1000;
            }
        }
        self.update_summary();
    }

    pub fn set_start_note(&mut self, note: u8) {
        let note = note.min(127);
        if note <= self.end_note {
            self.start_note = note;
        } else {
            self.start_note = self.end_note;
            self.end_note = note;
        }
        self.update_summary();
    }

    pub fn set_end_note(&mut self, note: u8) {
        let note = note.min(127);
        if note >= self.start_note {
            self.end_note = note;
        } else {
            self.end_note = self.start_note;
            self.start_note = note;
        }
        self.update_summary();
    }

    pub fn set_playthrough(&mut self, enabled: bool) {
        self.playthrough_enabled = enabled;
        if let Some(engine) = &self.sampling_engine {
            self.monitoring_stream = None;
            self.playthrough_stream = None;
            match engine.start_monitoring_stream_with_playthrough(enabled) {
                Ok((input, output)) => {
                    self.playthrough_enabled = enabled && output.is_some();
                    self.monitoring_stream = Some(input);
                    self.playthrough_stream = output;
                    if enabled && !self.playthrough_enabled {
                        self.error_message = Some("Software monitoring is unavailable for this output configuration. Use your interface's direct monitoring.".into());
                    }
                }
                Err(error) => {
                    self.playthrough_enabled = false;
                    engine.set_playthrough(false);
                    self.monitoring_stream = engine.start_monitoring_stream().ok();
                    self.error_message =
                        Some(format!("Could not start software monitoring: {error}"));
                }
            }
        }
    }

    /// Evaluate audio peak level and determine gain staging advice and status color.
    pub fn evaluate_gain_staging(peak_db: f32, peak_linear: f32) -> (String, &'static str) {
        if peak_linear >= 0.99 || peak_db >= -0.1 {
            (
                format!(
                    "⚠️ Clipping detected ({:.1} dB)! Lower hardware gain.",
                    peak_db
                ),
                "#ff4444", // Red
            )
        } else if peak_db > -3.0 {
            (
                format!(
                    "⚠️ Hot signal ({:.1} dB). Recommend lowering gain slightly.",
                    peak_db
                ),
                "#ffaa00", // Amber
            )
        } else if peak_db >= -18.0 {
            (
                format!("✓ Optimal headroom ({:.1} dB). Ready to record!", peak_db),
                "#00e676", // Green
            )
        } else if peak_db > -45.0 {
            (
                format!(
                    "ℹ Level low ({:.1} dB). Increase gain for better SNR.",
                    peak_db
                ),
                "#4a9eff", // Blue
            )
        } else {
            (
                "⚠️ No signal detected. Check audio cable & synth volume.".to_string(),
                "#888899", // Muted gray
            )
        }
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
            AudioFormat::Wav16Bit => "WAV 16-bit",
            AudioFormat::Wav24Bit => "WAV 24-bit",
            AudioFormat::Wav32BitFloat => "WAV 32-bit float",
            AudioFormat::DecentSampler => "DecentSampler",
            AudioFormat::SFZ => "SFZ",
            AudioFormat::DecentSamplerAndSfz => "DecentSampler + SFZ",
            AudioFormat::All => "All Formats",
        }
    }

    pub fn next_format(fmt: &AudioFormat) -> AudioFormat {
        match fmt {
            AudioFormat::Wav16Bit => AudioFormat::Wav24Bit,
            AudioFormat::Wav24Bit => AudioFormat::Wav32BitFloat,
            AudioFormat::Wav32BitFloat => AudioFormat::DecentSampler,
            AudioFormat::DecentSampler => AudioFormat::SFZ,
            AudioFormat::SFZ => AudioFormat::DecentSamplerAndSfz,
            AudioFormat::DecentSamplerAndSfz => AudioFormat::All,
            AudioFormat::All => AudioFormat::Wav16Bit,
        }
    }

    pub fn prev_format(fmt: &AudioFormat) -> AudioFormat {
        match fmt {
            AudioFormat::Wav16Bit => AudioFormat::All,
            AudioFormat::Wav24Bit => AudioFormat::Wav16Bit,
            AudioFormat::Wav32BitFloat => AudioFormat::Wav24Bit,
            AudioFormat::DecentSampler => AudioFormat::Wav32BitFloat,
            AudioFormat::SFZ => AudioFormat::DecentSampler,
            AudioFormat::DecentSamplerAndSfz => AudioFormat::SFZ,
            AudioFormat::All => AudioFormat::DecentSamplerAndSfz,
        }
    }

    pub fn estimated_duration_secs(&self) -> f32 {
        let total = self.total_samples() as f32;
        let per_note_secs =
            (self.note_duration_ms + self.release_duration_ms) as f32 / 1000.0 + 0.5;
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
        event.map(|app_event: &AppEvent, _| {
            let configuration_change = matches!(app_event,
                AppEvent::SetStartNote(_) | AppEvent::SetEndNote(_) | AppEvent::SetVelocityLayers(_)
                | AppEvent::SetDuration(_) | AppEvent::SetReleaseDuration(_) | AppEvent::SetInstrumentName(_)
                | AppEvent::SetAutoLoop(_) | AppEvent::SetTrimSilence(_) | AppEvent::SetExportFormat(_) | AppEvent::SetOutputDirectory(_)
                | AppEvent::CycleExportFormat | AppEvent::CycleExportFormatBack | AppEvent::SelectFormatByIndex(_)
                | AppEvent::SelectNoteStepByIndex(_) | AppEvent::SetOctavePreset(_) | AppEvent::ApplyInstrumentPreset(_)
                | AppEvent::IncrementStartNote | AppEvent::DecrementStartNote | AppEvent::IncrementEndNote
                | AppEvent::DecrementEndNote | AppEvent::IncrementVelocityLayers | AppEvent::DecrementVelocityLayers
                | AppEvent::IncrementDuration | AppEvent::DecrementDuration | AppEvent::SelectMidiDevice(_)
                | AppEvent::SelectAudioInput(_) | AppEvent::CycleNextMidiDevice | AppEvent::CyclePrevMidiDevice
                | AppEvent::CycleNextAudioInput | AppEvent::CyclePrevAudioInput | AppEvent::SelectChannelRouting(_)
                | AppEvent::CycleChannelRouting | AppEvent::SetInputGain(_) | AppEvent::AdjustInputGain(_)
                | AppEvent::ResetInputGain);
            if configuration_change {
                if self.is_busy() { return; }
                if matches!(app_event, AppEvent::SelectAudioInput(_) | AppEvent::CycleNextAudioInput | AppEvent::CyclePrevAudioInput) && self.app_state == AppState::Armed {
                    self.disarm_monitoring();
                    self.info_message = Some("Audio input changed. Check its signal before recording.".into());
                }
                if matches!(app_event, AppEvent::SelectMidiDevice(_) | AppEvent::CycleNextMidiDevice | AppEvent::CyclePrevMidiDevice) { self.silence_audition(); }
                self.mark_changed();
            }
            match app_event {
            AppEvent::RefreshDevices => {
                if self.is_busy() || self.demo_mode { return; }
                if self.app_state == AppState::Armed { self.disarm_monitoring(); }
                self.preferred_midi_device = self.midi_devices.as_slice().get(self.selected_midi_device).cloned().or(self.preferred_midi_device.take());
                self.preferred_audio_device = self.audio_input_devices.as_slice().get(self.selected_audio_input).cloned().or(self.preferred_audio_device.take());
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
                self.restore_device_selections();
            }
            AppEvent::Tick => {
                if self.preferences_dirty && !self.is_busy() {
                    if let Some(path) = &self.preferences_path {
                        if let Err(error) = session::save_settings(path, &self.session_settings()) {
                            self.error_message = Some(format!("Could not save preferences: {error}"));
                        }
                    }
                    self.preferences_dirty = false;
                }
                if self.recording_meter_state.is_none() && matches!(self.app_state, AppState::Recording | AppState::Stopping) {
                    if let Ok(slot) = self.worker_meter_slot.lock() { self.recording_meter_state = slot.clone(); }
                }
                if let Some(state) = &self.recording_meter_state {
                    let levels = state.get_levels();
                    self.viz_peaks = state.get_waveform_peaks();
                    self.meter_left = levels.peak_left;
                    self.meter_right = levels.peak_right;
                    self.meter_left_db = levels.peak_left_db;
                    self.meter_right_db = levels.peak_right_db;
                    self.is_clipping = levels.peak >= 1.0;
                }
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
                        self.meter_left = levels.peak_left;
                        self.meter_right = levels.peak_right;
                        self.meter_left_db = levels.peak_left_db;
                        self.meter_right_db = levels.peak_right_db;
                    }
                }
                // Reflect one-shot preview completion in the UI: once the
                // player reaches the end (or there's no player), clear playing
                // state so the Play/Stop button updates.
                if let Some(player) = &self.preview_player { self.playback_position = player.playback_position() as f64; }
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
            AppEvent::SetInstrumentName(name) => { self.instrument_name = name.chars().take(120).collect(); }
            AppEvent::SetReleaseDuration(ms) => { self.release_duration_ms = (*ms).min(10000); self.update_summary(); }
            AppEvent::SetTrimSilence(enabled) => { self.trim_silence = *enabled; }
            AppEvent::SetAutoLoop(enabled) => { self.auto_loop = *enabled; self.select_sample(self.selected_sample); }
            AppEvent::SelectSample(index) => { if !self.is_busy() { self.select_sample(*index); } }
            AppEvent::SaveSession => {
                if self.is_busy() { return; }
                self.session_busy = true;
                let settings = self.session_settings();
                let samples = self.recorded_samples.clone();
                let revision = self.settings_revision;
                let mut proxy = cx.get_proxy();
                std::thread::spawn(move || {
                    let filename = format!("{}.batcherbird", safe_instrument_name(&settings.instrument_name));
                    if let Some(path) = rfd::FileDialog::new().add_filter("Batcherbird session", &["batcherbird"]).set_file_name(filename).save_file() {
                        match session::save_session(&path, &settings, &samples) {
                            Ok(()) => { let _ = proxy.emit(AppEvent::SessionSaved { path, revision }); }
                            Err(error) => { let _ = proxy.emit(AppEvent::SessionError(error)); }
                        }
                    } else { let _ = proxy.emit(AppEvent::SessionDialogCancelled); }
                });
            }
            AppEvent::OpenSession => {
                if self.is_busy() { return; }
                self.stop_preview();
                self.session_busy = true;
                let mut proxy = cx.get_proxy();
                std::thread::spawn(move || {
                    if let Some(path) = rfd::FileDialog::new().add_filter("Batcherbird session", &["batcherbird"]).pick_file() {
                        match session::load_session(&path) {
                            Ok((settings, samples)) => {
                                let _ = proxy.emit(AppEvent::SessionLoaded { path, settings, samples });
                            }
                            Err(error) => { let _ = proxy.emit(AppEvent::SessionError(error)); }
                        }
                    } else { let _ = proxy.emit(AppEvent::SessionDialogCancelled); }
                });
            }
            AppEvent::SessionSaved { path, revision } => {
                self.session_busy = false;
                self.error_message = None;
                if *revision == self.settings_revision { self.has_unsaved_changes = false; }
                self.session_status = path.file_name().unwrap_or_default().to_string_lossy().into();
                self.info_message = Some(format!("Session saved to {}", path.display()));
            }
            AppEvent::SessionLoaded { path, settings, samples } => {
                self.receive_loaded_session(path.clone(), settings.clone(), samples.clone());
            }
            AppEvent::ConfirmSessionReplacement => { self.resolve_session_replacement(true); }
            AppEvent::KeepCurrentSession => { self.resolve_session_replacement(false); }
            AppEvent::SessionError(error) => { self.session_busy = false; self.info_message = None; self.error_message = Some(error.clone()); }
            AppEvent::SessionDialogCancelled => { self.session_busy = false; }
            AppEvent::RecoveryComplete(error) => {
                self.session_busy = false;
                if let Some(error) = error { self.error_message = Some(format!("Automatic recovery save failed: {error}. Save your session manually.")); }
            }
            AppEvent::SetStartNote(n) => {
                self.set_start_note(*n);
            }
            AppEvent::SetEndNote(n) => {
                self.set_end_note(*n);
            }
            AppEvent::SetVelocityLayers(n) => {
                self.velocity_layers = (*n).clamp(1, 4);
                self.update_summary();
            }
            AppEvent::SetDuration(ms) => {
                self.note_duration_ms = (*ms).clamp(500, 10000);
                self.update_summary();
            }
            AppEvent::SetExportFormat(fmt) => {
                self.export_format_display = Self::format_display(fmt).to_string();
                self.export_format = fmt.clone();
            }
            AppEvent::SetOutputDirectory(path) => self.output_directory = path.clone(),

            AppEvent::SelectNoteStepByIndex(idx) => {
                let steps = [1, 3, 12];
                if *idx < steps.len() {
                    self.selected_step_index = *idx;
                    self.note_step = steps[*idx];
                    self.update_summary();
                }
            }
            AppEvent::SetOctavePreset(octaves) => {
                self.apply_octave_preset(*octaves);
            }
            AppEvent::ApplyInstrumentPreset(preset) => {
                self.apply_instrument_preset(*preset);
            }

            AppEvent::CycleNextMidiDevice => {
                if !self.midi_devices.is_empty() {
                    self.selected_midi_device = (self.selected_midi_device + 1) % self.midi_devices.len();
                    self.audition_midi_conn = None;
                }
            }
            AppEvent::CycleNextAudioInput => {
                if !self.audio_input_devices.is_empty() {
                    self.selected_audio_input = (self.selected_audio_input + 1) % self.audio_input_devices.len();
                }
            }
            AppEvent::SelectMidiDevice(idx) => {
                self.selected_midi_device = *idx;
                self.audition_midi_conn = None;
            }
            AppEvent::SelectAudioInput(idx) => {
                self.selected_audio_input = *idx;
            }
            AppEvent::SelectChannelRouting(idx) => {
                self.set_channel_routing_index(*idx);
            }
            AppEvent::CycleChannelRouting => {
                self.cycle_channel_routing();
            }
            AppEvent::SetInputGain(db) => {
                self.set_input_gain_db(*db);
            }
            AppEvent::AdjustInputGain(delta) => {
                self.adjust_input_gain_db(*delta);
            }
            AppEvent::ResetInputGain => {
                self.reset_input_gain_db();
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
                    AudioFormat::DecentSamplerAndSfz,
                    AudioFormat::All,
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
                    self.audition_midi_conn = None;
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

            AppEvent::AuditionNoteOn(note) => {
                let note = *note;
                if self.demo_mode || self.is_busy() || note > 127 { return; }
                if let Some(previous) = self.audition_note.take() {
                    if let Some(conn) = &mut self.audition_midi_conn { let _ = batcherbird_core::midi::MidiManager::send_note_off(conn, 0, previous, 0); }
                }
                self.audition_note = Some(note);
                if self.audition_midi_conn.is_none() {
                    if let Ok(mut mgr) = batcherbird_core::midi::MidiManager::new() {
                        if let Ok(conn) = mgr.connect_output(self.selected_midi_device) {
                            self.audition_midi_conn = Some(AuditionConnection { connection: conn, note: None });
                        }
                    }
                }
                if let Some(conn) = &mut self.audition_midi_conn {
                    conn.note = Some(note);
                    if let Err(error) = batcherbird_core::midi::MidiManager::send_note_on(conn, 0, note, 100) { self.error_message = Some(error.to_string()); }
                }
            }
            AppEvent::AuditionNoteOff => {
                if let Some(note) = self.audition_note.take() {
                    if let Some(conn) = &mut self.audition_midi_conn {
                        let _ = batcherbird_core::midi::MidiManager::send_note_off(conn, 0, note, 0);
                        conn.note = None;
                    }
                }
            }

            AppEvent::TogglePlaythrough => {
                if self.is_busy() || self.demo_mode { return; }
                let new_val = !self.playthrough_enabled;
                self.set_playthrough(new_val);
            }
            AppEvent::SetPlaythrough(enabled) => {
                if self.is_busy() || self.demo_mode { return; }
                self.set_playthrough(*enabled);
            }

            AppEvent::IncrementStartNote => {
                if self.start_note < 127 && self.start_note < self.end_note {
                    self.start_note += 1;
                    self.update_summary();
                }
            }
            AppEvent::DecrementStartNote => {
                if self.start_note > 0 {
                    self.start_note -= 1;
                    self.update_summary();
                }
            }
            AppEvent::IncrementEndNote => {
                if self.end_note < 127 {
                    self.end_note += 1;
                    self.update_summary();
                }
            }
            AppEvent::DecrementEndNote => {
                if self.end_note > 0 && self.end_note > self.start_note {
                    self.end_note -= 1;
                    self.update_summary();
                }
            }
            AppEvent::IncrementVelocityLayers => {
                if self.velocity_layers < 4 {
                    self.velocity_layers += 1;
                    self.update_summary();
                }
            }
            AppEvent::DecrementVelocityLayers => {
                if self.velocity_layers > 1 {
                    self.velocity_layers -= 1;
                    self.update_summary();
                }
            }
            AppEvent::IncrementDuration => {
                if self.note_duration_ms < 10000 {
                    self.note_duration_ms = (self.note_duration_ms + 500).min(10000);
                    self.update_summary();
                }
            }
            AppEvent::DecrementDuration => {
                if self.note_duration_ms > 500 {
                    self.note_duration_ms = self.note_duration_ms.saturating_sub(500).max(500);
                    self.update_summary();
                }
            }

            AppEvent::Arm => {
                if self.app_state == AppState::Idle && !self.is_busy() {
                    if self.midi_devices.is_empty() || self.audio_input_devices.is_empty() {
                        self.error_message = Some("Connect an audio input and MIDI output, then refresh devices.".into());
                        return;
                    }
                    self.is_testing_note = false;
                    self.gain_check_message = None;
                    let config = self.build_sampling_config();
                    match SamplingEngine::new(config) {
                        Ok(engine) => match engine.start_monitoring_stream_with_playthrough(self.playthrough_enabled) {
                            Ok((input_stream, output_stream)) => {
                                if self.playthrough_enabled && output_stream.is_none() {
                                    self.playthrough_enabled = false;
                                    self.error_message = Some("Software monitoring is unavailable for this output configuration. Use your interface's direct monitoring.".into());
                                }
                                self.monitoring_stream = Some(input_stream);
                                self.playthrough_stream = output_stream;
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
                    if self.export_in_progress || self.session_busy { return; }
                    self.stop_preview();
                    self.test_generation += 1;
                    self.test_worker = None;
                    self.is_testing_note = false;
                    self.gain_check_message = None;
                    self.disarm_monitoring();
                }
            }
            AppEvent::Panic => {
                self.silence_audition();
                self.request_stop();
                self.test_generation += 1;
                self.test_worker = None;
                self.is_testing_note = false;
                if !matches!(self.app_state, AppState::Recording | AppState::Stopping) && !self.demo_mode {
                    let result = (|| {
                        let mut manager = batcherbird_core::midi::MidiManager::new()?;
                        let mut connection = manager.connect_output(self.selected_midi_device)?;
                        batcherbird_core::midi::MidiManager::send_midi_panic(&mut connection)
                    })();
                    if let Err(error) = result { self.error_message = Some(format!("MIDI panic: {error}")); }
                }
                self.info_message = Some("MIDI note cleanup requested.".into());
            }
            AppEvent::PlayTestNote => { self.begin_test_note(cx); }
            AppEvent::TestNoteResult { generation, peak_db, peak_linear } => {
                if *generation != self.test_generation { return; }
                self.test_worker = None;
                self.is_testing_note = false;
                let (msg, color) = Self::evaluate_gain_staging(*peak_db, *peak_linear);
                self.gain_check_message = Some(msg);
                self.gain_check_status_color = color.to_string();
            }
            AppEvent::TestNoteError { generation, message } => {
                if *generation != self.test_generation { return; }
                self.test_worker = None;
                self.is_testing_note = false;
                self.gain_check_message = Some(message.clone());
                self.gain_check_status_color = "#ff4444".to_string();
            }
            AppEvent::StartRecording => { self.begin_recording(cx, None); }
            AppEvent::RecordSelectedSample => { self.begin_recording(cx, Some(self.selected_sample)); }
            AppEvent::PushVizChunk(chunk) => {
                self.viz_peaks.push(chunk.peak);
                self.viz_chunks.push(chunk.clone());
            }
            AppEvent::CancelRecording => { self.request_stop(); }
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
                let was_stopped = self.app_state == AppState::Stopping;
                self.accept_captured_samples(samples);
                self.recording_worker = None;
                self.cancel_flag = None;
                self.recording_meter_state = None;
                if let Ok(mut slot) = self.worker_meter_slot.lock() { *slot = None; }
                self.app_state = if self.recorded_samples.is_empty() { AppState::Idle } else { AppState::Review };
                self.info_message = Some(if was_stopped {
                    format!("Recording stopped. {} completed samples kept.", self.recorded_count)
                } else { format!("{} samples recorded. Select a sample to audition it.", self.recorded_count) });
                self.save_recovery(cx);
            }
            AppEvent::PlayPreview => {
                if !self.is_busy() && !self.recorded_samples.is_empty() {
                    self.start_preview(self.selected_sample);
                }
            }
            AppEvent::PlaySample(idx) => {
                if !self.is_busy() { self.select_sample(*idx); self.start_preview(self.selected_sample); }
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
                self.app_state = if self.recorded_samples.is_empty() { AppState::Idle } else { AppState::Review };
                self.recording_worker = None;
                let partial = self.recorded_slot.lock().map(|mut slot| std::mem::take(&mut *slot)).unwrap_or_default();
                if self.replacement_sample.take().is_none() && !partial.is_empty() {
                    self.set_recorded_samples(partial);
                    self.mark_changed();
                    self.app_state = AppState::Review;
                    self.save_recovery(cx);
                } else { self.select_sample(self.selected_sample); }
                self.recording_meter_state = None;
                if let Ok(mut slot) = self.worker_meter_slot.lock() { *slot = None; }
                self.error_message = Some(format!("{} Completed samples have been kept.", message));
                self.info_message = None;
                self.cancel_flag = None;
            }
            AppEvent::ExportAll => {
                if self.is_busy() { return; }
                if self.recorded_samples.is_empty() {
                    self.error_message =
                        Some("No recorded samples to export.".to_string());
                    return;
                }
                self.error_message = None;
                self.info_message = None;

                self.export_in_progress = true;
                let cfg = self.build_export_config();
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
                self.export_in_progress = false;
                self.error_message = None;
                self.info_message = Some(format!(
                    "Exported {} file(s) to {}",
                    count,
                    directory.display()
                ));
            }
            AppEvent::ExportError(msg) => {
                self.export_in_progress = false;
                self.error_message = Some(msg.clone());
                self.info_message = None;
            }
            AppEvent::DismissError => {
                self.error_message = None;
                self.info_message = None;
            }

            AppEvent::SelectOutputDirectory => {
                if self.is_busy() { return; }
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
            }
            self.selected_format_index = match self.export_format { AudioFormat::Wav16Bit => 0, AudioFormat::Wav24Bit => 1,
                AudioFormat::Wav32BitFloat => 2, AudioFormat::DecentSampler => 3, AudioFormat::SFZ => 4,
                AudioFormat::DecentSamplerAndSfz => 5, AudioFormat::All => 6 };
            self.controls_busy = self.is_busy();
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
    fn stopping_waits_for_current_worker_and_preserves_generation() {
        let flag = Arc::new(AtomicBool::new(false));
        let mut data = AppData {
            recording_generation: 4,
            app_state: AppState::Recording,
            cancel_flag: Some(flag.clone()),
            ..AppData::default()
        };
        data.request_stop();
        assert_eq!(data.app_state, AppState::Stopping);
        assert!(flag.load(Ordering::Acquire));
        assert!(data.is_current_recording(4));
        assert!(data.is_busy());
    }

    #[test]
    fn recording_worker_drop_cancels_and_waits_for_cleanup() {
        let cancel = Arc::new(AtomicBool::new(false));
        let cleaned = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let worker_cleaned = cleaned.clone();
        let handle = std::thread::spawn(move || {
            while !worker_cancel.load(Ordering::Acquire) {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            worker_cleaned.store(true, Ordering::Release);
        });
        drop(RecordingWorker {
            cancel: cancel.clone(),
            handle: Some(handle),
        });
        assert!(cancel.load(Ordering::Acquire));
        assert!(cleaned.load(Ordering::Acquire));
    }

    #[test]
    fn instrument_filename_cannot_escape_export_folder() {
        assert_eq!(safe_instrument_name("../../"), "Untitled instrument");
        assert_eq!(safe_instrument_name(" DW6000 / Pad "), "DW6000 _ Pad");
    }

    fn confirmation_sample(note: u8, value: f32) -> Sample {
        Sample {
            note,
            velocity: 96,
            audio_data: vec![value, -value],
            sample_rate: 48000,
            channels: 1,
            recorded_at: std::time::SystemTime::UNIX_EPOCH,
            midi_timing: std::time::Duration::ZERO,
            audio_timing: std::time::Duration::ZERO,
        }
    }

    fn edited_confirmation_session() -> AppData {
        let mut data = AppData {
            instrument_name: "Edited current instrument".into(),
            release_duration_ms: 1700,
            session_status: "current.batcherbird".into(),
            app_state: AppState::Review,
            ..AppData::default()
        };
        data.set_recorded_samples(vec![confirmation_sample(60, 0.25)]);
        data.mark_changed();
        data
    }

    #[test]
    fn cancel_open_replacement_retains_edited_settings_and_original_audio() {
        let mut data = edited_confirmation_session();
        let settings = serde_json::to_value(data.session_settings()).unwrap();
        let revision = data.settings_revision;
        data.receive_loaded_session(
            PathBuf::from("candidate.batcherbird"),
            SessionSettings::default(),
            vec![confirmation_sample(72, 0.75)],
        );
        assert!(data.is_busy());
        assert_eq!(
            data.pending_session_name.as_deref(),
            Some("candidate.batcherbird")
        );
        assert_eq!(data.recorded_samples[0].audio_data, vec![0.25, -0.25]);
        data.resolve_session_replacement(false);
        assert!(!data.is_busy());
        assert!(data.pending_session_name.is_none());
        assert!(data.pending_session.is_none());
        assert_eq!(
            serde_json::to_value(data.session_settings()).unwrap(),
            settings
        );
        assert_eq!(data.settings_revision, revision);
        assert_eq!(data.recorded_samples[0].note, 60);
        assert_eq!(data.recorded_samples[0].audio_data, vec![0.25, -0.25]);
        assert_eq!(data.session_status, "current.batcherbird");
        assert!(data.has_unsaved_changes);
    }

    #[test]
    fn confirm_open_replacement_applies_only_the_validated_candidate() {
        let mut data = edited_confirmation_session();
        let settings = SessionSettings {
            instrument_name: "New candidate".into(),
            release_duration_ms: 3000,
            ..SessionSettings::default()
        };
        data.receive_loaded_session(
            PathBuf::from("candidate.batcherbird"),
            settings,
            vec![confirmation_sample(72, 0.75)],
        );
        assert_eq!(data.instrument_name, "Edited current instrument");
        data.resolve_session_replacement(true);
        assert!(!data.is_busy());
        assert!(data.pending_session_name.is_none());
        assert!(data.pending_session.is_none());
        assert_eq!(data.instrument_name, "New candidate");
        assert_eq!(data.release_duration_ms, 3000);
        assert_eq!(data.recorded_samples.len(), 1);
        assert_eq!(data.recorded_samples[0].note, 72);
        assert_eq!(data.recorded_samples[0].audio_data, vec![0.75, -0.75]);
        assert_eq!(data.session_status, "candidate.batcherbird");
        assert_eq!(data.app_state, AppState::Review);
        assert!(!data.has_unsaved_changes);
    }

    #[test]
    fn replacement_response_without_a_candidate_does_not_interrupt_another_operation() {
        let mut data = AppData {
            session_busy: true,
            ..AppData::default()
        };
        data.resolve_session_replacement(false);
        assert!(data.session_busy);
        data.resolve_session_replacement(true);
        assert!(data.session_busy);
    }
    fn dispatch_session_events(
        data: AppData,
        events: &[AppEvent],
    ) -> vizia::backend::BackendContext {
        let mut context = Context::default();
        data.build(&mut context);
        let mut backend = vizia::backend::BackendContext::new(context);
        let mut manager = vizia::events::EventManager::new();
        for event in events {
            backend.send_event(Event::new(event.clone()));
            manager.flush_events(&mut backend.0, |_| {});
        }
        backend
    }

    #[test]
    fn failed_open_or_save_and_cancelled_dialog_preserve_the_edited_session() {
        for event in [
            AppEvent::SessionError(
                "Cannot access missing audio; keep the sidecar folder together.".into(),
            ),
            AppEvent::SessionDialogCancelled,
        ] {
            let mut data = edited_confirmation_session();
            data.session_busy = true;
            data.info_message = Some("Previous save succeeded".into());
            let settings = serde_json::to_value(data.session_settings()).unwrap();
            let revision = data.settings_revision;
            let backend = dispatch_session_events(data, std::slice::from_ref(&event));
            let data = backend.0.data::<AppData>().unwrap();
            assert!(!data.session_busy && !data.controls_busy);
            assert_eq!(
                serde_json::to_value(data.session_settings()).unwrap(),
                settings
            );
            assert_eq!(data.settings_revision, revision);
            assert!(data.has_unsaved_changes);
            assert_eq!(data.session_status, "current.batcherbird");
            assert_eq!(data.recorded_samples[0].audio_data, vec![0.25, -0.25]);
            if matches!(event, AppEvent::SessionError(_)) {
                assert!(data
                    .error_message
                    .as_ref()
                    .unwrap()
                    .contains("missing audio"));
                assert!(data.info_message.is_none());
            }
        }
    }

    #[test]
    fn failed_session_operation_can_be_retried_and_saved_without_stale_error() {
        let mut data = edited_confirmation_session();
        data.session_busy = true;
        let revision = data.settings_revision;
        let backend = dispatch_session_events(
            data,
            &[
                AppEvent::SessionError("Disk full".into()),
                AppEvent::SessionSaved {
                    path: PathBuf::from("retry.batcherbird"),
                    revision,
                },
            ],
        );
        let data = backend.0.data::<AppData>().unwrap();
        assert!(!data.has_unsaved_changes && !data.controls_busy);
        assert!(data.error_message.is_none());
        assert_eq!(data.session_status, "retry.batcherbird");
        assert_eq!(data.recorded_samples[0].audio_data, vec![0.25, -0.25]);
    }
}
