use crate::{BatcherbirdError, Result};
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    SampleFormat, StreamConfig,
};
use hound::WavReader;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};

/// Professional audio playback engine following the heartbeat pattern
/// Stream runs continuously in a dedicated thread, controlled by atomic state
pub struct AudioPlayback {
    audio_manager: crate::audio::AudioManager,
    current_sample: Arc<Mutex<Option<PlaybackSample>>>,
    playback_position: Arc<AtomicU64>, // Sample position
    is_playing: Arc<AtomicBool>,
    // Thread handle for the audio thread - no Stream storage!
    audio_thread: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
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
            audio_thread: Arc::new(Mutex::new(None)),
        })
    }

    /// Load a WAV file for playback
    pub fn load_sample(&self, file_path: &str) -> Result<String> {
        // Verify file exists
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(BatcherbirdError::Audio(format!(
                "File not found: {}",
                file_path
            )));
        }

        // Open WAV file
        let reader = WavReader::open(file_path)
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to open WAV file: {}", e)))?;

        let spec = reader.spec();
        let sample_rate = spec.sample_rate;
        let channels = spec.channels;

        // Convert all samples to f32
        let samples: Vec<f32> = match spec.bits_per_sample {
            16 => reader
                .into_samples::<i16>()
                .filter_map(|r| r.ok())
                .map(|s| s as f32 / i16::MAX as f32)
                .collect(),
            24 => reader
                .into_samples::<i32>()
                .filter_map(|r| r.ok())
                .map(|s| (s >> 8) as f32 / (1 << 23) as f32)
                .collect(),
            32 => reader
                .into_samples::<f32>()
                .filter_map(|r| r.ok())
                .collect(),
            _ => {
                return Err(BatcherbirdError::Audio(format!(
                    "Unsupported bit depth: {}",
                    spec.bits_per_sample
                )));
            }
        };

        let total_samples = samples.len() / channels as usize;
        let duration = total_samples as f64 / sample_rate as f64;

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

        Ok(format!(
            "Loaded: {} ({:.2}s, {}Hz, {}ch)",
            path.file_name().unwrap_or_default().to_string_lossy(),
            duration,
            sample_rate,
            channels
        ))
    }

    /// Initialize the audio engine (creates the continuous audio thread)
    pub fn initialize_audio_engine(&self) -> Result<String> {
        // Check if already initialized
        let mut thread_guard = self.audio_thread.lock().unwrap();
        if thread_guard.is_some() {
            return Ok("Audio engine already initialized".to_string());
        }

        // Get output device
        let device = self.audio_manager.get_default_output_device()?;

        // Get device config
        let config = device
            .default_output_config()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to get output config: {}", e)))?;

        // Clone Arc references for the audio thread
        let current_sample = Arc::clone(&self.current_sample);
        let playback_position = Arc::clone(&self.playback_position);
        let is_playing = Arc::clone(&self.is_playing);

        // Spawn dedicated audio thread (following the heartbeat pattern)
        let handle = std::thread::Builder::new()
            .name("batcherbird-audio-playback".to_string())
            .spawn(move || {
                // Build and run the stream in this thread
                if let Err(e) = Self::run_audio_thread(
                    device,
                    config,
                    current_sample,
                    playback_position,
                    is_playing,
                ) {
                    eprintln!("Audio playback thread error: {}", e);
                }
            })
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to spawn audio thread: {}", e)))?;

        *thread_guard = Some(handle);

        Ok("Audio engine initialized".to_string())
    }

    /// Start playback (just sets the atomic flag)
    pub fn start_playback(&self) -> Result<String> {
        // Ensure audio engine is initialized
        self.initialize_audio_engine()?;

        // Check if sample is loaded
        let sample = self.current_sample.lock().unwrap();
        if sample.is_none() {
            return Err(BatcherbirdError::Audio("No audio file loaded".to_string()));
        }
        drop(sample);

        // Just set the playing flag - the audio thread will see it
        self.is_playing.store(true, Ordering::Relaxed);

        Ok("Playback started".to_string())
    }

    /// Stop playback
    pub fn stop_playback(&self) -> Result<String> {
        // Set playing flag to false
        self.is_playing.store(false, Ordering::Relaxed);

        // Reset position to beginning
        self.playback_position.store(0, Ordering::Relaxed);

        Ok("Playback stopped".to_string())
    }

    /// Pause playback (keeps position)
    pub fn pause_playback(&self) -> Result<String> {
        // Just set playing flag to false - position is maintained
        self.is_playing.store(false, Ordering::Relaxed);

        Ok("Playback paused".to_string())
    }

    /// Seek to position (0.0 to 1.0)
    pub fn seek_to_position(&self, position: f64) -> Result<String> {
        let position = position.clamp(0.0, 1.0);

        let sample = self.current_sample.lock().unwrap();
        if let Some(sample) = sample.as_ref() {
            let new_position = (position * sample.total_samples as f64) as u64;
            self.playback_position
                .store(new_position, Ordering::Relaxed);

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

    /// Run the audio thread with continuous stream (heartbeat pattern)
    fn run_audio_thread(
        device: cpal::Device,
        config: cpal::SupportedStreamConfig,
        current_sample: Arc<Mutex<Option<PlaybackSample>>>,
        playback_position: Arc<AtomicU64>,
        is_playing: Arc<AtomicBool>,
    ) -> Result<()> {
        let _sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;

        let stream_config = StreamConfig {
            channels: config.channels(),
            sample_rate: config.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = match config.sample_format() {
            SampleFormat::F32 => {
                device
                    .build_output_stream(
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
                                let mut position =
                                    playback_position.load(Ordering::Relaxed) as usize;

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
                                            *out_sample =
                                                sample.audio_data.get(idx).copied().unwrap_or(0.0);
                                        }
                                    } else if sample.channels == 1 && channels == 2 {
                                        // Mono to stereo
                                        let mono_sample =
                                            sample.audio_data.get(position).copied().unwrap_or(0.0);
                                        frame[0] = mono_sample;
                                        frame[1] = mono_sample;
                                    } else if sample.channels == 2 && channels == 1 {
                                        // Stereo to mono
                                        let left = sample
                                            .audio_data
                                            .get(position * 2)
                                            .copied()
                                            .unwrap_or(0.0);
                                        let right = sample
                                            .audio_data
                                            .get(position * 2 + 1)
                                            .copied()
                                            .unwrap_or(0.0);
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
                    )
                    .map_err(|e| {
                        BatcherbirdError::Audio(format!("Failed to build output stream: {}", e))
                    })?
            }
            _ => {
                return Err(BatcherbirdError::Audio(format!(
                    "Unsupported output format: {:?}. Only F32 is supported for now.",
                    config.sample_format()
                )));
            }
        };

        // Start the stream - it will run forever (heartbeat pattern)
        stream
            .play()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to start stream: {}", e)))?;

        // Keep the thread alive - the stream runs in the background
        // In a real implementation, you might want a way to signal shutdown
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            // Could check for a shutdown signal here
        }
    }
}
