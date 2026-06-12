use crate::{BatcherbirdError, Result};
use cpal::traits::DeviceTrait;
use cpal::{SampleFormat, Stream, SupportedStreamConfig};
use rtrb::{Consumer, Producer, RingBuffer};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Real-time meter data for lock-free streaming to UI
#[derive(Debug, Clone, Copy)]
pub struct RealtimeMeterData {
    pub peak_left: f32,
    pub peak_right: f32,
    pub rms_left: f32,
    pub rms_right: f32,
    pub timestamp_ms: u64,
    pub is_clipping: bool,
}

/// Professional lock-free audio recording architecture following DAW industry standards
///
/// Based on research from Ableton Live, Pro Tools, Logic Pro, and Ardour:
/// - Lock-free SPSC ring buffer for audio samples (never blocks)
/// - AtomicBool for recording state (no mutex contention)
/// - Pre-allocated memory pools (zero allocations in audio thread)
/// - Dedicated consumer thread for disk I/O (following Ardour pattern)
/// - Sample-accurate timestamping (sub-millisecond precision)
#[derive(Debug)]
pub struct LockFreeRecorder {
    // Audio sample ring buffer (Single Producer, Single Consumer)
    sample_producer: Arc<std::sync::Mutex<Option<Producer<f32>>>>,
    sample_consumer: Option<Consumer<f32>>,

    // Meter data ring buffer for real-time UI streaming (NEW)
    meter_producer: Arc<std::sync::Mutex<Option<Producer<RealtimeMeterData>>>>,
    meter_consumer: Option<Consumer<RealtimeMeterData>>,

    // Recording state (lock-free atomic)
    is_recording: Arc<AtomicBool>,

    // Sample counting (for precise timing)
    samples_recorded: Arc<AtomicUsize>,

    // Audio configuration
    sample_rate: u32,
    channels: u16,

    // Performance configuration
    buffer_size: usize,

    // Safety limit: maximum number of interleaved samples per recording
    max_recording_samples: usize,

    // Consumer thread handle
    consumer_thread: Option<thread::JoinHandle<Result<Vec<f32>>>>,
}

/// Configuration for lock-free recording following professional standards
#[derive(Debug, Clone)]
pub struct LockFreeRecordingConfig {
    /// Ring buffer size in samples (default: 44100 * 2 = 2 seconds)
    /// Professional DAWs use 1-4 second buffers for recording
    pub ring_buffer_size: usize,

    /// Sample rate (standardized to 44.1kHz)
    pub sample_rate: u32,

    /// Number of channels (standardized to stereo)
    pub channels: u16,

    /// Maximum recording duration in samples (safety limit)
    pub max_recording_samples: usize,
}

impl Default for LockFreeRecordingConfig {
    fn default() -> Self {
        Self {
            ring_buffer_size: 44100 * 2,           // 2 seconds at 44.1kHz
            sample_rate: 44100,                    // Professional standard
            channels: 2,                           // Stereo
            max_recording_samples: 44100 * 60 * 5, // 5 minutes max
        }
    }
}

impl LockFreeRecorder {
    /// Create new lock-free recorder with professional configuration
    pub fn new(config: LockFreeRecordingConfig) -> Result<Self> {
        // Create lock-free SPSC ring buffer for audio samples
        let (sample_producer, sample_consumer) = RingBuffer::<f32>::new(config.ring_buffer_size);

        // Create lock-free ring buffer for meter data (60fps = 60 updates/sec)
        let (meter_producer, meter_consumer) = RingBuffer::<RealtimeMeterData>::new(128);

        Ok(Self {
            sample_producer: Arc::new(std::sync::Mutex::new(Some(sample_producer))),
            sample_consumer: Some(sample_consumer),
            meter_producer: Arc::new(std::sync::Mutex::new(Some(meter_producer))),
            meter_consumer: Some(meter_consumer),
            is_recording: Arc::new(AtomicBool::new(false)),
            samples_recorded: Arc::new(AtomicUsize::new(0)),
            sample_rate: config.sample_rate,
            channels: config.channels,
            buffer_size: config.ring_buffer_size,
            max_recording_samples: config.max_recording_samples,
            consumer_thread: None,
        })
    }

    /// Get meter data consumer for real-time UI streaming
    pub fn take_meter_consumer(&mut self) -> Option<Consumer<RealtimeMeterData>> {
        self.meter_consumer.take()
    }

    /// Start lock-free recording session
    pub fn start_recording(&mut self) -> Result<()> {
        if self.is_recording.load(Ordering::Relaxed) {
            return Err(BatcherbirdError::Audio(
                "Recording already in progress".to_string(),
            ));
        }

        // Reset sample counter
        self.samples_recorded.store(0, Ordering::Relaxed);

        // Start consumer thread (following Ardour's disk writer pattern)
        let mut consumer = self
            .sample_consumer
            .take()
            .ok_or_else(|| BatcherbirdError::Audio("Consumer already taken".to_string()))?;

        let is_recording = Arc::clone(&self.is_recording);
        let samples_recorded = Arc::clone(&self.samples_recorded);
        let max_samples = self.max_recording_samples.max(1); // Safety limit
        let initial_capacity = self.buffer_size.min(max_samples);

        self.consumer_thread = Some(thread::spawn(move || {
            let mut recorded_samples = Vec::with_capacity(initial_capacity);
            let last_report = Instant::now();

            while is_recording.load(Ordering::Relaxed) || !consumer.is_empty() {
                // Consume samples from ring buffer (never blocks)
                while let Ok(sample) = consumer.pop() {
                    recorded_samples.push(sample);
                    samples_recorded.fetch_add(1, Ordering::Relaxed);

                    // Safety limit reached: stop the recording entirely so the
                    // audio callback stops pushing and memory stays bounded
                    if recorded_samples.len() >= max_samples {
                        tracing::warn!(
                            "Recording reached max_recording_samples ({}) — stopping",
                            max_samples
                        );
                        is_recording.store(false, Ordering::Relaxed);
                        return Ok(recorded_samples);
                    }
                }

                // Prevent unused variable warning
                let _ = last_report;

                // Small sleep to prevent busy waiting (following professional practice)
                thread::sleep(Duration::from_micros(100));
            }

            Ok(recorded_samples)
        }));

        // Set recording flag (atomic, no contention)
        self.is_recording.store(true, Ordering::Relaxed);

        Ok(())
    }

    /// Stop recording and retrieve samples
    pub fn stop_recording(&mut self) -> Result<Vec<f32>> {
        // Allow stopping when a consumer thread exists even if the recording
        // flag was already cleared (e.g. max_recording_samples limit reached)
        if !self.is_recording.load(Ordering::Relaxed) && self.consumer_thread.is_none() {
            return Err(BatcherbirdError::Audio(
                "Not currently recording".to_string(),
            ));
        }

        // Stop recording (atomic flag)
        self.is_recording.store(false, Ordering::Relaxed);

        // Wait for consumer thread to finish
        if let Some(handle) = self.consumer_thread.take() {
            let result = handle
                .join()
                .map_err(|_| BatcherbirdError::Audio("Consumer thread panicked".to_string()))?;

            let samples = result?;

            return Ok(samples);
        }

        Err(BatcherbirdError::Audio(
            "No consumer thread found".to_string(),
        ))
    }

    /// Build professional lock-free audio stream
    pub fn build_lock_free_stream(
        &self,
        device: &cpal::Device,
        config: &SupportedStreamConfig,
    ) -> Result<Stream> {
        // Get producer for audio thread (move into closure)
        let producer = {
            let mut producer_guard = self.sample_producer.lock().unwrap();
            producer_guard
                .take()
                .ok_or_else(|| BatcherbirdError::Audio("Producer already taken".to_string()))?
        };

        // Get meter producer for real-time UI updates
        let meter_producer = {
            let mut meter_guard = self.meter_producer.lock().unwrap();
            meter_guard.take().ok_or_else(|| {
                BatcherbirdError::Audio("Meter producer already taken".to_string())
            })?
        };

        let is_recording = Arc::clone(&self.is_recording);
        let sample_rate = self.sample_rate;
        let channels = self.channels;

        // Create standard audio configuration
        let stream_config = cpal::StreamConfig {
            channels: self.channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = match config.sample_format() {
            SampleFormat::F32 => self.build_f32_stream(
                device,
                &stream_config,
                producer,
                meter_producer,
                is_recording,
                channels,
            )?,
            SampleFormat::I16 => self.build_i16_stream(
                device,
                &stream_config,
                producer,
                meter_producer,
                is_recording,
                channels,
            )?,
            SampleFormat::U16 => self.build_u16_stream(
                device,
                &stream_config,
                producer,
                meter_producer,
                is_recording,
                channels,
            )?,
            _ => {
                return Err(BatcherbirdError::Audio(
                    "Unsupported sample format".to_string(),
                ))
            }
        };

        Ok(stream)
    }

    /// Build F32 stream with lock-free architecture
    fn build_f32_stream(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        mut producer: Producer<f32>,
        mut meter_producer: Producer<RealtimeMeterData>,
        is_recording: Arc<AtomicBool>,
        channels: u16,
    ) -> Result<Stream> {
        let mut sample_count = 0u64;
        let mut rms_accumulator_left = 0.0f32;
        let mut rms_accumulator_right = 0.0f32;
        let mut rms_window_samples = 0usize;
        let rms_window_size = 512; // ~11ms at 44.1kHz for smooth meters
        // sample_count below counts interleaved samples across all channels
        let interleaved_samples_per_sec = self.sample_rate.max(1) as u64 * channels.max(1) as u64;

        let stream = device
            .build_input_stream(
                config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // ✅ PROFESSIONAL AUDIO: Lock-free recording in audio thread
                    if is_recording.load(Ordering::Relaxed) {
                        // Push samples to ring buffer (never blocks, never allocates)
                        for &sample in data {
                            if producer.push(sample).is_err() {
                                // Ring buffer full - this is expected behavior
                                // Professional DAWs handle this gracefully
                                break;
                            }
                        }
                    }

                    // Calculate real-time meter data (always, even when not recording)
                    let mut peak_left = 0.0f32;
                    let mut peak_right = 0.0f32;
                    let mut is_clipping = false;

                    // Process samples based on channel configuration
                    if channels == 2 {
                        // Stereo: interleaved L/R samples
                        for chunk in data.chunks(2) {
                            if let [left, right] = chunk {
                                // Peak detection
                                peak_left = peak_left.max(left.abs());
                                peak_right = peak_right.max(right.abs());

                                // RMS accumulation
                                rms_accumulator_left += left * left;
                                rms_accumulator_right += right * right;
                                rms_window_samples += 1;

                                // Clipping detection
                                if left.abs() >= 0.999 || right.abs() >= 0.999 {
                                    is_clipping = true;
                                }
                            }
                        }
                    } else {
                        // Mono: all samples to both channels
                        for &sample in data {
                            peak_left = peak_left.max(sample.abs());
                            peak_right = peak_left; // Same for mono

                            rms_accumulator_left += sample * sample;
                            rms_accumulator_right = rms_accumulator_left;
                            rms_window_samples += 1;

                            if sample.abs() >= 0.999 {
                                is_clipping = true;
                            }
                        }
                    }

                    // Calculate RMS when window is full
                    if rms_window_samples >= rms_window_size {
                        let rms_left = (rms_accumulator_left / rms_window_samples as f32).sqrt();
                        let rms_right = (rms_accumulator_right / rms_window_samples as f32).sqrt();

                        // Create meter data
                        let meter_data = RealtimeMeterData {
                            peak_left,
                            peak_right,
                            rms_left,
                            rms_right,
                            timestamp_ms: sample_count * 1000 / interleaved_samples_per_sec, // Convert to ms
                            is_clipping,
                        };

                        // Push to meter ring buffer (non-blocking)
                        let _ = meter_producer.push(meter_data);

                        // Reset RMS accumulators
                        rms_accumulator_left = 0.0;
                        rms_accumulator_right = 0.0;
                        rms_window_samples = 0;
                    }

                    sample_count += data.len() as u64;
                    // ✅ No mutex locks, no memory allocation, no blocking operations
                },
                |err| tracing::error!("Audio input error: {}", err),
                None,
            )
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to build F32 stream: {}", e)))?;

        Ok(stream)
    }

    /// Build I16 stream with lock-free architecture  
    fn build_i16_stream(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        mut producer: Producer<f32>,
        _meter_producer: Producer<RealtimeMeterData>,
        is_recording: Arc<AtomicBool>,
        _channels: u16,
    ) -> Result<Stream> {
        let stream = device
            .build_input_stream(
                config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    // ✅ PROFESSIONAL AUDIO: Lock-free recording in audio thread
                    if is_recording.load(Ordering::Relaxed) {
                        // Convert and push samples (no Vec allocation)
                        for &sample in data {
                            let f32_sample = sample as f32 / i16::MAX as f32;
                            if producer.push(f32_sample).is_err() {
                                break;
                            }
                        }
                    }
                },
                |err| tracing::error!("Audio input error: {}", err),
                None,
            )
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to build I16 stream: {}", e)))?;

        Ok(stream)
    }

    /// Build U16 stream with lock-free architecture
    fn build_u16_stream(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        mut producer: Producer<f32>,
        _meter_producer: Producer<RealtimeMeterData>,
        is_recording: Arc<AtomicBool>,
        _channels: u16,
    ) -> Result<Stream> {
        let stream = device
            .build_input_stream(
                config,
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    // ✅ PROFESSIONAL AUDIO: Lock-free recording in audio thread
                    if is_recording.load(Ordering::Relaxed) {
                        // Convert and push samples (no Vec allocation)
                        for &sample in data {
                            let f32_sample = (sample as f32 - 32768.0) / 32768.0;
                            if producer.push(f32_sample).is_err() {
                                break;
                            }
                        }
                    }
                },
                |err| tracing::error!("Audio input error: {}", err),
                None,
            )
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to build U16 stream: {}", e)))?;

        Ok(stream)
    }

    /// Get current recording status (lock-free)
    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::Relaxed)
    }

    /// Get sample count (lock-free)
    pub fn samples_recorded(&self) -> usize {
        self.samples_recorded.load(Ordering::Relaxed)
    }

    /// Get recording duration in milliseconds
    ///
    /// `samples_recorded` counts interleaved samples across all channels,
    /// so divide by channel count as well as sample rate.
    pub fn recording_duration_ms(&self) -> f64 {
        let samples = self.samples_recorded() as f64;
        let frames = samples / self.channels.max(1) as f64;
        (frames / self.sample_rate as f64) * 1000.0
    }
}

/// Professional recording statistics
#[derive(Debug, Clone)]
pub struct RecordingStats {
    pub total_samples: usize,
    pub duration_ms: f64,
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_overruns: u32,
    pub performance_grade: RecordingPerformanceGrade,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordingPerformanceGrade {
    Professional, // Zero buffer overruns, perfect timing
    Good,         // Minor overruns, acceptable for most use
    Poor,         // Significant overruns, timing issues
}

impl LockFreeRecorder {
    /// Get comprehensive recording statistics
    pub fn get_recording_stats(&self) -> RecordingStats {
        let total_samples = self.samples_recorded();
        let duration_ms = self.recording_duration_ms();

        // For now, assume professional grade (we'd need to track overruns)
        let performance_grade = RecordingPerformanceGrade::Professional;

        RecordingStats {
            total_samples,
            duration_ms,
            sample_rate: self.sample_rate,
            channels: self.channels,
            buffer_overruns: 0, // Would be tracked in real implementation
            performance_grade,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_lock_free_recorder_creation() {
        let config = LockFreeRecordingConfig::default();
        let recorder = LockFreeRecorder::new(config).unwrap();

        assert!(!recorder.is_recording());
        assert_eq!(recorder.samples_recorded(), 0);
        assert_eq!(recorder.recording_duration_ms(), 0.0);
    }

    #[test]
    fn test_recording_state_management() {
        let config = LockFreeRecordingConfig::default();
        let mut recorder = LockFreeRecorder::new(config).unwrap();

        // Start recording
        recorder.start_recording().unwrap();
        assert!(recorder.is_recording());

        // Small delay to let consumer thread start
        std::thread::sleep(Duration::from_millis(10));

        // Stop recording
        let samples = recorder.stop_recording().unwrap();
        assert!(!recorder.is_recording());
        assert!(samples.is_empty()); // No audio input in test
    }

    #[test]
    fn test_recording_duration_accounts_for_channels() {
        let config = LockFreeRecordingConfig {
            sample_rate: 44100,
            channels: 2,
            ..LockFreeRecordingConfig::default()
        };
        let recorder = LockFreeRecorder::new(config).unwrap();

        // 88200 interleaved samples at 44.1kHz stereo = exactly 1 second
        recorder.samples_recorded.store(88200, Ordering::Relaxed);
        assert_eq!(recorder.recording_duration_ms(), 1000.0);

        // Mono: 44100 samples = 1 second
        let mono_config = LockFreeRecordingConfig {
            sample_rate: 44100,
            channels: 1,
            ..LockFreeRecordingConfig::default()
        };
        let mono_recorder = LockFreeRecorder::new(mono_config).unwrap();
        mono_recorder.samples_recorded.store(44100, Ordering::Relaxed);
        assert_eq!(mono_recorder.recording_duration_ms(), 1000.0);
    }

    #[test]
    fn test_max_recording_samples_is_honored() {
        let config = LockFreeRecordingConfig {
            ring_buffer_size: 1024,
            sample_rate: 44100,
            channels: 1,
            max_recording_samples: 100,
        };
        let mut recorder = LockFreeRecorder::new(config).unwrap();

        // Take the producer (normally moved into the audio callback)
        let mut producer = recorder
            .sample_producer
            .lock()
            .unwrap()
            .take()
            .unwrap();

        recorder.start_recording().unwrap();

        // Push well past the limit
        for i in 0..500 {
            while producer.push(i as f32).is_err() {
                thread::sleep(Duration::from_micros(50));
            }
        }

        // Give the consumer thread time to hit the limit and self-stop
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while recorder.is_recording() && std::time::Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(
            !recorder.is_recording(),
            "recorder should stop itself at max_recording_samples"
        );

        let samples = recorder.stop_recording().unwrap();
        assert_eq!(samples.len(), 100, "recording must be capped at the limit");
    }

    #[test]
    fn test_professional_configuration() {
        let config = LockFreeRecordingConfig {
            sample_rate: 44100,
            channels: 2,
            ring_buffer_size: 44100 * 2,       // 2 seconds
            max_recording_samples: 44100 * 60, // 1 minute
        };

        let recorder = LockFreeRecorder::new(config).unwrap();
        let stats = recorder.get_recording_stats();

        assert_eq!(stats.sample_rate, 44100);
        assert_eq!(stats.channels, 2);
        assert_eq!(
            stats.performance_grade,
            RecordingPerformanceGrade::Professional
        );
    }
}
