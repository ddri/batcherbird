use crate::{Result, BatcherbirdError};
use crate::midi::MidiManager;
use crate::audio::AudioManager;
use crate::detection::{SampleDetector, DetectionConfig, DetectionResult};
use midir::MidiOutputConnection;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use tokio::time::Instant;
use cpal::traits::{DeviceTrait, StreamTrait};

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
}

impl SamplingEngine {
    pub fn new(config: SamplingConfig) -> Result<Self> {
        let audio_manager = AudioManager::new()?;
        
        Ok(Self {
            audio_manager,
            config,
        })
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
        let stream = self.build_recording_stream(&device, &config, samples_clone, complete_clone)?;
        
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

    fn build_recording_stream(
        &self,
        device: &cpal::Device,
        config: &cpal::SupportedStreamConfig,
        samples: Arc<Mutex<Vec<f32>>>,
        complete: Arc<Mutex<bool>>,
    ) -> Result<cpal::Stream> {
        use cpal::{SampleFormat, StreamConfig};

        let stream_config = StreamConfig {
            channels: config.channels(),
            sample_rate: config.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = match config.sample_format() {
            SampleFormat::F32 => {
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
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
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let mut audio_samples = samples.lock().unwrap();
                        let recording_complete = complete.lock().unwrap();
                        
                        if !*recording_complete {
                            for &sample in data {
                                audio_samples.push(sample as f32 / i16::MAX as f32);
                            }
                        }
                    },
                    |err| eprintln!("Audio input error: {}", err),
                    None,
                ).map_err(|e| BatcherbirdError::Audio(format!("Failed to build input stream: {}", e)))?
            }
            SampleFormat::U16 => {
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        let mut audio_samples = samples.lock().unwrap();
                        let recording_complete = complete.lock().unwrap();
                        
                        if !*recording_complete {
                            for &sample in data {
                                audio_samples.push((sample as f32 - 32768.0) / 32768.0);
                            }
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
        use cpal::{SampleFormat, StreamConfig};

        let stream_config = StreamConfig {
            channels: config.channels(),
            sample_rate: config.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = match config.sample_format() {
            SampleFormat::F32 => {
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
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
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let recording_flag = recording_active.lock().unwrap();
                        
                        if *recording_flag {
                            let mut audio_samples = samples.lock().unwrap();
                            for &sample in data {
                                audio_samples.push(sample as f32 / i16::MAX as f32);
                            }
                        }
                    },
                    |err| eprintln!("Persistent stream audio input error: {}", err),
                    None,
                ).map_err(|e| BatcherbirdError::Audio(format!("Failed to build persistent input stream: {}", e)))?
            }
            SampleFormat::U16 => {
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        let recording_flag = recording_active.lock().unwrap();
                        
                        if *recording_flag {
                            let mut audio_samples = samples.lock().unwrap();
                            for &sample in data {
                                audio_samples.push((sample as f32 - 32768.0) / 32768.0);
                            }
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
    
    /// Helper method to convert note number to name
    fn note_to_name(note: u8) -> String {
        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let octave = (note / 12).saturating_sub(1);
        let note_name = note_names[(note % 12) as usize];
        format!("{}{}", note_name, octave)
    }
}