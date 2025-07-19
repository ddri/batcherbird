use crate::{Result, BatcherbirdError};
use crate::midi::MidiManager;
use crate::audio::AudioManager;
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

    pub async fn sample_single_note(
        &self,
        midi_conn: &mut MidiOutputConnection,
        note: u8,
    ) -> Result<Sample> {
        println!("🎵 Sampling note {} ({})", note, Self::note_to_name(note));
        
        let total_duration = self.config.pre_delay_ms 
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

    pub async fn sample_note_range(
        &self,
        midi_conn: &mut MidiOutputConnection,
        start_note: u8,
        end_note: u8,
    ) -> Result<Vec<Sample>> {
        let mut samples = Vec::new();
        
        println!("🎹 Sampling note range: {} to {} ({} notes)", 
            Self::note_to_name(start_note), 
            Self::note_to_name(end_note), 
            end_note - start_note + 1
        );
        
        for note in start_note..=end_note {
            let sample = self.sample_single_note(midi_conn, note).await?;
            samples.push(sample);
            
            // Brief pause between samples
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        println!("✅ Completed sampling {} notes", samples.len());
        Ok(samples)
    }

    fn note_to_name(note: u8) -> String {
        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let octave = (note / 12).saturating_sub(1);
        let note_name = note_names[(note % 12) as usize];
        format!("{}{}", note_name, octave)
    }
}