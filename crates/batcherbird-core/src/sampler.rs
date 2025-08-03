use crate::{Result, BatcherbirdError};
use crate::midi::MidiManager;
use crate::audio::AudioManager;
use crate::detection::{SampleDetector, DetectionConfig, DetectionResult};
use crate::loop_detection::{LoopDetector, LoopDetectionConfig, LoopDetectionResult};
use crate::professional_meters::{ProfessionalMeterEngine, ProfessionalMeterReadings};
use midir::MidiOutputConnection;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Instant;
use cpal::traits::{DeviceTrait, StreamTrait};
use rtrb::{RingBuffer, Producer, Consumer};

#[derive(Debug, Clone)]
pub struct SamplingConfig {
    pub note_duration_ms: u64,
    pub release_time_ms: u64,
    pub pre_delay_ms: u64,
    pub post_delay_ms: u64,
    pub midi_channel: u8,
    pub velocity: u8,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            note_duration_ms: 2000,   // 2 second note duration
            release_time_ms: 1000,    // 1 second release capture
            pre_delay_ms: 100,        // 100ms pre-roll
            post_delay_ms: 100,       // 100ms post delay
            midi_channel: 0,          // Channel 1 (0-indexed)
            velocity: 100,            // Default velocity
        }
    }
}

/// Professional audio level detector for real-time metering
#[derive(Debug)]
pub struct AudioLevelDetector {
    peak_level: f32,
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
        // Calculate peak level (instantaneous maximum)
        for &sample in samples {
            let abs_sample = sample.abs();
            if abs_sample > self.peak_level {
                self.peak_level = abs_sample;
            }
            
            // Accumulate for RMS calculation
            self.rms_accumulator += sample * sample;
            self.rms_sample_count += 1;
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
        let _professional_readings = self.professional_meters.process_samples(samples);
        
        AudioLevels {
            peak: self.peak_level,
            rms: rms_level,
            peak_db: if self.peak_level > 0.0 { 20.0 * self.peak_level.log10() } else { -60.0 },
            rms_db: if rms_level > 0.0 { 20.0 * rms_level.log10() } else { -60.0 },
        }
    }
    
    /// Get professional meter readings (Epic 3.1.1 - Professional Meter Engine)
    pub fn get_professional_readings(&mut self, samples: &[f32]) -> ProfessionalMeterReadings {
        self.professional_meters.process_samples(samples)
    }
    
    /// Reset peak level (called periodically for peak hold behavior)
    pub fn reset_peak(&mut self) {
        self.peak_level = 0.0;
    }
}

/// Real-time audio levels (thread-safe)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioLevels {
    pub peak: f32,      // Linear peak level (0.0 to 1.0)
    pub rms: f32,       // RMS level (0.0 to 1.0)
    pub peak_db: f32,   // Peak in dBFS
    pub rms_db: f32,    // RMS in dBFS
}

impl Default for AudioLevels {
    fn default() -> Self {
        Self {
            peak: 0.0,
            rms: 0.0,
            peak_db: -60.0,
            rms_db: -60.0,
        }
    }
}

/// Real-time visualization data chunk for waveform display
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VizChunk {
    pub peak: f32,        // Peak amplitude for this chunk (0.0 to 1.0)
    pub rms: f32,         // RMS level for this chunk (0.0 to 1.0)
    pub peak_db: f32,     // Peak in dBFS
    pub rms_db: f32,      // RMS in dBFS
    pub timestamp: u64,   // Timestamp in samples since recording start
    pub chunk_size: usize,// Number of samples in this chunk
}

impl VizChunk {
    /// Create a new visualization chunk from audio samples
    pub fn from_samples(samples: &[f32], timestamp: u64) -> Self {
        let chunk_size = samples.len();
        
        // Calculate peak
        let peak = samples.iter()
            .map(|&sample| sample.abs())
            .fold(0.0f32, f32::max);
        
        // Calculate RMS
        let rms = if chunk_size > 0 {
            let sum_squares: f32 = samples.iter()
                .map(|&sample| sample * sample)
                .sum();
            (sum_squares / chunk_size as f32).sqrt()
        } else {
            0.0
        };
        
        // Convert to dB
        let peak_db = if peak > 0.0 { 20.0 * peak.log10() } else { -60.0 };
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

/// Thread-safe level meter state using atomic operations
#[derive(Debug)]
pub struct LevelMeterState {
    input_peak: AtomicU32,      // Store f32 as u32 bits for atomicity
    input_rms: AtomicU32,
    input_peak_db: AtomicU32,
    input_rms_db: AtomicU32,
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
            last_update: std::time::Instant::now(),
        }
    }
    
    /// Update levels from audio thread (atomic write)
    pub fn update_levels(&self, levels: AudioLevels) {
        self.input_peak.store(f32::to_bits(levels.peak), Ordering::Relaxed);
        self.input_rms.store(f32::to_bits(levels.rms), Ordering::Relaxed);
        self.input_peak_db.store(f32::to_bits(levels.peak_db), Ordering::Relaxed);
        self.input_rms_db.store(f32::to_bits(levels.rms_db), Ordering::Relaxed);
    }
    
    /// Get current levels for UI (atomic read)
    pub fn get_levels(&self) -> AudioLevels {
        AudioLevels {
            peak: f32::from_bits(self.input_peak.load(Ordering::Relaxed)),
            rms: f32::from_bits(self.input_rms.load(Ordering::Relaxed)),
            peak_db: f32::from_bits(self.input_peak_db.load(Ordering::Relaxed)),
            rms_db: f32::from_bits(self.input_rms_db.load(Ordering::Relaxed)),
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
}

/// Ring buffer size for visualization data
/// At 60fps, we need ~1 second of buffer = 60 chunks
const VIZ_RING_BUFFER_SIZE: usize = 64;

impl SamplingEngine {
    pub fn new(config: SamplingConfig) -> Result<Self> {
        let audio_manager = AudioManager::new()?;
        
        Ok(Self {
            audio_manager,
            config,
            level_meter_state: Arc::new(LevelMeterState::new()),
        })
    }
    
    /// Get current audio levels for UI (thread-safe)
    pub fn get_audio_levels(&self) -> AudioLevels {
        self.level_meter_state.get_levels()
    }
    
    
    /// Start persistent audio monitoring stream with optional playthrough
    pub fn start_monitoring_stream_with_playthrough(&self, enable_playthrough: bool) -> Result<(cpal::Stream, Option<cpal::Stream>)> {
        println!("🎛️ Starting audio monitoring stream (playthrough: {})", enable_playthrough);
        
        let input_device = self.audio_manager.get_default_input_device()?;
        let input_config = input_device.default_input_config()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to get input config: {}", e)))?;
        
        let sample_rate = input_config.sample_rate().0;
        let _channels = input_config.channels();
        let level_state = Arc::clone(&self.level_meter_state);
            
        // Create shared buffer for input->output if playthrough enabled
        let shared_buffer: Option<Arc<Mutex<Vec<f32>>>> = if enable_playthrough {
            Some(Arc::new(Mutex::new(Vec::new())))
        } else {
            None
        };
        
        use cpal::SampleFormat;
        let input_stream_config = AudioManager::get_standard_stream_config();
        
        // Build input stream
        let input_stream = match input_config.sample_format() {
            SampleFormat::F32 => {
                let level_state_clone = Arc::clone(&level_state);
                let shared_buffer_clone = shared_buffer.clone();
                let mut level_detector = AudioLevelDetector::new(sample_rate);
                
                input_device.build_input_stream(
                    &input_stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        // Level detection for UI meters
                        let levels = level_detector.process_samples(data);
                        level_state_clone.update_levels(levels);
                        
                        // Store for playthrough if enabled
                        if let Some(ref buffer) = shared_buffer_clone {
                            if let Ok(mut buf) = buffer.try_lock() {
                                buf.clear();
                                buf.extend_from_slice(data);
                            }
                        }
                    },
                    |err| eprintln!("Audio input error: {}", err),
                    None,
                ).map_err(|e| BatcherbirdError::Audio(format!("Failed to build input stream: {}", e)))?
            }
            _ => {
                return Err(BatcherbirdError::Audio("Playthrough currently only supports F32 format".to_string()));
            }
        };
        
        // Build output stream for playthrough if enabled
        let output_stream = if enable_playthrough {
            let output_device = self.audio_manager.get_default_output_device()?;
            let output_config = output_device.default_output_config()
                .map_err(|e| BatcherbirdError::Audio(format!("Failed to get output config: {}", e)))?;
            
            let output_stream_config = AudioManager::get_standard_stream_config();
            
            let shared_buffer_output = shared_buffer.unwrap(); // Safe because we created it above
            
            let stream = match output_config.sample_format() {
                SampleFormat::F32 => {
                    output_device.build_output_stream(
                        &output_stream_config,
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                            // Copy input to output (playthrough)
                            if let Ok(buf) = shared_buffer_output.try_lock() {
                                if !buf.is_empty() {
                                    let copy_len = data.len().min(buf.len());
                                    data[..copy_len].copy_from_slice(&buf[..copy_len]);
                                    // Fill remainder with silence if needed
                                    for sample in data[copy_len..].iter_mut() {
                                        *sample = 0.0;
                                    }
                                } else {
                                    // Silence if no input data
                                    data.fill(0.0);
                                }
                            } else {
                                // Silence if can't lock buffer
                                data.fill(0.0);
                            }
                        },
                        |err| eprintln!("Audio output error: {}", err),
                        None,
                    ).map_err(|e| BatcherbirdError::Audio(format!("Failed to build output stream: {}", e)))?
                }
                _ => {
                    return Err(BatcherbirdError::Audio("Playthrough output currently only supports F32 format".to_string()));
                }
            };
            Some(stream)
        } else {
            None
        };
        
        println!("✅ Audio monitoring stream created (with playthrough: {})", enable_playthrough);
        Ok((input_stream, output_stream))
    }

    /// Start persistent audio monitoring stream (separate from recording)
    pub fn start_monitoring_stream(&self) -> Result<cpal::Stream> {
        println!("🎛️ Starting persistent audio monitoring stream");
        
        let device = self.audio_manager.get_default_input_device()?;
        let config = device.default_input_config()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to get input config: {}", e)))?;

        let sample_rate = 44100; // Use our standard sample rate
        let level_state = Arc::clone(&self.level_meter_state);
        
        use cpal::SampleFormat;

        let stream_config = AudioManager::get_standard_stream_config();

        let stream = match config.sample_format() {
            SampleFormat::F32 => {
                let level_state_clone = Arc::clone(&level_state);
                let mut level_detector = AudioLevelDetector::new(sample_rate);
                
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        // Continuous level detection for monitoring
                        let levels = level_detector.process_samples(data);
                        level_state_clone.update_levels(levels);
                    },
                    |err| eprintln!("Audio monitoring error: {}", err),
                    None,
                ).map_err(|e| BatcherbirdError::Audio(format!("Failed to build monitoring stream: {}", e)))?
            }
            SampleFormat::I16 => {
                let level_state_clone = Arc::clone(&level_state);
                let mut level_detector = AudioLevelDetector::new(sample_rate);
                
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        // Convert to f32 for level detection
                        let f32_samples: Vec<f32> = data.iter()
                            .map(|&sample| sample as f32 / i16::MAX as f32)
                            .collect();
                        
                        let levels = level_detector.process_samples(&f32_samples);
                        level_state_clone.update_levels(levels);
                    },
                    |err| eprintln!("Audio monitoring error: {}", err),
                    None,
                ).map_err(|e| BatcherbirdError::Audio(format!("Failed to build monitoring stream: {}", e)))?
            }
            SampleFormat::U16 => {
                let level_state_clone = Arc::clone(&level_state);
                let mut level_detector = AudioLevelDetector::new(sample_rate);
                
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        // Convert to f32 for level detection
                        let f32_samples: Vec<f32> = data.iter()
                            .map(|&sample| (sample as f32 - 32768.0) / 32768.0)
                            .collect();
                        
                        let levels = level_detector.process_samples(&f32_samples);
                        level_state_clone.update_levels(levels);
                    },
                    |err| eprintln!("Audio monitoring error: {}", err),
                    None,
                ).map_err(|e| BatcherbirdError::Audio(format!("Failed to build monitoring stream: {}", e)))?
            }
            _ => {
                return Err(BatcherbirdError::Audio(format!("Unsupported sample format: {:?}", config.sample_format())));
            }
        };

        println!("✅ Persistent audio monitoring stream created");
        Ok(stream)
    }
    

    /// Blocking interface for Tauri GUI layer (follows TAURI_AUDIO_ARCHITECTURE.md)
    pub fn sample_single_note_blocking(
        &self,
        midi_conn: &mut MidiOutputConnection,
        note: u8,
    ) -> Result<Sample> {
        // Create dedicated runtime for this blocking operation
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to create runtime: {}", e)))?;
        
        // Execute the async operation in blocking context
        rt.block_on(self.sample_single_note_async(midi_conn, note))
    }
    
    /// Blocking interface with real-time visualization support
    pub fn sample_single_note_with_viz_blocking(
        &self,
        midi_conn: &mut MidiOutputConnection,
        note: u8,
    ) -> Result<(Sample, Consumer<VizChunk>)> {
        // Create dedicated runtime for this blocking operation
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to create runtime: {}", e)))?;
        
        // Execute the async operation in blocking context
        rt.block_on(self.sample_single_note_with_viz_async(midi_conn, note))
    }

    /// Internal async implementation (Core Audio Engine)
    async fn sample_single_note_async(
        &self,
        midi_conn: &mut MidiOutputConnection,
        note: u8,
    ) -> Result<Sample> {
        println!("🎵 Sampling note {} ({})", note, Self::note_to_name(note));
        
        let _total_duration = self.config.pre_delay_ms 
            + self.config.note_duration_ms 
            + self.config.release_time_ms 
            + self.config.post_delay_ms;

        println!("   Pre-delay: {}ms, Note: {}ms, Release: {}ms, Post: {}ms", 
            self.config.pre_delay_ms,
            self.config.note_duration_ms,
            self.config.release_time_ms,
            self.config.post_delay_ms
        );

        // Start recording first
        let audio_samples = Arc::new(Mutex::new(Vec::new()));
        let recording_complete = Arc::new(Mutex::new(false));
        let samples_clone = audio_samples.clone();
        let complete_clone = recording_complete.clone();

        let device = self.audio_manager.get_default_input_device()?;
        let config = device.default_input_config()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to get input config: {}", e)))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        // Build recording stream
        let stream = self.build_recording_stream(&device, &config, samples_clone, complete_clone, None)?;
        
        // Start recording
        stream.play().map_err(|e| BatcherbirdError::Audio(format!("Failed to start stream: {}", e)))?;
        
        let start_time = Instant::now();
        
        // Pre-delay
        if self.config.pre_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.config.pre_delay_ms)).await;
        }
        
        // Safety: Clear any stuck notes on this channel before starting
        MidiManager::send_channel_panic(midi_conn, self.config.midi_channel)?;
        
        // Brief delay after panic to ensure hardware processes it
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Send MIDI note on
        let midi_start = Instant::now();
        MidiManager::send_note_on(midi_conn, self.config.midi_channel, note, self.config.velocity)?;
        
        // Wait for note duration
        tokio::time::sleep(Duration::from_millis(self.config.note_duration_ms)).await;
        
        // Send MIDI note off
        MidiManager::send_note_off(midi_conn, self.config.midi_channel, note, self.config.velocity)?;
        let midi_timing = midi_start.elapsed();
        
        // Wait for release
        if self.config.release_time_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.config.release_time_ms)).await;
        }
        
        // Post delay
        if self.config.post_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.config.post_delay_ms)).await;
        }
        
        // Stop recording
        {
            let mut complete = recording_complete.lock().unwrap();
            *complete = true;
        }
        stream.pause().map_err(|e| BatcherbirdError::Audio(format!("Failed to stop stream: {}", e)))?;
        
        let audio_timing = start_time.elapsed();
        let audio_data = audio_samples.lock().unwrap().clone();
        
        println!("   ✅ Captured {} samples in {:.1}ms", audio_data.len(), audio_timing.as_millis());
        
        Ok(Sample {
            note,
            velocity: self.config.velocity,
            audio_data,
            sample_rate,
            channels,
            recorded_at: std::time::SystemTime::now(),
            midi_timing,
            audio_timing,
        })
    }

    /// Internal async implementation with real-time visualization support
    async fn sample_single_note_with_viz_async(
        &self,
        midi_conn: &mut MidiOutputConnection,
        note: u8,
    ) -> Result<(Sample, Consumer<VizChunk>)> {
        println!("🎵 Sampling note {} ({}) with real-time visualization", note, Self::note_to_name(note));
        
        // Create visualization ring buffer
        let (viz_producer, viz_consumer) = RingBuffer::<VizChunk>::new(VIZ_RING_BUFFER_SIZE);
        
        let _total_duration = self.config.pre_delay_ms 
            + self.config.note_duration_ms 
            + self.config.release_time_ms 
            + self.config.post_delay_ms;

        println!("   Pre-delay: {}ms, Note: {}ms, Release: {}ms, Post: {}ms", 
            self.config.pre_delay_ms,
            self.config.note_duration_ms,
            self.config.release_time_ms,
            self.config.post_delay_ms
        );

        // Start recording first
        let audio_samples = Arc::new(Mutex::new(Vec::new()));
        let recording_complete = Arc::new(Mutex::new(false));
        let samples_clone = audio_samples.clone();
        let complete_clone = recording_complete.clone();

        let device = self.audio_manager.get_default_input_device()?;
        let config = device.default_input_config()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to get input config: {}", e)))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        // Build recording stream with visualization
        let stream = self.build_recording_stream(&device, &config, samples_clone, complete_clone, Some(viz_producer))?;
        
        // Start recording
        stream.play().map_err(|e| BatcherbirdError::Audio(format!("Failed to start stream: {}", e)))?;
        
        let start_time = Instant::now();
        
        // Pre-delay
        if self.config.pre_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.config.pre_delay_ms)).await;
        }
        
        // Safety: Clear any stuck notes on this channel before starting
        MidiManager::send_channel_panic(midi_conn, self.config.midi_channel)?;
        
        // Brief delay after panic to ensure hardware processes it
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Send MIDI note on
        let midi_start = Instant::now();
        MidiManager::send_note_on(midi_conn, self.config.midi_channel, note, self.config.velocity)?;
        
        // Wait for note duration
        tokio::time::sleep(Duration::from_millis(self.config.note_duration_ms)).await;
        
        // Send MIDI note off
        MidiManager::send_note_off(midi_conn, self.config.midi_channel, note, self.config.velocity)?;
        let midi_timing = midi_start.elapsed();
        
        // Wait for release
        if self.config.release_time_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.config.release_time_ms)).await;
        }
        
        // Post delay
        if self.config.post_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.config.post_delay_ms)).await;
        }
        
        // Stop recording
        {
            let mut complete = recording_complete.lock().unwrap();
            *complete = true;
        }
        stream.pause().map_err(|e| BatcherbirdError::Audio(format!("Failed to stop stream: {}", e)))?;
        
        let audio_timing = start_time.elapsed();
        let audio_data = audio_samples.lock().unwrap().clone();
        
        println!("   ✅ Captured {} samples in {:.1}ms with real-time visualization", audio_data.len(), audio_timing.as_millis());
        
        let sample = Sample {
            note,
            velocity: self.config.velocity,
            audio_data,
            sample_rate,
            channels,
            recorded_at: std::time::SystemTime::now(),
            midi_timing,
            audio_timing,
        };

        Ok((sample, viz_consumer))
    }

    fn build_recording_stream(
        &self,
        device: &cpal::Device,
        config: &cpal::SupportedStreamConfig,
        samples: Arc<Mutex<Vec<f32>>>,
        complete: Arc<Mutex<bool>>,
        viz_producer: Option<Producer<VizChunk>>,
    ) -> Result<cpal::Stream> {
        let level_state = Arc::clone(&self.level_meter_state);
        let sample_rate = 44100; // Use our standard sample rate
        use cpal::SampleFormat;

        let stream_config = AudioManager::get_standard_stream_config();

        let stream = match config.sample_format() {
            SampleFormat::F32 => {
                let level_state_clone = Arc::clone(&level_state);
                let mut level_detector = AudioLevelDetector::new(sample_rate);
                let mut viz_producer_moved = viz_producer;
                let mut sample_timestamp = 0u64;
                
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        // Professional level detection in audio thread
                        let levels = level_detector.process_samples(data);
                        level_state_clone.update_levels(levels);
                        
                        // Push visualization data to ring buffer (lock-free, never blocks)
                        if let Some(ref mut producer) = viz_producer_moved {
                            let viz_chunk = VizChunk::from_samples(data, sample_timestamp);
                            let _ = producer.push(viz_chunk); // Ignore if buffer is full
                        }
                        sample_timestamp += data.len() as u64;
                        
                        let mut audio_samples = samples.lock().unwrap();
                        let recording_complete = complete.lock().unwrap();
                        
                        if !*recording_complete {
                            audio_samples.extend_from_slice(data);
                        }
                    },
                    |err| eprintln!("Audio input error: {}", err),
                    None,
                ).map_err(|e| BatcherbirdError::Audio(format!("Failed to build input stream: {}", e)))?
            }
            SampleFormat::I16 => {
                let level_state_clone = Arc::clone(&level_state);
                let mut level_detector = AudioLevelDetector::new(sample_rate);
                let mut viz_producer_moved = viz_producer;
                let mut sample_timestamp = 0u64;
                
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        // Convert to f32 for processing
                        let f32_samples: Vec<f32> = data.iter()
                            .map(|&sample| sample as f32 / i16::MAX as f32)
                            .collect();
                        
                        // Professional level detection in audio thread
                        let levels = level_detector.process_samples(&f32_samples);
                        level_state_clone.update_levels(levels);
                        
                        // Push visualization data to ring buffer (lock-free, never blocks)
                        if let Some(ref mut producer) = viz_producer_moved {
                            let viz_chunk = VizChunk::from_samples(&f32_samples, sample_timestamp);
                            let _ = producer.push(viz_chunk); // Ignore if buffer is full
                        }
                        sample_timestamp += f32_samples.len() as u64;
                        
                        let mut audio_samples = samples.lock().unwrap();
                        let recording_complete = complete.lock().unwrap();
                        
                        if !*recording_complete {
                            audio_samples.extend(f32_samples);
                        }
                    },
                    |err| eprintln!("Audio input error: {}", err),
                    None,
                ).map_err(|e| BatcherbirdError::Audio(format!("Failed to build input stream: {}", e)))?
            }
            SampleFormat::U16 => {
                let level_state_clone = Arc::clone(&level_state);
                let mut level_detector = AudioLevelDetector::new(sample_rate);
                let mut viz_producer_moved = viz_producer;
                let mut sample_timestamp = 0u64;
                
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        // Convert to f32 for processing
                        let f32_samples: Vec<f32> = data.iter()
                            .map(|&sample| (sample as f32 - 32768.0) / 32768.0)
                            .collect();
                        
                        // Professional level detection in audio thread
                        let levels = level_detector.process_samples(&f32_samples);
                        level_state_clone.update_levels(levels);
                        
                        // Push visualization data to ring buffer (lock-free, never blocks)
                        if let Some(ref mut producer) = viz_producer_moved {
                            let viz_chunk = VizChunk::from_samples(&f32_samples, sample_timestamp);
                            let _ = producer.push(viz_chunk); // Ignore if buffer is full
                        }
                        sample_timestamp += f32_samples.len() as u64;
                        
                        let mut audio_samples = samples.lock().unwrap();
                        let recording_complete = complete.lock().unwrap();
                        
                        if !*recording_complete {
                            audio_samples.extend(f32_samples);
                        }
                    },
                    |err| eprintln!("Audio input error: {}", err),
                    None,
                ).map_err(|e| BatcherbirdError::Audio(format!("Failed to build input stream: {}", e)))?
            }
            _ => {
                return Err(BatcherbirdError::Audio(format!("Unsupported sample format: {:?}", config.sample_format())));
            }
        };

        Ok(stream)
    }

    /// Build a persistent recording stream for range sampling (Ableton-style)
    /// Unlike build_recording_stream, this uses recording_active flag instead of recording_complete
    fn build_persistent_recording_stream(
        &self,
        device: &cpal::Device,
        config: &cpal::SupportedStreamConfig,
        samples: Arc<Mutex<Vec<f32>>>,
        recording_active: Arc<Mutex<bool>>,
    ) -> Result<cpal::Stream> {
        let level_state = Arc::clone(&self.level_meter_state);
        let sample_rate = 44100; // Use our standard sample rate
        use cpal::SampleFormat;

        let stream_config = AudioManager::get_standard_stream_config();

        let stream = match config.sample_format() {
            SampleFormat::F32 => {
                let level_state_clone = Arc::clone(&level_state);
                let mut level_detector = AudioLevelDetector::new(sample_rate);
                
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        // Always update level meters, even when not recording
                        let levels = level_detector.process_samples(data);
                        level_state_clone.update_levels(levels);
                        
                        let recording_flag = recording_active.lock().unwrap();
                        
                        // Only collect samples when recording is active
                        if *recording_flag {
                            let mut audio_samples = samples.lock().unwrap();
                            audio_samples.extend_from_slice(data);
                        }
                        // Stream stays alive but ignores data when recording_active = false
                    },
                    |err| eprintln!("Persistent stream audio input error: {}", err),
                    None,
                ).map_err(|e| BatcherbirdError::Audio(format!("Failed to build persistent input stream: {}", e)))?
            }
            SampleFormat::I16 => {
                let level_state_clone = Arc::clone(&level_state);
                let mut level_detector = AudioLevelDetector::new(sample_rate);
                
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        // Convert to f32 for level detection
                        let f32_samples: Vec<f32> = data.iter()
                            .map(|&sample| sample as f32 / i16::MAX as f32)
                            .collect();
                        
                        // Always update level meters
                        let levels = level_detector.process_samples(&f32_samples);
                        level_state_clone.update_levels(levels);
                        
                        let recording_flag = recording_active.lock().unwrap();
                        
                        if *recording_flag {
                            let mut audio_samples = samples.lock().unwrap();
                            audio_samples.extend(f32_samples);
                        }
                    },
                    |err| eprintln!("Persistent stream audio input error: {}", err),
                    None,
                ).map_err(|e| BatcherbirdError::Audio(format!("Failed to build persistent input stream: {}", e)))?
            }
            SampleFormat::U16 => {
                let level_state_clone = Arc::clone(&level_state);
                let mut level_detector = AudioLevelDetector::new(sample_rate);
                
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        // Convert to f32 for level detection
                        let f32_samples: Vec<f32> = data.iter()
                            .map(|&sample| (sample as f32 - 32768.0) / 32768.0)
                            .collect();
                        
                        // Always update level meters
                        let levels = level_detector.process_samples(&f32_samples);
                        level_state_clone.update_levels(levels);
                        
                        let recording_flag = recording_active.lock().unwrap();
                        
                        if *recording_flag {
                            let mut audio_samples = samples.lock().unwrap();
                            audio_samples.extend(f32_samples);
                        }
                    },
                    |err| eprintln!("Persistent stream audio input error: {}", err),
                    None,
                ).map_err(|e| BatcherbirdError::Audio(format!("Failed to build persistent input stream: {}", e)))?
            }
            _ => {
                return Err(BatcherbirdError::Audio(format!("Unsupported sample format: {:?}", config.sample_format())));
            }
        };

        Ok(stream)
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

    /// Internal async implementation for range sampling with persistent stream (Ableton-style)
    async fn sample_note_range_async(
        &self,
        midi_conn: &mut MidiOutputConnection,
        start_note: u8,
        end_note: u8,
    ) -> Result<Vec<Sample>> {
        let mut samples = Vec::new();
        let total_notes = end_note - start_note + 1;
        
        println!("🎹 Range sampling with persistent stream: {} to {} ({} notes)", 
            Self::note_to_name(start_note), 
            Self::note_to_name(end_note), 
            total_notes
        );
        
        // === PHASE 1: Setup persistent audio stream (like Ableton's audio engine) ===
        println!("🔧 Setting up persistent audio stream...");
        
        // Safety: Clear any stuck notes before starting range recording session
        println!("🚨 Sending MIDI panic before range recording for safety...");
        MidiManager::send_midi_panic(midi_conn)?;
        tokio::time::sleep(Duration::from_millis(100)).await; // Give hardware time to process
        
        let device = self.audio_manager.get_default_input_device()?;
        let config = device.default_input_config()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to get input config: {}", e)))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        
        // Shared audio buffer - reused for all notes
        let audio_samples = Arc::new(Mutex::new(Vec::new()));
        let recording_active = Arc::new(Mutex::new(false));
        let samples_clone = audio_samples.clone();
        let recording_clone = recording_active.clone();

        // Create ONE stream for entire range (like professional DAWs)
        let stream = self.build_persistent_recording_stream(&device, &config, samples_clone, recording_clone)?;
        
        // Start the persistent stream
        stream.play().map_err(|e| BatcherbirdError::Audio(format!("Failed to start persistent stream: {}", e)))?;
        println!("✅ Persistent audio stream started");
        
        // === PHASE 2: Record each note using the same stream ===
        for (index, note) in (start_note..=end_note).enumerate() {
            println!("🎵 Recording note {}/{}: {} ({})", 
                index + 1, total_notes, Self::note_to_name(note), note);
            
            // Clear the buffer for this note
            {
                let mut buffer = audio_samples.lock().unwrap();
                buffer.clear();
                println!("   🧹 Buffer cleared ({} samples removed)", buffer.len());
            }
            
            // Start recording for this note
            {
                let mut recording = recording_active.lock().unwrap();
                *recording = true;
                println!("   🔴 Recording started");
            }
            
            let start_time = Instant::now();
            
            // Pre-delay
            if self.config.pre_delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.config.pre_delay_ms)).await;
            }
            
            // Safety: Clear any stuck notes on this channel before starting
            MidiManager::send_channel_panic(midi_conn, self.config.midi_channel)?;
            
            // Brief delay after panic to ensure hardware processes it
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            // Send MIDI note on
            let midi_start = Instant::now();
            MidiManager::send_note_on(midi_conn, self.config.midi_channel, note, self.config.velocity)?;
            println!("   🎹 MIDI Note On sent");
            
            // Wait for note duration
            tokio::time::sleep(Duration::from_millis(self.config.note_duration_ms)).await;
            
            // Send MIDI note off
            MidiManager::send_note_off(midi_conn, self.config.midi_channel, note, self.config.velocity)?;
            let midi_timing = midi_start.elapsed();
            println!("   🎹 MIDI Note Off sent");
            
            // Wait for release
            if self.config.release_time_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.config.release_time_ms)).await;
            }
            
            // Post delay
            if self.config.post_delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.config.post_delay_ms)).await;
            }
            
            // Stop recording for this note
            {
                let mut recording = recording_active.lock().unwrap();
                *recording = false;
                println!("   ⏹️ Recording stopped");
            }
            
            let audio_timing = start_time.elapsed();
            
            // Extract recorded audio data
            let audio_data = {
                let buffer = audio_samples.lock().unwrap();
                buffer.clone()
            };
            
            println!("   ✅ Captured {} samples in {:.1}ms", audio_data.len(), audio_timing.as_millis());
            
            // Create sample record
            let sample = Sample {
                note,
                velocity: self.config.velocity,
                audio_data,
                sample_rate,
                channels,
                recorded_at: std::time::SystemTime::now(),
                midi_timing,
                audio_timing,
            };
            
            samples.push(sample);
            
            // Brief pause between notes (hardware stability)
            if index < total_notes as usize - 1 {
                println!("   ⏸️ Pausing 300ms between notes...");
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
        }
        
        // === PHASE 3: Clean shutdown of persistent stream ===
        println!("🔧 Shutting down persistent stream...");
        stream.pause().map_err(|e| BatcherbirdError::Audio(format!("Failed to stop persistent stream: {}", e)))?;
        drop(stream); // Explicit cleanup
        println!("✅ Persistent stream shut down cleanly");
        
        // Safety: Final MIDI panic to ensure no stuck notes (professional practice)
        println!("🚨 Final MIDI panic after range recording for safety...");
        MidiManager::send_midi_panic(midi_conn)?;
        
        println!("🎉 Range sampling complete: {} notes recorded successfully", samples.len());
        Ok(samples)
    }

    fn note_to_name(note: u8) -> String {
        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let octave = (note / 12).saturating_sub(1);
        let note_name = note_names[(note % 12) as usize];
        format!("{}{}", note_name, octave)
    }
}

impl Sample {
    /// Apply sample detection and trimming to this sample
    pub fn apply_detection(&mut self, config: DetectionConfig) -> Result<DetectionResult> {
        let detector = SampleDetector::new(config);
        let detection_result = detector.detect_boundaries(&self.audio_data, self.sample_rate)?;
        
        if detection_result.success {
            println!("🎵 Applying detection to {} sample ({})", 
                Self::note_to_name(self.note), self.note);
            
            // Trim the audio data
            self.audio_data = detector.trim_audio(&self.audio_data, &detection_result);
            
            println!("   Sample trimmed successfully");
        } else {
            println!("⚠️  Detection failed for {} sample ({}): {}", 
                Self::note_to_name(self.note), self.note,
                detection_result.failure_reason.as_deref().unwrap_or("Unknown reason"));
        }
        
        Ok(detection_result)
    }
    
    /// Apply loop detection to find optimal loop points in the sample
    pub fn apply_loop_detection(&mut self, config: LoopDetectionConfig) -> Result<LoopDetectionResult> {
        println!("🔄 Applying loop detection to {} sample ({})", 
            Self::note_to_name(self.note), self.note);
        
        let detector = LoopDetector::new(config);
        let loop_result = detector.detect_loop_points(&self.audio_data, self.sample_rate);
        
        if loop_result.success {
            if let Some(ref candidate) = loop_result.best_candidate {
                println!("   ✅ Loop detected: {:.2}s length, quality {:.3}", 
                    candidate.length_samples as f32 / self.sample_rate as f32,
                    candidate.quality_score);
                
                // Optionally apply the loop with crossfading
                if let Err(e) = detector.apply_loop_with_crossfade(
                    &mut self.audio_data, 
                    candidate, 
                    self.sample_rate
                ) {
                    println!("   ⚠️ Failed to apply crossfade: {}", e);
                }
            }
        } else {
            println!("   ⚠️ Loop detection failed: {}", 
                loop_result.failure_reason.as_deref().unwrap_or("Unknown reason"));
        }
        
        Ok(loop_result)
    }
    
    /// Helper method to convert note number to name
    fn note_to_name(note: u8) -> String {
        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let octave = (note / 12).saturating_sub(1);
        let note_name = note_names[(note % 12) as usize];
        format!("{}{}", note_name, octave)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rtrb::RingBuffer;
    use std::thread;
    use std::time::Duration;

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
            
            for _ in 0..60 {  // 60 iterations = 1 second at 60fps
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
        assert!(chunks_received > 0, "Should receive some visualization chunks");
        assert!(chunks_received < 1000, "Should not receive all chunks due to 60fps consumption");
        
        println!("✅ Ring buffer stress test passed: {} chunks received", chunks_received);
    }
}