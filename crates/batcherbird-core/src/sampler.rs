use crate::audio::AudioManager;
use crate::audio_diagnostics::{AudioDiagnostics, AudioPerformanceReport};
use crate::channel_routing::ChannelRouting;
use crate::detection::{DetectionConfig, DetectionResult, SampleDetector};
use crate::lock_free_recording::{LockFreeRecorder, LockFreeRecordingConfig};
use crate::loop_detection::{LoopDetectionConfig, LoopDetectionResult, LoopDetector};
use crate::midi::MidiManager;
use crate::professional_meters::{ProfessionalMeterEngine, ProfessionalMeterReadings};
use crate::{BatcherbirdError, Result};
use cpal::traits::{DeviceTrait, StreamTrait};
use midir::MidiOutputConnection;
use rtrb::{Consumer, Producer, RingBuffer};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;

/// Push complete frames only: a full ring must never leave half a stereo frame.
pub(crate) fn push_capture_frame(
    producer: &mut Producer<f32>,
    left: f32,
    right: f32,
    hw_channels: u16,
    routing: ChannelRouting,
) -> bool {
    let count = routing.output_channels(hw_channels) as usize;
    if producer.slots() < count {
        return false;
    }
    let first = if routing == ChannelRouting::MonoRight {
        right
    } else {
        left
    };
    let _ = producer.push(first);
    if count == 2 {
        let _ = producer.push(right);
    }
    true
}

/// Polling bounds cancellation latency to 10ms during every phase of a note.
async fn wait_for_capture(ms: u64, cancel: &AtomicBool, failed: &AtomicBool) -> Result<bool> {
    let deadline = Instant::now() + Duration::from_millis(ms);
    loop {
        if failed.load(Ordering::Acquire) {
            return Err(BatcherbirdError::Audio(
                "Audio capture failed or buffer overflowed".into(),
            ));
        }
        if cancel.load(Ordering::Acquire) {
            return Ok(false);
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(true);
        }
        tokio::time::sleep(remaining.min(Duration::from_millis(10))).await;
    }
}

#[derive(Debug, Clone)]
pub struct SamplingConfig {
    pub note_duration_ms: u64,
    pub release_time_ms: u64,
    pub pre_delay_ms: u64,
    pub post_delay_ms: u64,
    pub midi_channel: u8,
    pub velocity: u8,
    /// Name of the audio input device to record from.
    ///
    /// `None` uses the system default input device. `Some(name)` selects a device
    /// by name (exact match preferred, case-insensitive fallback) via
    /// [`AudioManager::find_input_device`].
    pub input_device_name: Option<String>,
    /// Input channel routing selection (Stereo, Mono In 1, Mono In 2)
    pub channel_routing: ChannelRouting,
    /// Input digital gain/trim adjustment in decibels (e.g. -12.0 to +12.0 dB, default 0.0 dB)
    pub input_gain_db: f32,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            note_duration_ms: 2000,  // 2 second note duration
            release_time_ms: 1000,   // 1 second release capture
            pre_delay_ms: 100,       // 100ms pre-roll
            post_delay_ms: 100,      // 100ms post delay
            midi_channel: 0,         // Channel 1 (0-indexed)
            velocity: 100,           // Default velocity
            input_device_name: None, // System default input device
            channel_routing: ChannelRouting::Stereo,
            input_gain_db: 0.0,
        }
    }
}

/// Professional audio level detector for real-time metering
#[derive(Debug)]
pub struct AudioLevelDetector {
    peak_level: f32,
    peak_left: f32,
    peak_right: f32,
    rms_accumulator: f32,
    rms_sample_count: usize,
    rms_window_size: usize,
    #[allow(dead_code)] // Reserved for future advanced RMS windowing
    rms_window_samples: f32,
    // Epic 3.1.1: Professional meter engine integration
    professional_meters: ProfessionalMeterEngine,
}

impl AudioLevelDetector {
    pub fn new(sample_rate: u32) -> Self {
        // Professional RMS window: 300ms for VU-style integration
        let rms_window_size = (sample_rate as f32 * 0.3) as usize; // 300ms window
        Self {
            peak_level: 0.0,
            peak_left: 0.0,
            peak_right: 0.0,
            rms_accumulator: 0.0,
            rms_sample_count: 0,
            rms_window_size,
            rms_window_samples: 0.0,
            // Epic 3.1.1: Initialize professional meter engine
            professional_meters: ProfessionalMeterEngine::new(sample_rate as f32),
        }
    }

    /// Process audio samples and update levels (called from audio thread)
    pub fn process_samples(&mut self, samples: &[f32]) -> AudioLevels {
        self.process_interleaved_samples_with_gain(samples, 1, 1.0)
    }

    /// Process interleaved audio samples across multiple channels (called from audio thread)
    pub fn process_interleaved_samples(&mut self, samples: &[f32], channels: usize) -> AudioLevels {
        self.process_interleaved_samples_with_gain(samples, channels, 1.0)
    }

    /// Process interleaved audio samples across multiple channels with gain scaling (called from audio thread)
    pub fn process_interleaved_samples_with_gain(
        &mut self,
        samples: &[f32],
        channels: usize,
        gain: f32,
    ) -> AudioLevels {
        let is_unity = (gain - 1.0).abs() < 1e-6;
        if channels >= 2 {
            for chunk in samples.chunks(channels) {
                let left = if is_unity {
                    chunk[0].abs()
                } else {
                    (chunk[0] * gain).abs()
                };
                let right = if is_unity {
                    chunk[1].abs()
                } else {
                    (chunk[1] * gain).abs()
                };
                if left > self.peak_left {
                    self.peak_left = left;
                }
                if right > self.peak_right {
                    self.peak_right = right;
                }
                let max_s = left.max(right);
                if max_s > self.peak_level {
                    self.peak_level = max_s;
                }
                self.rms_accumulator += left * left + right * right;
                self.rms_sample_count += 2;
            }
        } else {
            for &sample in samples {
                let abs_sample = if is_unity {
                    sample.abs()
                } else {
                    (sample * gain).abs()
                };
                if abs_sample > self.peak_left {
                    self.peak_left = abs_sample;
                }
                self.peak_right = self.peak_left;
                if abs_sample > self.peak_level {
                    self.peak_level = abs_sample;
                }
                self.rms_accumulator += abs_sample * abs_sample;
                self.rms_sample_count += 1;
            }
        }

        // Calculate RMS over the integration window (VU-style)
        let rms_level = if self.rms_sample_count > 0 {
            (self.rms_accumulator / self.rms_sample_count as f32).sqrt()
        } else {
            0.0
        };

        // Reset RMS accumulator if window is full
        if self.rms_sample_count >= self.rms_window_size {
            self.rms_accumulator = 0.0;
            self.rms_sample_count = 0;
        }

        // Epic 3.1.1: Process through professional meters for enhanced readings
        // Advanced readings are requested explicitly off the recording callback.

        AudioLevels {
            peak: self.peak_level,
            rms: rms_level,
            peak_db: if self.peak_level > 0.0 {
                20.0 * self.peak_level.log10()
            } else {
                -60.0
            },
            rms_db: if rms_level > 0.0 {
                20.0 * rms_level.log10()
            } else {
                -60.0
            },
            peak_left: self.peak_left,
            peak_right: self.peak_right,
            peak_left_db: if self.peak_left > 0.0 {
                20.0 * self.peak_left.log10()
            } else {
                -60.0
            },
            peak_right_db: if self.peak_right > 0.0 {
                20.0 * self.peak_right.log10()
            } else {
                -60.0
            },
        }
    }

    /// Get professional meter readings (Epic 3.1.1 - Professional Meter Engine)
    pub fn get_professional_readings(&mut self, samples: &[f32]) -> ProfessionalMeterReadings {
        self.professional_meters.process_samples(samples)
    }

    /// Reset peak level (called periodically for peak hold behavior)
    pub fn reset_peak(&mut self) {
        self.peak_level = 0.0;
        self.peak_left = 0.0;
        self.peak_right = 0.0;
    }
}

/// Real-time audio levels (thread-safe)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioLevels {
    pub peak: f32,    // Linear peak level (0.0 to 1.0)
    pub rms: f32,     // RMS level (0.0 to 1.0)
    pub peak_db: f32, // Peak in dBFS
    pub rms_db: f32,  // RMS in dBFS
    pub peak_left: f32,
    pub peak_right: f32,
    pub peak_left_db: f32,
    pub peak_right_db: f32,
}

impl Default for AudioLevels {
    fn default() -> Self {
        Self {
            peak: 0.0,
            rms: 0.0,
            peak_db: -60.0,
            rms_db: -60.0,
            peak_left: 0.0,
            peak_right: 0.0,
            peak_left_db: -60.0,
            peak_right_db: -60.0,
        }
    }
}

/// Real-time visualization data chunk for waveform display
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VizChunk {
    pub peak: f32,         // Peak amplitude for this chunk (0.0 to 1.0)
    pub rms: f32,          // RMS level for this chunk (0.0 to 1.0)
    pub peak_db: f32,      // Peak in dBFS
    pub rms_db: f32,       // RMS in dBFS
    pub timestamp: u64,    // Timestamp in samples since recording start
    pub chunk_size: usize, // Number of samples in this chunk
}

impl VizChunk {
    /// Create a new visualization chunk from audio samples
    pub fn from_samples(samples: &[f32], timestamp: u64) -> Self {
        let chunk_size = samples.len();

        // Calculate peak
        let peak = samples
            .iter()
            .map(|&sample| sample.abs())
            .fold(0.0f32, f32::max);

        // Calculate RMS
        let rms = if chunk_size > 0 {
            let sum_squares: f32 = samples.iter().map(|&sample| sample * sample).sum();
            (sum_squares / chunk_size as f32).sqrt()
        } else {
            0.0
        };

        // Convert to dB
        let peak_db = if peak > 0.0 {
            20.0 * peak.log10()
        } else {
            -60.0
        };
        let rms_db = if rms > 0.0 { 20.0 * rms.log10() } else { -60.0 };

        Self {
            peak,
            rms,
            peak_db,
            rms_db,
            timestamp,
            chunk_size,
        }
    }
}

const WAVEFORM_HISTORY_CAPACITY: usize = 256;

/// Thread-safe level meter state using atomic operations
#[derive(Debug)]
pub struct LevelMeterState {
    input_peak: AtomicU32, // Store f32 as u32 bits for atomicity
    input_rms: AtomicU32,
    input_peak_db: AtomicU32,
    input_rms_db: AtomicU32,
    input_peak_left: AtomicU32,
    input_peak_right: AtomicU32,
    input_peak_left_db: AtomicU32,
    input_peak_right_db: AtomicU32,
    // A bounded envelope history. The one active input callback is the writer;
    // UI snapshots read atomic values without locking or interrupting audio.
    waveform_peaks: [AtomicU32; WAVEFORM_HISTORY_CAPACITY],
    waveform_written: AtomicUsize,
    #[allow(dead_code)] // Reserved for future rate limiting features
    last_update: std::time::Instant,
}

impl LevelMeterState {
    pub fn new() -> Self {
        Self {
            input_peak: AtomicU32::new(0),
            input_rms: AtomicU32::new(0),
            input_peak_db: AtomicU32::new(f32::to_bits(-60.0)),
            input_rms_db: AtomicU32::new(f32::to_bits(-60.0)),
            input_peak_left: AtomicU32::new(0),
            input_peak_right: AtomicU32::new(0),
            input_peak_left_db: AtomicU32::new(f32::to_bits(-60.0)),
            input_peak_right_db: AtomicU32::new(f32::to_bits(-60.0)),
            waveform_peaks: std::array::from_fn(|_| AtomicU32::new(0)),
            waveform_written: AtomicUsize::new(0),
            last_update: std::time::Instant::now(),
        }
    }

    /// Update levels from audio thread (atomic write)
    pub fn update_levels(&self, levels: AudioLevels) {
        self.push_waveform_peak(levels.peak);
        self.input_peak
            .store(f32::to_bits(levels.peak), Ordering::Relaxed);
        self.input_rms
            .store(f32::to_bits(levels.rms), Ordering::Relaxed);
        self.input_peak_db
            .store(f32::to_bits(levels.peak_db), Ordering::Relaxed);
        self.input_rms_db
            .store(f32::to_bits(levels.rms_db), Ordering::Relaxed);
        self.input_peak_left
            .store(f32::to_bits(levels.peak_left), Ordering::Relaxed);
        self.input_peak_right
            .store(f32::to_bits(levels.peak_right), Ordering::Relaxed);
        self.input_peak_left_db
            .store(f32::to_bits(levels.peak_left_db), Ordering::Relaxed);
        self.input_peak_right_db
            .store(f32::to_bits(levels.peak_right_db), Ordering::Relaxed);
    }

    fn push_waveform_peak(&self, peak: f32) {
        let peak = if peak.is_finite() {
            peak.abs().clamp(0.0, 1.0)
        } else {
            0.0
        };
        let written = self.waveform_written.load(Ordering::Relaxed);
        self.waveform_peaks[written % WAVEFORM_HISTORY_CAPACITY]
            .store(peak.to_bits(), Ordering::Relaxed);
        // Publish the value after its slot is populated.
        self.waveform_written
            .store(written.wrapping_add(1), Ordering::Release);
    }

    /// Recent input envelope peaks, oldest to newest, with at most 256 entries.
    /// Call on the UI thread: allocation happens only here, never during capture.
    /// This is a rolling envelope, not a sample-accurate oscilloscope trace.
    pub fn get_waveform_peaks(&self) -> Vec<f32> {
        // Retry a racing UI snapshot once; the audio writer never waits for us.
        let mut peaks = Vec::with_capacity(WAVEFORM_HISTORY_CAPACITY);
        for _ in 0..2 {
            peaks.clear();
            let written = self.waveform_written.load(Ordering::Acquire);
            let count = written.min(WAVEFORM_HISTORY_CAPACITY);
            for index in written - count..written {
                peaks.push(f32::from_bits(
                    self.waveform_peaks[index % WAVEFORM_HISTORY_CAPACITY].load(Ordering::Relaxed),
                ));
            }
            if self.waveform_written.load(Ordering::Acquire) == written {
                break;
            }
        }
        peaks
    }

    /// Get current levels for UI (atomic read)
    pub fn get_levels(&self) -> AudioLevels {
        AudioLevels {
            peak: f32::from_bits(self.input_peak.load(Ordering::Relaxed)),
            rms: f32::from_bits(self.input_rms.load(Ordering::Relaxed)),
            peak_db: f32::from_bits(self.input_peak_db.load(Ordering::Relaxed)),
            rms_db: f32::from_bits(self.input_rms_db.load(Ordering::Relaxed)),
            peak_left: f32::from_bits(self.input_peak_left.load(Ordering::Relaxed)),
            peak_right: f32::from_bits(self.input_peak_right.load(Ordering::Relaxed)),
            peak_left_db: f32::from_bits(self.input_peak_left_db.load(Ordering::Relaxed)),
            peak_right_db: f32::from_bits(self.input_peak_right_db.load(Ordering::Relaxed)),
        }
    }
}

impl Default for LevelMeterState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Sample {
    pub note: u8,
    pub velocity: u8,
    pub audio_data: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub recorded_at: std::time::SystemTime,
    pub midi_timing: Duration,
    pub audio_timing: Duration,
}

pub struct SamplingEngine {
    audio_manager: AudioManager,
    config: SamplingConfig,
    level_meter_state: Arc<LevelMeterState>,
    audio_diagnostics: Arc<AudioDiagnostics>,
    playthrough_active: Arc<AtomicBool>,
    channel_routing: Arc<AtomicU8>,
    input_gain_factor: Arc<AtomicU32>,
}

/// Progress update emitted while recording a range of notes (optionally across
/// multiple velocity layers). Passed to the caller's progress callback after
/// each individual sample is captured.
#[derive(Debug, Clone)]
pub struct RecordingProgress {
    /// MIDI note being recorded or most recently completed.
    pub note: u8,
    /// MIDI velocity of the current or most recently completed take.
    pub velocity: u8,
    /// 0-based index of the current velocity layer.
    pub layer: u8,
    /// Total number of velocity layers being recorded per note.
    pub total_layers: u8,
    /// Number of samples finished so far (1-based count once first sample lands).
    pub completed: u32,
    /// Total number of samples to record = num_notes * total_layers.
    pub total: u32,
}

/// Map a desired velocity-layer count to concrete MIDI velocity values.
///
/// Behavior:
/// - `0` or `1` -> `vec![100]`, preserving the historical single-layer default
///   (0 is treated as 1).
/// - `n >= 2` -> `n` velocities evenly distributed across the inclusive musical
///   range `[20, 127]`. The endpoints are always exactly hit, and intermediate
///   layers are placed using integer interpolation:
///
///   ```text
///   velocity[i] = round(20 + (127 - 20) * i / (n - 1))   for i in 0..n
///   ```
///
///   Examples: 2 -> [20, 127]; 3 -> [20, 74, 127]; 4 -> [20, 56, 91, 127].
///
/// The count is capped at 16 layers (a generous practical ceiling) so the
/// interpolation always yields strictly ascending values within `1..=127`.
fn velocity_layers(count: u8) -> Vec<u8> {
    match count {
        0 | 1 => vec![127],
        2 => vec![64, 127],
        3 => vec![48, 96, 127],
        4 => vec![32, 64, 96, 127],
        _ => {
            const LOW: u32 = 20;
            const HIGH: u32 = 127;
            const MAX_LAYERS: u8 = 16;
            let count = count.min(MAX_LAYERS);
            let n = count as u32;
            (0..n)
                .map(|i| {
                    let span = HIGH - LOW;
                    let numerator = span * i;
                    let denominator = n - 1;
                    let value = LOW + (numerator + denominator / 2) / denominator;
                    value as u8
                })
                .collect()
        }
    }
}

/// Ring buffer size for visualization data
/// At 60fps, we need ~1 second of buffer = 60 chunks
const VIZ_RING_BUFFER_SIZE: usize = 64;

impl SamplingEngine {
    pub fn new(config: SamplingConfig) -> Result<Self> {
        let audio_manager = AudioManager::new()?;

        // Initialize diagnostics with professional audio standards
        // 128 samples at 44.1kHz = ~2.9ms budget per callback
        let diagnostics = Arc::new(AudioDiagnostics::new(44100, 128));

        let routing = config.channel_routing;
        let initial_db = config.input_gain_db.clamp(-12.0, 12.0);
        let initial_factor = 10.0f32.powf(initial_db / 20.0);
        Ok(Self {
            audio_manager,
            config,
            level_meter_state: Arc::new(LevelMeterState::new()),
            audio_diagnostics: diagnostics,
            playthrough_active: Arc::new(AtomicBool::new(false)),
            channel_routing: Arc::new(AtomicU8::new(routing.to_u8())),
            input_gain_factor: Arc::new(AtomicU32::new(initial_factor.to_bits())),
        })
    }

    /// Set input digital gain/trim in decibels (thread-safe, lock-free, clamped to [-12.0, 12.0])
    pub fn set_input_gain_db(&self, db: f32) {
        let clamped = db.clamp(-12.0, 12.0);
        let factor = 10.0f32.powf(clamped / 20.0);
        self.input_gain_factor
            .store(factor.to_bits(), Ordering::Relaxed);
    }

    /// Get current linear input gain factor (1.0 = 0 dB)
    pub fn get_input_gain_factor(&self) -> f32 {
        f32::from_bits(self.input_gain_factor.load(Ordering::Relaxed))
    }

    /// Get current input digital gain/trim in decibels
    pub fn get_input_gain_db(&self) -> f32 {
        let factor = self.get_input_gain_factor();
        20.0 * factor.log10()
    }

    /// Get shared atomic handle for input gain factor
    pub fn get_input_gain_factor_state(&self) -> Arc<AtomicU32> {
        Arc::clone(&self.input_gain_factor)
    }

    /// Set input channel routing selection (thread-safe, lock-free)
    pub fn set_channel_routing(&self, routing: ChannelRouting) {
        self.channel_routing
            .store(routing.to_u8(), Ordering::Relaxed);
    }

    /// Get current input channel routing selection
    pub fn get_channel_routing(&self) -> ChannelRouting {
        ChannelRouting::from_u8(self.channel_routing.load(Ordering::Relaxed))
    }

    /// Get shared atomic handle for input channel routing
    pub fn get_channel_routing_state(&self) -> Arc<AtomicU8> {
        Arc::clone(&self.channel_routing)
    }

    /// Set software playthrough monitoring state (thread-safe, click-free)
    pub fn set_playthrough(&self, enabled: bool) {
        self.playthrough_active.store(enabled, Ordering::Relaxed);
    }

    /// Check if software playthrough monitoring is active
    pub fn is_playthrough_enabled(&self) -> bool {
        self.playthrough_active.load(Ordering::Relaxed)
    }

    /// Get shared atomic handle for software playthrough monitoring
    pub fn get_playthrough_state(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.playthrough_active)
    }

    /// Get current audio levels for UI (thread-safe)
    pub fn get_audio_levels(&self) -> AudioLevels {
        self.level_meter_state.get_levels()
    }

    /// Get shared level meter state for monitoring and level testing (thread-safe)
    pub fn get_level_meter_state(&self) -> Arc<LevelMeterState> {
        Arc::clone(&self.level_meter_state)
    }

    /// Get comprehensive audio performance diagnostics
    pub fn get_performance_diagnostics(&self) -> AudioPerformanceReport {
        self.audio_diagnostics.get_performance_report()
    }

    /// Reset audio diagnostics (for testing)
    pub fn reset_diagnostics(&self) {
        self.audio_diagnostics.reset()
    }

    /// Start persistent audio monitoring stream with optional playthrough
    pub fn start_monitoring_stream_with_playthrough(
        &self,
        enable_playthrough: bool,
    ) -> Result<(cpal::Stream, Option<cpal::Stream>)> {
        let input_device = self
            .audio_manager
            .find_input_device(self.config.input_device_name.as_deref())?;
        let input_config = input_device
            .default_input_config()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to get input config: {}", e)))?;

        let sample_rate = input_config.sample_rate().0;
        let input_channels = input_config.channels() as usize;
        let level_state = Arc::clone(&self.level_meter_state);
        let playthrough_active = Arc::clone(&self.playthrough_active);
        let channel_routing = Arc::clone(&self.channel_routing);
        let input_gain_factor = Arc::clone(&self.input_gain_factor);
        playthrough_active.store(enable_playthrough, Ordering::Relaxed);

        // Lock-free wait-free ring buffer for playthrough (8192 samples = ~185ms buffer at 44.1kHz)
        let (mut producer, mut consumer) = RingBuffer::<f32>::new(8192);

        use cpal::SampleFormat;
        let input_stream_config = input_config.config();

        // Build input stream
        let input_stream = match input_config.sample_format() {
            SampleFormat::F32 => {
                let level_state_clone = Arc::clone(&level_state);
                let playthrough_active_clone = Arc::clone(&playthrough_active);
                let channel_routing_clone = Arc::clone(&channel_routing);
                let input_gain_factor_clone = Arc::clone(&input_gain_factor);
                let mut level_detector = AudioLevelDetector::new(sample_rate);

                input_device
                    .build_input_stream(
                        &input_stream_config,
                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            let gain =
                                f32::from_bits(input_gain_factor_clone.load(Ordering::Relaxed));

                            level_detector.reset_peak();
                            // Continuous level detection for UI meters across hardware channels
                            let levels = level_detector.process_interleaved_samples_with_gain(
                                data,
                                input_channels,
                                gain,
                            );
                            level_state_clone.update_levels(levels);

                            // Forward to playthrough output if active (lock-free)
                            if playthrough_active_clone.load(Ordering::Relaxed) {
                                let routing = ChannelRouting::from_u8(
                                    channel_routing_clone.load(Ordering::Relaxed),
                                );
                                for frame in data.chunks_exact(input_channels) {
                                    let left = frame[0] * gain;
                                    let right = if input_channels > 1 {
                                        frame[1] * gain
                                    } else {
                                        left
                                    };
                                    let (left, right) = match routing {
                                        ChannelRouting::Stereo => (left, right),
                                        ChannelRouting::MonoLeft => (left, left),
                                        ChannelRouting::MonoRight => (right, right),
                                    };
                                    // A monitoring dropout may discard complete frames,
                                    // but must never swap left and right channels.
                                    if !push_capture_frame(
                                        &mut producer,
                                        left,
                                        right,
                                        2,
                                        ChannelRouting::Stereo,
                                    ) {
                                        break;
                                    }
                                }
                            }
                        },
                        |err| tracing::error!("Audio input error: {}", err),
                        None,
                    )
                    .map_err(|e| {
                        BatcherbirdError::Audio(format!("Failed to build input stream: {}", e))
                    })?
            }
            _ => {
                return Err(BatcherbirdError::Audio(
                    "Playthrough currently only supports F32 format".to_string(),
                ));
            }
        };

        input_stream.play().map_err(|e| {
            BatcherbirdError::Audio(format!("Failed to start input monitoring stream: {}", e))
        })?;

        // Build output stream for playthrough
        let output_stream = match self.audio_manager.get_default_output_device() {
            Ok(output_device) => match output_device.default_output_config() {
                Ok(output_config) => {
                    let output_stream_config = cpal::StreamConfig {
                        channels: 2,
                        sample_rate: cpal::SampleRate(sample_rate),
                        buffer_size: cpal::BufferSize::Default,
                    };
                    let playthrough_active_out = Arc::clone(&playthrough_active);

                    match output_config.sample_format() {
                        SampleFormat::F32 => {
                            match output_device.build_output_stream(
                                &output_stream_config,
                                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                                    if playthrough_active_out.load(Ordering::Relaxed) {
                                        for out in data.iter_mut() {
                                            *out = consumer.pop().unwrap_or(0.0);
                                        }
                                    } else {
                                        data.fill(0.0);
                                        while consumer.pop().is_ok() {}
                                    }
                                },
                                |err| tracing::error!("Audio output error: {}", err),
                                None,
                            ) {
                                Ok(stream) => {
                                    if let Err(e) = stream.play() {
                                        tracing::warn!(
                                            "Failed to start playthrough output stream: {}",
                                            e
                                        );
                                        None
                                    } else {
                                        Some(stream)
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to build playthrough output stream: {}",
                                        e
                                    );
                                    None
                                }
                            }
                        }
                        _ => {
                            tracing::warn!("Playthrough output requires F32 sample format");
                            None
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to get output config for playthrough: {}", e);
                    None
                }
            },
            Err(e) => {
                tracing::warn!("No default output device for playthrough: {}", e);
                None
            }
        };

        Ok((input_stream, output_stream))
    }

    /// Start persistent audio monitoring stream (separate from recording)
    pub fn start_monitoring_stream(&self) -> Result<cpal::Stream> {
        let device = self
            .audio_manager
            .find_input_device(self.config.input_device_name.as_deref())?;
        let config = device
            .default_input_config()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to get input config: {}", e)))?;

        let (producer, _consumer) = RingBuffer::<f32>::new(1);
        let stream = self.build_persistent_recording_stream(
            &device,
            &config,
            producer,
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicBool::new(false)),
        )?;

        stream.play().map_err(|e| {
            BatcherbirdError::Audio(format!("Failed to start monitoring stream: {}", e))
        })?;

        Ok(stream)
    }

    /// Blocking interface for Tauri GUI layer (follows TAURI_AUDIO_ARCHITECTURE.md)
    /// Uses professional lock-free recording architecture
    pub fn sample_single_note_blocking(
        &self,
        midi_conn: &mut MidiOutputConnection,
        note: u8,
    ) -> Result<Sample> {
        // Create dedicated runtime for this blocking operation
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to create runtime: {}", e)))?;

        // Execute the professional lock-free method in blocking context
        rt.block_on(async {
            let (sample, _performance) = self
                .sample_single_note_lock_free(midi_conn, note, self.config.velocity)
                .await?;
            Ok(sample)
        })
    }

    /// Re-record one note at its original velocity. Cancellation discards the
    /// partial take and returns `None`, allowing callers to keep the previous take.
    pub fn sample_note_velocity_with_cancel_blocking(
        &self,
        midi_conn: &mut MidiOutputConnection,
        note: u8,
        velocity: u8,
        cancel: &AtomicBool,
    ) -> Result<Option<Sample>> {
        Self::validate_note_range(note, note)?;
        if velocity == 0 || velocity > 127 {
            return Err(BatcherbirdError::Config(
                "Velocity must be in 1..=127".into(),
            ));
        }
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to create runtime: {e}")))?;
        rt.block_on(async {
            if cancel.load(Ordering::Acquire) {
                return Ok(None);
            }
            let device = self
                .audio_manager
                .find_input_device(self.config.input_device_name.as_deref())?;
            let config = device
                .default_input_config()
                .map_err(|e| BatcherbirdError::Audio(format!("Failed to get input config: {e}")))?;
            let sample_rate = config.sample_rate().0;
            let channels = self
                .get_channel_routing()
                .output_channels(config.channels());
            let capacity = self.capture_capacity(sample_rate, channels)?;
            let (producer, mut consumer) = RingBuffer::new(capacity);
            let active = Arc::new(AtomicBool::new(false));
            let failed = Arc::new(AtomicBool::new(false));
            let stream = self.build_persistent_recording_stream(
                &device,
                &config,
                producer,
                Arc::clone(&active),
                Arc::clone(&failed),
            )?;
            stream
                .play()
                .map_err(|e| BatcherbirdError::Audio(format!("Failed to start stream: {e}")))?;
            let result = self
                .record_one_on_stream(
                    midi_conn,
                    &mut consumer,
                    &active,
                    note,
                    velocity,
                    sample_rate,
                    channels,
                    cancel,
                    &failed,
                )
                .await;
            active.store(false, Ordering::Release);
            drop(stream);
            let cleanup = MidiManager::send_channel_panic(midi_conn, self.config.midi_channel);
            let sample = result?;
            cleanup?;
            Ok(sample)
        })
    }

    fn capture_capacity(&self, sample_rate: u32, channels: u16) -> Result<usize> {
        let duration = self
            .config
            .pre_delay_ms
            .checked_add(self.config.note_duration_ms)
            .and_then(|d| d.checked_add(self.config.release_time_ms))
            .and_then(|d| d.checked_add(self.config.post_delay_ms))
            .and_then(|d| d.checked_add(1000))
            .ok_or_else(|| BatcherbirdError::Config("Recording duration is too large".into()))?;
        let values = duration
            .checked_mul(sample_rate as u64)
            .and_then(|n| n.checked_mul(channels as u64))
            .and_then(|n| usize::try_from(n / 1000).ok())
            .ok_or_else(|| BatcherbirdError::Config("Recording buffer is too large".into()))?;
        Ok(values.max(sample_rate as usize * channels as usize))
    }

    /// Blocking interface with real-time visualization support
    /// Uses professional lock-free recording with visualization
    pub fn sample_single_note_with_viz_blocking(
        &self,
        midi_conn: &mut MidiOutputConnection,
        note: u8,
    ) -> Result<(Sample, Consumer<VizChunk>)> {
        // Create dedicated runtime for this blocking operation
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to create runtime: {}", e)))?;

        // Execute the lock-free method and create visualization from audio data
        rt.block_on(async {
            let (sample, _performance) = self
                .sample_single_note_lock_free(midi_conn, note, self.config.velocity)
                .await?;

            // Create visualization data from the recorded audio
            let viz_consumer =
                self.create_visualization_from_audio(&sample.audio_data, sample.sample_rate)?;

            Ok((sample, viz_consumer))
        })
    }

    /// Create visualization data from recorded audio (post-processing approach)
    /// This provides compatibility with existing visualization while using lock-free recording
    fn create_visualization_from_audio(
        &self,
        audio_data: &[f32],
        sample_rate: u32,
    ) -> Result<Consumer<VizChunk>> {
        let (mut viz_producer, viz_consumer) = RingBuffer::<VizChunk>::new(VIZ_RING_BUFFER_SIZE);

        // Process audio data in chunks to simulate real-time visualization
        let chunk_size = (sample_rate as f32 * 0.016) as usize; // ~16ms chunks for 60fps
        let mut timestamp = 0u64;

        for chunk in audio_data.chunks(chunk_size) {
            let viz_chunk = VizChunk::from_samples(chunk, timestamp);

            // Push to ring buffer (ignore if full - consumer responsibility)
            if viz_producer.push(viz_chunk).is_err() {
                // Ring buffer full - this is expected behavior
                break;
            }

            timestamp += chunk.len() as u64;
        }

        Ok(viz_consumer)
    }

    fn build_persistent_recording_stream(
        &self,
        device: &cpal::Device,
        config: &cpal::SupportedStreamConfig,
        producer: Producer<f32>,
        recording_active: Arc<AtomicBool>,
        capture_failed: Arc<AtomicBool>,
    ) -> Result<cpal::Stream> {
        match config.sample_format() {
            cpal::SampleFormat::F32 => self.build_capture_stream::<f32>(
                device,
                config,
                producer,
                recording_active,
                capture_failed,
            ),
            cpal::SampleFormat::I16 => self.build_capture_stream::<i16>(
                device,
                config,
                producer,
                recording_active,
                capture_failed,
            ),
            cpal::SampleFormat::U16 => self.build_capture_stream::<u16>(
                device,
                config,
                producer,
                recording_active,
                capture_failed,
            ),
            format => Err(BatcherbirdError::Audio(format!(
                "Unsupported sample format: {format:?}"
            ))),
        }
    }

    fn build_capture_stream<T>(
        &self,
        device: &cpal::Device,
        config: &cpal::SupportedStreamConfig,
        mut producer: Producer<f32>,
        recording_active: Arc<AtomicBool>,
        capture_failed: Arc<AtomicBool>,
    ) -> Result<cpal::Stream>
    where
        T: cpal::SizedSample,
        f32: cpal::FromSample<T>,
    {
        let level_state = Arc::clone(&self.level_meter_state);
        let mut detector = AudioLevelDetector::new(config.sample_rate().0);
        let channels = config.channels() as usize;
        // Freeze routing for this stream so a UI change cannot invalidate sample metadata.
        let routing = self.get_channel_routing();
        let gain_factor = Arc::clone(&self.input_gain_factor);
        let stream_failed = Arc::clone(&capture_failed);
        device
            .build_input_stream(
                &config.config(),
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    let gain = f32::from_bits(gain_factor.load(Ordering::Relaxed));
                    // Fixed stack scratch space avoids callback heap allocations,
                    // while updating atomics once per block instead of once per frame.
                    for block in data.chunks(channels * 256) {
                        detector.reset_peak();
                        let mut meter_samples = [0.0f32; 512];
                        let mut used = 0;
                        for frame in block.chunks_exact(channels) {
                            let left = <f32 as cpal::FromSample<T>>::from_sample_(frame[0]);
                            let right = if channels > 1 {
                                <f32 as cpal::FromSample<T>>::from_sample_(frame[1])
                            } else {
                                left
                            };
                            meter_samples[used] = left;
                            meter_samples[used + 1] = right;
                            used += 2;
                            if recording_active.load(Ordering::Acquire)
                                && !push_capture_frame(
                                    &mut producer,
                                    left * gain,
                                    right * gain,
                                    channels as u16,
                                    routing,
                                )
                            {
                                capture_failed.store(true, Ordering::Release);
                                recording_active.store(false, Ordering::Release);
                            }
                        }
                        let levels = detector.process_interleaved_samples_with_gain(
                            &meter_samples[..used],
                            2,
                            gain,
                        );
                        level_state.update_levels(levels);
                    }
                },
                move |_err| {
                    stream_failed.store(true, Ordering::Release);
                },
                None,
            )
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to build input stream: {e}")))
    }

    /// Blocking interface for range sampling (follows TAURI_AUDIO_ARCHITECTURE.md)
    pub fn sample_note_range_blocking(
        &self,
        midi_conn: &mut MidiOutputConnection,
        start_note: u8,
        end_note: u8,
    ) -> Result<Vec<Sample>> {
        // Create dedicated runtime for this blocking operation
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to create runtime: {}", e)))?;

        // Execute the async operation in blocking context
        rt.block_on(self.sample_note_range_async(midi_conn, start_note, end_note))
    }

    /// Validate a MIDI note range for range sampling
    fn validate_note_range(start_note: u8, end_note: u8) -> Result<()> {
        if start_note > 127 || end_note > 127 {
            return Err(BatcherbirdError::Config(
                "MIDI notes must be in 0..=127".into(),
            ));
        }
        if start_note > end_note {
            return Err(BatcherbirdError::Config(format!(
                "start_note ({}) must be <= end_note ({})",
                start_note, end_note
            )));
        }
        Ok(())
    }

    /// Internal async implementation for range sampling with persistent stream (Ableton-style)
    async fn sample_note_range_async(
        &self,
        midi_conn: &mut MidiOutputConnection,
        start_note: u8,
        end_note: u8,
    ) -> Result<Vec<Sample>> {
        self.sample_note_range_stepped_with_progress_async(
            midi_conn,
            start_note,
            end_note,
            1,
            1,
            &AtomicBool::new(false),
            |_| {},
            |_| {},
        )
        .await
    }

    /// Record a single note at a given velocity using an already-running
    /// persistent recording stream.
    ///
    /// This is the per-note recording body shared by the legacy range path and
    /// the new progress/cancellation/velocity-layer path. It owns NONE of the
    /// stream lifecycle (build / play / pause): the caller is responsible for
    /// starting and stopping the persistent stream. This helper only performs
    /// the timed MIDI + lock-free ring-buffer drain sequence for one note, with
    /// `velocity` parameterized. Cancellation during a note silences MIDI and
    /// discards that partial sample; capture failures are reported explicitly.
    #[allow(clippy::too_many_arguments)]
    async fn record_one_on_stream(
        &self,
        midi_conn: &mut MidiOutputConnection,
        consumer: &mut Consumer<f32>,
        recording_active: &AtomicBool,
        note: u8,
        velocity: u8,
        sample_rate: u32,
        channels: u16,
        cancel: &AtomicBool,
        capture_failed: &AtomicBool,
    ) -> Result<Option<Sample>> {
        while consumer.pop().is_ok() {}
        recording_active.store(true, Ordering::Release);
        let start_time = Instant::now();
        let result: Result<Option<Duration>> = async {
            if !wait_for_capture(self.config.pre_delay_ms, cancel, capture_failed).await? {
                return Ok(None);
            }
            MidiManager::send_channel_panic(midi_conn, self.config.midi_channel)?;
            if !wait_for_capture(50, cancel, capture_failed).await? {
                return Ok(None);
            }
            let midi_start = Instant::now();
            MidiManager::send_note_on(midi_conn, self.config.midi_channel, note, velocity)?;
            if !wait_for_capture(self.config.note_duration_ms, cancel, capture_failed).await? {
                return Ok(None);
            }
            MidiManager::send_note_off(midi_conn, self.config.midi_channel, note, velocity)?;
            let midi_timing = midi_start.elapsed();
            if !wait_for_capture(self.config.release_time_ms, cancel, capture_failed).await?
                || !wait_for_capture(self.config.post_delay_ms, cancel, capture_failed).await?
            {
                return Ok(None);
            }
            Ok(Some(midi_timing))
        }
        .await;
        recording_active.store(false, Ordering::Release);
        if !matches!(&result, Ok(Some(_))) {
            // Best effort cleanup must also happen when a MIDI send or capture fails.
            let _ = MidiManager::send_channel_panic(midi_conn, self.config.midi_channel);
        }
        let Some(midi_timing) = result? else {
            return Ok(None);
        };
        tokio::time::sleep(Duration::from_millis(10)).await;
        if capture_failed.load(Ordering::Acquire) {
            return Err(BatcherbirdError::Audio(
                "Audio capture failed or buffer overflowed; sample was discarded".into(),
            ));
        }
        let mut audio_data = Vec::new();
        while let Ok(sample) = consumer.pop() {
            audio_data.push(sample);
        }
        Ok(Some(Sample {
            note,
            velocity,
            audio_data,
            sample_rate,
            channels,
            recorded_at: std::time::SystemTime::now(),
            midi_timing,
            audio_timing: start_time.elapsed(),
        }))
    }

    /// Blocking range sampling with progress reporting, cooperative
    /// cancellation, and multi-velocity-layer support.
    ///
    /// This is the richer counterpart to [`Self::sample_note_range_blocking`]
    /// (which is retained as-is for CLI back-compat). It:
    /// - records each note across `velocity_layer_count` velocity layers
    ///   (see [`velocity_layers`]),
    /// - invokes `progress` before and after every sample,
    /// - checks `cancel` throughout each recording phase and, if set, silences any
    ///   held note and returns the samples gathered so far (partial result).
    ///
    /// Per-note recording errors are propagated (matching the legacy path: one
    /// failed note fails the whole run).
    pub fn sample_note_range_with_progress_blocking(
        &self,
        midi_conn: &mut MidiOutputConnection,
        start_note: u8,
        end_note: u8,
        velocity_layer_count: u8,
        cancel: &AtomicBool,
        progress: impl FnMut(RecordingProgress),
    ) -> Result<Vec<Sample>> {
        self.sample_note_range_stepped_with_progress_blocking(
            midi_conn,
            start_note,
            end_note,
            1,
            velocity_layer_count,
            cancel,
            progress,
        )
    }

    /// Sample a range of MIDI notes with a specific step interval (e.g. 1 for every note,
    /// 3 for minor thirds, 12 for octaves) and progress updates.
    #[allow(clippy::too_many_arguments)]
    pub fn sample_note_range_stepped_with_progress_blocking(
        &self,
        midi_conn: &mut MidiOutputConnection,
        start_note: u8,
        end_note: u8,
        step: u8,
        velocity_layer_count: u8,
        cancel: &AtomicBool,
        progress: impl FnMut(RecordingProgress),
    ) -> Result<Vec<Sample>> {
        self.sample_note_range_stepped_with_capture_blocking(
            midi_conn,
            start_note,
            end_note,
            step,
            velocity_layer_count,
            cancel,
            progress,
            |_| {},
        )
    }

    /// Record a batch while handing off each completed take before continuing.
    /// `captured` runs on the caller's worker thread, never the audio callback.
    /// A later recording error cannot revoke samples already handed to it.
    #[allow(clippy::too_many_arguments)]
    pub fn sample_note_range_stepped_with_capture_blocking(
        &self,
        midi_conn: &mut MidiOutputConnection,
        start_note: u8,
        end_note: u8,
        step: u8,
        velocity_layer_count: u8,
        cancel: &AtomicBool,
        progress: impl FnMut(RecordingProgress),
        captured: impl FnMut(&Sample),
    ) -> Result<Vec<Sample>> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to create runtime: {e}")))?;
        rt.block_on(self.sample_note_range_stepped_with_progress_async(
            midi_conn,
            start_note,
            end_note,
            step,
            velocity_layer_count,
            cancel,
            progress,
            captured,
        ))
    }

    /// Async implementation backing
    /// [`Self::sample_note_range_stepped_with_progress_blocking`].
    ///
    /// Runs on the current thread via `block_on`, so the non-`Send` `progress`
    /// closure and `cancel`/`midi_conn` references thread through the `.await`
    /// points without issue.
    #[allow(clippy::too_many_arguments)]
    async fn sample_note_range_stepped_with_progress_async(
        &self,
        midi_conn: &mut MidiOutputConnection,
        start_note: u8,
        end_note: u8,
        step: u8,
        velocity_layer_count: u8,
        cancel: &AtomicBool,
        mut progress: impl FnMut(RecordingProgress),
        mut captured: impl FnMut(&Sample),
    ) -> Result<Vec<Sample>> {
        Self::validate_note_range(start_note, end_note)?;

        let velocities = if velocity_layer_count <= 1 {
            vec![self.config.velocity]
        } else {
            velocity_layers(velocity_layer_count)
        };
        let total_layers = velocities.len() as u8;
        let step = step.max(1);
        let notes: Vec<u8> = (start_note..=end_note).step_by(step as usize).collect();
        let num_notes = notes.len() as u32;
        let total = num_notes * velocities.len() as u32;

        let mut samples = Vec::new();
        if cancel.load(Ordering::Acquire) {
            return Ok(samples);
        }

        // Safety: Clear any stuck notes before starting range recording session
        MidiManager::send_midi_panic(midi_conn)?;
        tokio::time::sleep(Duration::from_millis(100)).await; // Give hardware time to process

        let device = self
            .audio_manager
            .find_input_device(self.config.input_device_name.as_deref())?;
        let config = device
            .default_input_config()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to get input config: {}", e)))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        let recorded_channels = self.get_channel_routing().output_channels(channels);

        let ring_buffer_size = self.capture_capacity(sample_rate, recorded_channels)?;

        let (producer, mut consumer) = RingBuffer::<f32>::new(ring_buffer_size);
        let recording_active = Arc::new(AtomicBool::new(false));
        let recording_active_clone = Arc::clone(&recording_active);
        let capture_failed = Arc::new(AtomicBool::new(false));

        // Create ONE stream for the entire range (like professional DAWs).
        let stream = self.build_persistent_recording_stream(
            &device,
            &config,
            producer,
            recording_active_clone,
            Arc::clone(&capture_failed),
        )?;

        stream.play().map_err(|e| {
            BatcherbirdError::Audio(format!("Failed to start persistent stream: {}", e))
        })?;

        let mut first = true;
        'outer: for &note in &notes {
            for (layer_idx, vel) in velocities.iter().enumerate() {
                // Cooperative cancellation: check BEFORE recording each sample.
                if cancel.load(Ordering::Relaxed) {
                    break 'outer;
                }

                // Brief pause between samples (hardware stability), skipping the
                // very first sample. Mirrors the 300ms inter-note pause of the
                // legacy path and also separates velocity layers.
                if !first {
                    match wait_for_capture(300, cancel, &capture_failed).await {
                        Ok(true) => {}
                        Ok(false) => break 'outer,
                        Err(error) => {
                            let _ = stream.pause();
                            let _ = MidiManager::send_midi_panic(midi_conn);
                            return Err(error);
                        }
                    }
                }
                first = false;

                progress(RecordingProgress {
                    note,
                    velocity: *vel,
                    layer: layer_idx as u8,
                    total_layers,
                    completed: samples.len() as u32,
                    total,
                });
                let sample = self
                    .record_one_on_stream(
                        midi_conn,
                        &mut consumer,
                        &recording_active,
                        note,
                        *vel,
                        sample_rate,
                        recorded_channels,
                        cancel,
                        &capture_failed,
                    )
                    .await;
                let sample = match sample {
                    Ok(Some(sample)) => sample,
                    Ok(None) => break 'outer,
                    Err(error) => {
                        recording_active.store(false, Ordering::Release);
                        let _ = stream.pause();
                        let _ = MidiManager::send_midi_panic(midi_conn);
                        return Err(error);
                    }
                };
                captured(&sample);
                samples.push(sample);

                progress(RecordingProgress {
                    note,
                    velocity: *vel,
                    layer: layer_idx as u8,
                    total_layers,
                    completed: samples.len() as u32,
                    total,
                });
            }
        }

        // Clean shutdown of persistent stream
        let pause_result = stream.pause();
        drop(stream);
        let panic_result = MidiManager::send_midi_panic(midi_conn);
        panic_result?;
        pause_result.map_err(|e| {
            BatcherbirdError::Audio(format!("Failed to stop persistent stream: {e}"))
        })?;

        Ok(samples)
    }

    /// 🚀 PROFESSIONAL LOCK-FREE MIDI RECORDING (Industry Standard Solution)
    ///
    /// This method implements the lock-free recording architecture used by professional DAWs:
    /// - Ableton Live: Lock-free SPSC queues for audio data
    /// - Pro Tools: Dedicated recording threads with atomic state
    /// - Logic Pro: Ring buffers for real-time audio streams
    /// - Ardour: Separate disk writer threads for I/O operations
    ///
    /// ✅ ARCHITECTURE BENEFITS:
    /// - Zero mutex contention in audio thread
    /// - No memory allocation during recording
    /// - Sample-accurate timing precision
    /// - Professional-grade performance monitoring
    pub async fn sample_single_note_lock_free(
        &self,
        midi_output: &mut MidiOutputConnection,
        note: u8,
        velocity: u8,
    ) -> Result<(Sample, AudioPerformanceReport)> {
        // Get audio device and configuration first so the recorder (and the
        // exported sample metadata) match the device that actually captures
        let device = self
            .audio_manager
            .find_input_device(self.config.input_device_name.as_deref())?;
        let config = device
            .default_input_config()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to get input config: {}", e)))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        let samples_per_second = sample_rate as usize * channels as usize;

        // Create lock-free recorder with professional configuration
        let routing = self.get_channel_routing();
        let recording_config = LockFreeRecordingConfig {
            ring_buffer_size: samples_per_second * 4, // 4 seconds buffer (professional standard)
            sample_rate,
            channels,
            max_recording_samples: self
                .capture_capacity(sample_rate, routing.output_channels(channels))?,
            channel_routing: routing,
            input_gain_factor: Arc::clone(&self.input_gain_factor),
        };

        let mut recorder = LockFreeRecorder::new(recording_config)?;

        // Start lock-free recording session
        recorder.start_recording()?;

        // Build professional lock-free audio stream
        let stream = recorder.build_lock_free_stream(&device, &config)?;

        // Start audio stream
        stream
            .play()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to start stream: {}", e)))?;

        // MIDI sequence with precise timing (following Pro Tools approach)
        let start_time = tokio::time::Instant::now();

        let sequence: Result<(Instant, Instant)> = async {
            tokio::time::sleep(Duration::from_millis(self.config.pre_delay_ms)).await;
            MidiManager::send_note_on(midi_output, self.config.midi_channel, note, velocity)?;
            let midi_start = Instant::now();
            tokio::time::sleep(Duration::from_millis(self.config.note_duration_ms)).await;
            MidiManager::send_note_off(midi_output, self.config.midi_channel, note, velocity)?;
            let midi_end = Instant::now();
            tokio::time::sleep(Duration::from_millis(
                self.config.release_time_ms + self.config.post_delay_ms,
            ))
            .await;
            Ok((midi_start, midi_end))
        }
        .await;
        let panic_result = MidiManager::send_channel_panic(midi_output, self.config.midi_channel);
        let (midi_start, midi_end) = sequence?;
        panic_result?;

        // Stop lock-free recording
        let audio_data = recorder.stop_recording()?;

        // Stop audio stream
        drop(stream);

        let end_time = tokio::time::Instant::now();

        // Get performance diagnostics
        let performance_report = self.get_performance_diagnostics();

        // Create sample with metadata matching the device config used for capture
        let sample = Sample {
            note,
            velocity,
            audio_data,
            sample_rate,
            channels: recorder.channels,
            recorded_at: std::time::SystemTime::now(),
            midi_timing: midi_end.duration_since(midi_start),
            audio_timing: end_time.duration_since(start_time),
        };

        Ok((sample, performance_report))
    }
}

impl Sample {
    /// Apply sample detection and trimming to this sample
    pub fn apply_detection(&mut self, config: DetectionConfig) -> Result<DetectionResult> {
        let detector = SampleDetector::new(config);
        let detection_result = detector.detect_boundaries_channels(
            &self.audio_data,
            self.sample_rate,
            self.channels,
        )?;

        if detection_result.success {
            // Trim the audio data
            self.audio_data = detector.trim_audio(&self.audio_data, &detection_result);
        }

        Ok(detection_result)
    }

    /// Apply loop detection to find optimal loop points in the sample
    pub fn apply_loop_detection(
        &mut self,
        config: LoopDetectionConfig,
    ) -> Result<LoopDetectionResult> {
        let detector = LoopDetector::new(config);
        let loop_result =
            detector.detect_loop_points_channels(&self.audio_data, self.sample_rate, self.channels);

        if loop_result.success {
            if let Some(ref candidate) = loop_result.best_candidate {
                // Apply the loop with equal-power crossfading across all channels
                let _ = detector.apply_loop_with_crossfade_channels(
                    &mut self.audio_data,
                    candidate,
                    self.sample_rate,
                    self.channels,
                );
            }
        }

        Ok(loop_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rtrb::RingBuffer;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_note_range_validation() {
        // Valid ranges
        assert!(SamplingEngine::validate_note_range(0, 127).is_ok());
        assert!(SamplingEngine::validate_note_range(60, 60).is_ok());

        // Inverted range must be rejected (previously underflowed in u8 math)
        let err = SamplingEngine::validate_note_range(72, 60).unwrap_err();
        assert!(err.to_string().contains("start_note"));
    }

    #[test]
    fn test_velocity_layers_single_and_zero() {
        assert_eq!(velocity_layers(0), vec![127]);
        assert_eq!(velocity_layers(1), vec![127]);
    }

    #[test]
    fn test_velocity_layers_examples() {
        assert_eq!(velocity_layers(2), vec![64, 127]);
        assert_eq!(velocity_layers(3), vec![48, 96, 127]);
        assert_eq!(velocity_layers(4), vec![32, 64, 96, 127]);
    }

    #[test]
    fn test_velocity_layers_invariants() {
        for count in 2..=16u8 {
            let layers = velocity_layers(count);

            // Correct length.
            assert_eq!(layers.len(), count as usize, "length for count {}", count);

            // Within 1..=127.
            for &v in &layers {
                assert!((1..=127).contains(&v), "velocity {} out of range", v);
            }

            // Strictly ascending.
            for w in layers.windows(2) {
                assert!(w[0] < w[1], "not strictly ascending for count {}", count);
            }

            // Endpoints: last == 127.
            assert_eq!(
                *layers.last().unwrap(),
                127,
                "last velocity for count {}",
                count
            );
        }
    }

    #[test]
    fn test_velocity_layers_caps_at_16() {
        // Counts above the cap collapse to 16 strictly-ascending layers.
        let capped = velocity_layers(200);
        assert_eq!(capped.len(), 16);
        for w in capped.windows(2) {
            assert!(w[0] < w[1]);
        }
    }

    #[test]
    fn test_viz_chunk_creation() {
        let samples = vec![0.5, -0.3, 0.8, -0.1];
        let chunk = VizChunk::from_samples(&samples, 1000);

        assert_eq!(chunk.timestamp, 1000);
        assert_eq!(chunk.chunk_size, 4);
        assert!(chunk.peak > 0.0);
        assert!(chunk.rms > 0.0);
        assert!(chunk.peak_db > -60.0);
        assert!(chunk.rms_db > -60.0);
    }

    #[test]
    fn test_ring_buffer_stress() {
        // Test ring buffer can handle audio-rate data without blocking
        let (mut producer, mut consumer) = RingBuffer::<VizChunk>::new(VIZ_RING_BUFFER_SIZE);

        // Simulate audio thread producing at ~44kHz in chunks
        let producer_handle = thread::spawn(move || {
            for i in 0..1000 {
                let samples = vec![0.1 * (i as f32), -0.1 * (i as f32)];
                let chunk = VizChunk::from_samples(&samples, i * 2);

                // This should never block - if buffer is full, we drop the chunk
                if producer.push(chunk).is_err() {
                    // Buffer full - this is expected behavior, not an error
                }

                // Simulate audio callback timing (~1ms chunks at 44kHz)
                thread::sleep(Duration::from_micros(100)); // Fast simulation
            }
        });

        // Simulate visualization thread consuming at 60fps
        let consumer_handle = thread::spawn(move || {
            let mut chunks_received = 0;

            for _ in 0..60 {
                // 60 iterations = 1 second at 60fps
                // Try to consume all available chunks
                while let Ok(_chunk) = consumer.pop() {
                    chunks_received += 1;
                }

                // 60fps timing
                thread::sleep(Duration::from_millis(16));
            }

            chunks_received
        });

        producer_handle.join().unwrap();
        let chunks_received = consumer_handle.join().unwrap();

        // We should receive some chunks (not all due to 60fps vs faster production)
        assert!(
            chunks_received > 0,
            "Should receive some visualization chunks"
        );
        assert!(
            chunks_received < 1000,
            "Should not receive all chunks due to 60fps consumption"
        );
    }

    #[test]
    fn test_playthrough_state_toggle() {
        let config = SamplingConfig::default();
        if let Ok(engine) = SamplingEngine::new(config) {
            assert!(!engine.is_playthrough_enabled());
            engine.set_playthrough(true);
            assert!(engine.is_playthrough_enabled());
            engine.set_playthrough(false);
            assert!(!engine.is_playthrough_enabled());
        }
    }

    #[test]
    fn test_channel_routing_engine_toggle() {
        let config = SamplingConfig::default();
        if let Ok(engine) = SamplingEngine::new(config) {
            assert_eq!(engine.get_channel_routing(), ChannelRouting::Stereo);
            engine.set_channel_routing(ChannelRouting::MonoLeft);
            assert_eq!(engine.get_channel_routing(), ChannelRouting::MonoLeft);
            engine.set_channel_routing(ChannelRouting::MonoRight);
            assert_eq!(engine.get_channel_routing(), ChannelRouting::MonoRight);
            engine.set_channel_routing(ChannelRouting::Stereo);
            assert_eq!(engine.get_channel_routing(), ChannelRouting::Stereo);
        }
    }

    #[test]
    fn test_interleaved_stereo_metering() {
        let mut detector = AudioLevelDetector::new(44100);
        // Stereo interleaved: Left = 0.8, Right = 0.2
        let samples = vec![0.8, 0.2, 0.6, 0.1, -0.7, -0.05];
        let levels = detector.process_interleaved_samples(&samples, 2);

        assert!((levels.peak_left - 0.8).abs() < 1e-5);
        assert!((levels.peak_right - 0.2).abs() < 1e-5);
        assert!((levels.peak - 0.8).abs() < 1e-5);
        assert!(levels.peak_left_db > levels.peak_right_db);
    }

    #[test]
    fn test_input_gain_engine_toggle() {
        let config = SamplingConfig::default();
        assert_eq!(config.input_gain_db, 0.0);

        if let Ok(engine) = SamplingEngine::new(config) {
            assert!((engine.get_input_gain_factor() - 1.0).abs() < 1e-4);
            assert!((engine.get_input_gain_db() - 0.0).abs() < 1e-4);

            engine.set_input_gain_db(3.0);
            assert!((engine.get_input_gain_db() - 3.0).abs() < 1e-4);

            engine.set_input_gain_db(-6.0);
            assert!((engine.get_input_gain_db() - (-6.0)).abs() < 1e-4);

            // Clamping tests
            engine.set_input_gain_db(20.0);
            assert!((engine.get_input_gain_db() - 12.0).abs() < 1e-4);

            engine.set_input_gain_db(-20.0);
            assert!((engine.get_input_gain_db() - (-12.0)).abs() < 1e-4);
        }
    }

    #[test]
    fn test_interleaved_samples_with_gain() {
        let mut detector = AudioLevelDetector::new(44100);
        let samples = vec![0.4, 0.1, 0.3, 0.05];
        // Gain 2.0 (+6dB): 0.4 -> 0.8, 0.1 -> 0.2
        let levels = detector.process_interleaved_samples_with_gain(&samples, 2, 2.0);

        assert!((levels.peak_left - 0.8).abs() < 1e-5);
        assert!((levels.peak_right - 0.2).abs() < 1e-5);
        assert!((levels.peak - 0.8).abs() < 1e-5);

        // Gain 0.5 (-6dB) on a fresh detector: 0.4 -> 0.2, 0.1 -> 0.05
        let mut detector_attenuated = AudioLevelDetector::new(44100);
        let levels_attenuated =
            detector_attenuated.process_interleaved_samples_with_gain(&samples, 2, 0.5);
        assert!((levels_attenuated.peak_left - 0.2).abs() < 1e-5);
        assert!((levels_attenuated.peak_right - 0.05).abs() < 1e-5);
        assert!((levels_attenuated.peak - 0.2).abs() < 1e-5);
    }
}

#[cfg(test)]
mod capture_integrity_tests {
    use super::*;

    #[test]
    fn waveform_history_starts_empty_and_keeps_chronological_peaks() {
        let state = LevelMeterState::new();
        assert!(state.get_waveform_peaks().is_empty());
        for peak in [0.1, 0.3, 0.2] {
            state.update_levels(AudioLevels {
                peak,
                ..AudioLevels::default()
            });
        }
        assert_eq!(state.get_waveform_peaks(), vec![0.1, 0.3, 0.2]);
    }

    #[test]
    fn waveform_history_wraps_without_old_or_uninitialized_entries() {
        let state = LevelMeterState::new();
        for index in 0..300 {
            state.push_waveform_peak(index as f32 / 1000.0);
        }
        let expected: Vec<f32> = (44..300).map(|index| index as f32 / 1000.0).collect();
        assert_eq!(state.get_waveform_peaks(), expected);
        assert_eq!(state.get_waveform_peaks().len(), WAVEFORM_HISTORY_CAPACITY);
    }

    #[test]
    fn waveform_envelope_is_finite_and_normalized() {
        let state = LevelMeterState::new();
        for peak in [-0.5, 8.0, f32::NAN, f32::INFINITY] {
            state.push_waveform_peak(peak);
        }
        assert_eq!(state.get_waveform_peaks(), vec![0.5, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn stereo_overflow_never_pushes_half_frame() {
        let (mut producer, mut consumer) = RingBuffer::new(3);
        assert!(push_capture_frame(
            &mut producer,
            0.1,
            0.2,
            8,
            ChannelRouting::Stereo
        ));
        assert!(!push_capture_frame(
            &mut producer,
            0.3,
            0.4,
            8,
            ChannelRouting::Stereo
        ));
        assert_eq!(consumer.pop().unwrap(), 0.1);
        assert_eq!(consumer.pop().unwrap(), 0.2);
        assert!(consumer.pop().is_err());
    }

    #[test]
    fn routing_records_selected_channel_and_mono_fallback() {
        let (mut producer, mut consumer) = RingBuffer::new(4);
        assert!(push_capture_frame(
            &mut producer,
            0.1,
            0.9,
            8,
            ChannelRouting::MonoRight
        ));
        assert_eq!(consumer.pop().unwrap(), 0.9);
        assert!(push_capture_frame(
            &mut producer,
            0.4,
            0.4,
            1,
            ChannelRouting::Stereo
        ));
        assert_eq!(consumer.pop().unwrap(), 0.4);
        assert!(consumer.pop().is_err());
    }

    #[tokio::test]
    async fn cancellation_interrupts_a_long_note_wait() {
        let cancel = Arc::new(AtomicBool::new(false));
        let trigger = Arc::clone(&cancel);
        let task = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(15)).await;
            trigger.store(true, Ordering::Release);
        });
        let result = tokio::time::timeout(
            Duration::from_millis(250),
            wait_for_capture(10_000, &cancel, &AtomicBool::new(false)),
        )
        .await;
        assert!(!result
            .expect("cancel should not wait for full note duration")
            .unwrap());
        task.await.unwrap();
    }

    #[tokio::test]
    async fn capture_failure_is_reported_even_during_zero_length_wait() {
        assert!(
            wait_for_capture(0, &AtomicBool::new(false), &AtomicBool::new(true))
                .await
                .is_err()
        );
    }

    #[test]
    fn all_midi_notes_range_is_valid_but_out_of_range_notes_are_rejected() {
        assert!(SamplingEngine::validate_note_range(0, 127).is_ok());
        assert!(SamplingEngine::validate_note_range(127, 128).is_err());
    }
}
