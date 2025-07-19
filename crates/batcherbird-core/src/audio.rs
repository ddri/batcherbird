use crate::{Result, BatcherbirdError};
use cpal::{Host, Stream, StreamConfig, SampleFormat, traits::{DeviceTrait, HostTrait, StreamTrait}};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct AudioManager {
    host: Host,
}

impl AudioManager {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        Ok(Self { host })
    }

    pub fn list_input_devices(&self) -> Result<Vec<String>> {
        let mut devices = Vec::new();
        
        let input_devices = self.host.input_devices()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to enumerate input devices: {}", e)))?;
            
        for device in input_devices {
            if let Ok(name) = device.name() {
                devices.push(name);
            }
        }
        
        Ok(devices)
    }

    pub fn list_output_devices(&self) -> Result<Vec<String>> {
        let mut devices = Vec::new();
        
        let output_devices = self.host.output_devices()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to enumerate output devices: {}", e)))?;
            
        for device in output_devices {
            if let Ok(name) = device.name() {
                devices.push(name);
            }
        }
        
        Ok(devices)
    }

    pub fn get_default_input_device(&self) -> Result<cpal::Device> {
        // Try to find MiniFuse first, then fall back to default
        let input_devices = self.host.input_devices()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to enumerate input devices: {}", e)))?;
            
        for device in input_devices {
            if let Ok(name) = device.name() {
                if name.contains("MiniFuse") {
                    println!("🎤 Found MiniFuse: {}", name);
                    return Ok(device);
                }
            }
        }
        
        // Fall back to default device
        self.host.default_input_device()
            .ok_or_else(|| BatcherbirdError::Audio("No default input device found".to_string()))
    }

    pub fn record_test_audio(&self, duration_secs: u64) -> Result<Vec<f32>> {
        let device = self.get_default_input_device()?;
        let config = device.default_input_config()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to get input config: {}", e)))?;

        println!("🎤 Recording from: {}", device.name().unwrap_or("Unknown".to_string()));
        println!("   Sample rate: {} Hz", config.sample_rate().0);
        println!("   Channels: {}", config.channels());
        println!("   Format: {:?}", config.sample_format());

        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let expected_samples = (duration_secs * sample_rate as u64 * channels as u64) as usize;
        
        let recorded_samples = Arc::new(Mutex::new(Vec::with_capacity(expected_samples)));
        let samples_clone = recorded_samples.clone();
        let recording_complete = Arc::new(Mutex::new(false));
        let complete_clone = recording_complete.clone();

        let stream = match config.sample_format() {
            SampleFormat::F32 => {
                let stream_config = StreamConfig {
                    channels: config.channels(),
                    sample_rate: config.sample_rate(),
                    buffer_size: cpal::BufferSize::Default,
                };

                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut samples = samples_clone.lock().unwrap();
                        let complete = complete_clone.lock().unwrap();
                        
                        if !*complete {
                            samples.extend_from_slice(data);
                        }
                    },
                    |err| eprintln!("Audio input error: {}", err),
                    None,
                ).map_err(|e| BatcherbirdError::Audio(format!("Failed to build input stream: {}", e)))?
            }
            SampleFormat::I16 => {
                let stream_config = StreamConfig {
                    channels: config.channels(),
                    sample_rate: config.sample_rate(),
                    buffer_size: cpal::BufferSize::Default,
                };

                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let mut samples = samples_clone.lock().unwrap();
                        let complete = complete_clone.lock().unwrap();
                        
                        if !*complete {
                            for &sample in data {
                                samples.push(sample as f32 / i16::MAX as f32);
                            }
                        }
                    },
                    |err| eprintln!("Audio input error: {}", err),
                    None,
                ).map_err(|e| BatcherbirdError::Audio(format!("Failed to build input stream: {}", e)))?
            }
            SampleFormat::U16 => {
                let stream_config = StreamConfig {
                    channels: config.channels(),
                    sample_rate: config.sample_rate(),
                    buffer_size: cpal::BufferSize::Default,
                };

                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        let mut samples = samples_clone.lock().unwrap();
                        let complete = complete_clone.lock().unwrap();
                        
                        if !*complete {
                            for &sample in data {
                                samples.push((sample as f32 - 32768.0) / 32768.0);
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

        // Start recording
        stream.play().map_err(|e| BatcherbirdError::Audio(format!("Failed to start stream: {}", e)))?;
        
        println!("🔴 Recording for {} seconds... (make some noise!)", duration_secs);
        
        // Record for specified duration
        std::thread::sleep(Duration::from_secs(duration_secs));
        
        // Stop recording
        {
            let mut complete = recording_complete.lock().unwrap();
            *complete = true;
        }
        
        stream.pause().map_err(|e| BatcherbirdError::Audio(format!("Failed to stop stream: {}", e)))?;
        
        let samples = recorded_samples.lock().unwrap().clone();
        println!("✅ Recording complete! Captured {} samples", samples.len());
        
        Ok(samples)
    }

    pub fn analyze_audio_samples(samples: &[f32]) -> (f32, f32, f32) {
        if samples.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let mut sum_squares = 0.0;
        let mut peak = 0.0;
        
        for &sample in samples {
            let abs_sample = sample.abs();
            if abs_sample > peak {
                peak = abs_sample;
            }
            sum_squares += sample * sample;
        }
        
        let rms = (sum_squares / samples.len() as f32).sqrt();
        let rms_db = if rms > 0.0 { 20.0 * rms.log10() } else { -100.0 };
        let peak_db = if peak > 0.0 { 20.0 * peak.log10() } else { -100.0 };
        
        (rms, rms_db, peak_db)
    }
}