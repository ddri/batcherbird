use crate::{Result, BatcherbirdError};
use cpal::{traits::{DeviceTrait, StreamTrait}, StreamConfig, SampleFormat};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::path::Path;
use hound::WavReader;

/// Professional audio playback engine following SamplingEngine patterns
pub struct AudioPlayback {
    audio_manager: crate::audio::AudioManager,
    current_sample: Arc<Mutex<Option<PlaybackSample>>>,
    playback_position: Arc<AtomicU64>, // Sample position
    is_playing: Arc<AtomicBool>,
    playback_stream: Arc<Mutex<Option<cpal::Stream>>>,
}

/// Loaded audio sample ready for playback
#[derive(Debug, Clone)]
pub struct PlaybackSample {
    pub audio_data: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub file_path: String,
    pub total_samples: usize, // Per channel
}

impl AudioPlayback {
    pub fn new() -> Result<Self> {
        let audio_manager = crate::audio::AudioManager::new()?;
        
        Ok(Self {
            audio_manager,
            current_sample: Arc::new(Mutex::new(None)),
            playback_position: Arc::new(AtomicU64::new(0)),
            is_playing: Arc::new(AtomicBool::new(false)),
            playback_stream: Arc::new(Mutex::new(None)),
        })
    }
    
    /// Load a WAV file for playback
    pub fn load_sample(&self, file_path: &str) -> Result<String> {
        println!("🎵 Loading audio file for playback: {}", file_path);
        
        // Verify file exists
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(BatcherbirdError::Audio(format!("File not found: {}", file_path)));
        }
        
        // Open WAV file
        let reader = WavReader::open(file_path)
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to open WAV file: {}", e)))?;
        
        let spec = reader.spec();
        let sample_rate = spec.sample_rate;
        let channels = spec.channels;
        
        println!("   📊 Sample rate: {} Hz, Channels: {}, Bit depth: {}", 
                 sample_rate, channels, spec.bits_per_sample);
        
        // Convert all samples to f32
        let samples: Vec<f32> = match spec.bits_per_sample {
            16 => {
                reader.into_samples::<i16>()
                    .filter_map(Result::ok)
                    .map(|s| s as f32 / i16::MAX as f32)
                    .collect()
            },
            24 => {
                reader.into_samples::<i32>()
                    .filter_map(Result::ok)
                    .map(|s| (s >> 8) as f32 / (1 << 23) as f32)
                    .collect()
            },
            32 => {
                reader.into_samples::<f32>()
                    .filter_map(Result::ok)
                    .collect()
            },
            _ => {
                return Err(BatcherbirdError::Audio(
                    format!("Unsupported bit depth: {}", spec.bits_per_sample)
                ));
            }
        };
        
        let total_samples = samples.len() / channels as usize;
        let duration = total_samples as f64 / sample_rate as f64;
        
        println!("   ✅ Loaded {} samples ({:.2} seconds)", samples.len(), duration);
        
        // Store the loaded sample
        let playback_sample = PlaybackSample {
            audio_data: samples,
            sample_rate,
            channels,
            file_path: file_path.to_string(),
            total_samples,
        };
        
        *self.current_sample.lock().unwrap() = Some(playback_sample);
        
        // Reset playback position
        self.playback_position.store(0, Ordering::Relaxed);
        
        Ok(format!("Loaded: {} ({:.2}s, {}Hz, {}ch)", 
                   path.file_name().unwrap_or_default().to_string_lossy(),
                   duration, sample_rate, channels))
    }
    
    /// Start or resume playback from current position
    pub fn start_playback(&self) -> Result<String> {
        println!("▶️ Starting audio playback");
        
        // Check if sample is loaded
        let sample = self.current_sample.lock().unwrap();
        if sample.is_none() {
            return Err(BatcherbirdError::Audio("No audio file loaded".to_string()));
        }
        let sample = sample.as_ref().unwrap().clone();
        drop(sample); // Release lock early
        
        // Check if already playing
        if self.is_playing.load(Ordering::Relaxed) {
            return Ok("Already playing".to_string());
        }
        
        // Get output device (similar to input device selection in AudioManager)
        let device = self.audio_manager.get_default_output_device()?;
        println!("   🔊 Using output device: {}", device.name().unwrap_or("Unknown".to_string()));
        
        // Get device config
        let config = device.default_output_config()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to get output config: {}", e)))?;
        
        println!("   📊 Output config: {} Hz, {} channels", 
                 config.sample_rate().0, config.channels());
        
        // Build output stream
        let stream = self.build_output_stream(&device, &config)?;
        
        // Start the stream
        stream.play()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to start playback: {}", e)))?;
        
        // Store stream reference
        *self.playback_stream.lock().unwrap() = Some(stream);
        
        // Set playing flag
        self.is_playing.store(true, Ordering::Relaxed);
        
        Ok("Playback started".to_string())
    }
    
    /// Stop playback
    pub fn stop_playback(&self) -> Result<String> {
        println!("⏹️ Stopping audio playback");
        
        // Set playing flag to false
        self.is_playing.store(false, Ordering::Relaxed);
        
        // Drop the stream to stop playback
        *self.playback_stream.lock().unwrap() = None;
        
        // Reset position to beginning
        self.playback_position.store(0, Ordering::Relaxed);
        
        Ok("Playback stopped".to_string())
    }
    
    /// Pause playback (keeps position)
    pub fn pause_playback(&self) -> Result<String> {
        println!("⏸️ Pausing audio playback");
        
        // Set playing flag to false
        self.is_playing.store(false, Ordering::Relaxed);
        
        // Drop the stream but keep position
        *self.playback_stream.lock().unwrap() = None;
        
        Ok("Playback paused".to_string())
    }
    
    /// Seek to position (0.0 to 1.0)
    pub fn seek_to_position(&self, position: f64) -> Result<String> {
        let position = position.clamp(0.0, 1.0);
        
        let sample = self.current_sample.lock().unwrap();
        if let Some(sample) = sample.as_ref() {
            let new_position = (position * sample.total_samples as f64) as u64;
            self.playback_position.store(new_position, Ordering::Relaxed);
            
            println!("⏩ Seeking to position: {:.1}% (sample {})", 
                     position * 100.0, new_position);
            
            Ok(format!("Seeked to {:.1}%", position * 100.0))
        } else {
            Err(BatcherbirdError::Audio("No audio file loaded".to_string()))
        }
    }
    
    /// Get current playback position (0.0 to 1.0)
    pub fn get_playback_position(&self) -> f64 {
        let sample = self.current_sample.lock().unwrap();
        if let Some(sample) = sample.as_ref() {
            let current_pos = self.playback_position.load(Ordering::Relaxed);
            current_pos as f64 / sample.total_samples as f64
        } else {
            0.0
        }
    }
    
    /// Check if currently playing
    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::Relaxed)
    }
    
    /// Build output stream following SamplingEngine patterns
    fn build_output_stream(
        &self,
        device: &cpal::Device,
        config: &cpal::SupportedStreamConfig,
    ) -> Result<cpal::Stream> {
        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        
        // Clone Arc references for the audio callback
        let current_sample = Arc::clone(&self.current_sample);
        let playback_position = Arc::clone(&self.playback_position);
        let is_playing = Arc::clone(&self.is_playing);
        
        let stream_config = StreamConfig {
            channels: config.channels(),
            sample_rate: config.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };
        
        let stream = match config.sample_format() {
            SampleFormat::F32 => {
                device.build_output_stream(
                    &stream_config,
                    move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        // Clear buffer first
                        output.fill(0.0);
                        
                        // Check if we should play
                        if !is_playing.load(Ordering::Relaxed) {
                            return;
                        }
                        
                        // Get current sample
                        let sample_lock = current_sample.lock().unwrap();
                        if let Some(sample) = sample_lock.as_ref() {
                            let mut position = playback_position.load(Ordering::Relaxed) as usize;
                            
                            // Fill output buffer
                            for frame in output.chunks_mut(channels) {
                                if position >= sample.total_samples {
                                    // Reached end of sample
                                    is_playing.store(false, Ordering::Relaxed);
                                    break;
                                }
                                
                                // Handle channel conversion
                                if sample.channels as usize == channels {
                                    // Same channel count - direct copy
                                    for (ch, out_sample) in frame.iter_mut().enumerate() {
                                        let idx = position * sample.channels as usize + ch;
                                        *out_sample = sample.audio_data.get(idx).copied().unwrap_or(0.0);
                                    }
                                } else if sample.channels == 1 && channels == 2 {
                                    // Mono to stereo
                                    let mono_sample = sample.audio_data.get(position).copied().unwrap_or(0.0);
                                    frame[0] = mono_sample;
                                    frame[1] = mono_sample;
                                } else if sample.channels == 2 && channels == 1 {
                                    // Stereo to mono
                                    let left = sample.audio_data.get(position * 2).copied().unwrap_or(0.0);
                                    let right = sample.audio_data.get(position * 2 + 1).copied().unwrap_or(0.0);
                                    frame[0] = (left + right) * 0.5;
                                }
                                
                                position += 1;
                            }
                            
                            // Update position
                            playback_position.store(position as u64, Ordering::Relaxed);
                        }
                    },
                    |err| eprintln!("Audio output error: {}", err),
                    None,
                ).map_err(|e| BatcherbirdError::Audio(format!("Failed to build output stream: {}", e)))?
            }
            _ => {
                return Err(BatcherbirdError::Audio(
                    format!("Unsupported output format: {:?}. Only F32 is supported for now.", 
                            config.sample_format())
                ));
            }
        };
        
        Ok(stream)
    }
}