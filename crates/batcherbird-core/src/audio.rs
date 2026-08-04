use crate::{BatcherbirdError, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Host, SampleFormat, SampleRate, StreamConfig,
};
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

    /// Get our standard audio configuration (44.1kHz, stereo, 16-bit equivalent)
    pub fn get_standard_stream_config() -> StreamConfig {
        StreamConfig {
            channels: 2,                    // Stereo
            sample_rate: SampleRate(44100), // Music industry standard
            buffer_size: cpal::BufferSize::Default,
        }
    }

    pub fn list_input_devices(&self) -> Result<Vec<String>> {
        let mut devices = Vec::new();

        let input_devices = self.host.input_devices().map_err(|e| {
            BatcherbirdError::Audio(format!("Failed to enumerate input devices: {}", e))
        })?;

        for device in input_devices {
            if let Ok(name) = device.name() {
                devices.push(name);
            }
        }

        Ok(devices)
    }

    pub fn list_output_devices(&self) -> Result<Vec<String>> {
        let mut devices = Vec::new();

        let output_devices = self.host.output_devices().map_err(|e| {
            BatcherbirdError::Audio(format!("Failed to enumerate output devices: {}", e))
        })?;

        for device in output_devices {
            if let Ok(name) = device.name() {
                devices.push(name);
            }
        }

        Ok(devices)
    }

    pub fn get_default_input_device(&self) -> Result<cpal::Device> {
        self.host
            .default_input_device()
            .ok_or_else(|| BatcherbirdError::Audio("No default input device found".to_string()))
    }

    pub fn get_default_output_device(&self) -> Result<cpal::Device> {
        self.host
            .default_output_device()
            .ok_or_else(|| BatcherbirdError::Audio("No default output device found".to_string()))
    }

    /// Find an input device by name.
    ///
    /// `None` returns the system default input device. `Some(name)` matches the
    /// enumerated input device names: an exact match is preferred, falling back to
    /// a case-insensitive match. If no device matches, an error listing the
    /// available device names is returned (no silent fallback to the default —
    /// recording from the wrong device is worse than failing loudly).
    pub fn find_input_device(&self, name: Option<&str>) -> Result<cpal::Device> {
        let name = match name {
            None => return self.get_default_input_device(),
            Some(name) => name,
        };

        let devices: Vec<cpal::Device> = self
            .host
            .input_devices()
            .map_err(|e| {
                BatcherbirdError::Audio(format!("Failed to enumerate input devices: {}", e))
            })?
            .collect();
        let available: Vec<String> = devices
            .iter()
            .filter_map(|d| d.name().ok())
            .collect();

        match match_device_name(&available, Some(name)) {
            DeviceMatch::Default => self.get_default_input_device(),
            DeviceMatch::Index(idx) => {
                // The available-names vector is built in the same order as `devices`,
                // skipping devices whose name could not be read. Re-resolve the
                // device by matching its name so indices line up regardless.
                let target = &available[idx];
                devices
                    .into_iter()
                    .find(|d| d.name().ok().as_deref() == Some(target.as_str()))
                    .ok_or_else(|| {
                        BatcherbirdError::Audio(format!(
                            "Input device '{}' disappeared during lookup",
                            target
                        ))
                    })
            }
            DeviceMatch::NotFound { available } => Err(BatcherbirdError::Audio(
                not_found_message("input", name, &available),
            )),
        }
    }

    /// Find an output device by name. See [`find_input_device`](Self::find_input_device).
    pub fn find_output_device(&self, name: Option<&str>) -> Result<cpal::Device> {
        let name = match name {
            None => return self.get_default_output_device(),
            Some(name) => name,
        };

        let devices: Vec<cpal::Device> = self
            .host
            .output_devices()
            .map_err(|e| {
                BatcherbirdError::Audio(format!("Failed to enumerate output devices: {}", e))
            })?
            .collect();
        let available: Vec<String> = devices
            .iter()
            .filter_map(|d| d.name().ok())
            .collect();

        match match_device_name(&available, Some(name)) {
            DeviceMatch::Default => self.get_default_output_device(),
            DeviceMatch::Index(idx) => {
                let target = &available[idx];
                devices
                    .into_iter()
                    .find(|d| d.name().ok().as_deref() == Some(target.as_str()))
                    .ok_or_else(|| {
                        BatcherbirdError::Audio(format!(
                            "Output device '{}' disappeared during lookup",
                            target
                        ))
                    })
            }
            DeviceMatch::NotFound { available } => Err(BatcherbirdError::Audio(
                not_found_message("output", name, &available),
            )),
        }
    }

    pub fn record_test_audio(&self, duration_secs: u64) -> Result<Vec<f32>> {
        let device = self.get_default_input_device()?;
        let config = device
            .default_input_config()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to get input config: {}", e)))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let expected_samples = (duration_secs * sample_rate as u64 * channels as u64) as usize;

        let recorded_samples = Arc::new(Mutex::new(Vec::with_capacity(expected_samples)));
        let samples_clone = recorded_samples.clone();
        let recording_complete = Arc::new(Mutex::new(false));
        let complete_clone = recording_complete.clone();

        let stream = match config.sample_format() {
            SampleFormat::F32 => {
                let stream_config = Self::get_standard_stream_config();

                device
                    .build_input_stream(
                        &stream_config,
                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            let mut samples = samples_clone.lock().unwrap();
                            let complete = complete_clone.lock().unwrap();

                            if !*complete {
                                samples.extend_from_slice(data);
                            }
                        },
                        |err| tracing::error!("Audio input error: {}", err),
                        None,
                    )
                    .map_err(|e| {
                        BatcherbirdError::Audio(format!("Failed to build input stream: {}", e))
                    })?
            }
            SampleFormat::I16 => {
                let stream_config = Self::get_standard_stream_config();

                device
                    .build_input_stream(
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
                        |err| tracing::error!("Audio input error: {}", err),
                        None,
                    )
                    .map_err(|e| {
                        BatcherbirdError::Audio(format!("Failed to build input stream: {}", e))
                    })?
            }
            SampleFormat::U16 => {
                let stream_config = Self::get_standard_stream_config();

                device
                    .build_input_stream(
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
                        |err| tracing::error!("Audio input error: {}", err),
                        None,
                    )
                    .map_err(|e| {
                        BatcherbirdError::Audio(format!("Failed to build input stream: {}", e))
                    })?
            }
            _ => {
                return Err(BatcherbirdError::Audio(format!(
                    "Unsupported sample format: {:?}",
                    config.sample_format()
                )));
            }
        };

        // Start recording
        stream
            .play()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to start stream: {}", e)))?;

        // Record for specified duration
        std::thread::sleep(Duration::from_secs(duration_secs));

        // Stop recording
        {
            let mut complete = recording_complete.lock().unwrap();
            *complete = true;
        }

        stream
            .pause()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to stop stream: {}", e)))?;

        let samples = recorded_samples.lock().unwrap().clone();

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
        let rms_db = if rms > 0.0 {
            20.0 * rms.log10()
        } else {
            -100.0
        };
        let peak_db = if peak > 0.0 {
            20.0 * peak.log10()
        } else {
            -100.0
        };

        (rms, rms_db, peak_db)
    }
}

/// Result of matching a requested device name against the available names.
///
/// Kept free of cpal types so the match/fallback/error logic can be unit-tested
/// without real audio hardware.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DeviceMatch {
    /// No name was requested — caller should use the system default device.
    Default,
    /// Matched the device at this index into the supplied names slice.
    Index(usize),
    /// No device matched; carries the available names for an actionable error.
    NotFound { available: Vec<String> },
}

/// Pure name-matching logic shared by input and output device lookup.
///
/// `None` -> [`DeviceMatch::Default`]. `Some(name)` prefers an exact match, then
/// falls back to a case-insensitive match, otherwise [`DeviceMatch::NotFound`].
pub(crate) fn match_device_name(available: &[String], requested: Option<&str>) -> DeviceMatch {
    let requested = match requested {
        None => return DeviceMatch::Default,
        Some(r) => r,
    };

    if let Some(idx) = available.iter().position(|n| n == requested) {
        return DeviceMatch::Index(idx);
    }

    if let Some(idx) = available
        .iter()
        .position(|n| n.eq_ignore_ascii_case(requested))
    {
        return DeviceMatch::Index(idx);
    }

    DeviceMatch::NotFound {
        available: available.to_vec(),
    }
}

/// Build an actionable "device not found" error message listing available names.
pub(crate) fn not_found_message(kind: &str, requested: &str, available: &[String]) -> String {
    if available.is_empty() {
        format!(
            "{} device '{}' not found. No {} devices are available.",
            kind, requested, kind
        )
    } else {
        format!(
            "{} device '{}' not found. Available {} devices: {}",
            kind,
            requested,
            kind,
            available
                .iter()
                .map(|n| format!("'{}'", n))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn none_requested_yields_default() {
        let available = names(&["Built-in Microphone", "Scarlett 2i2"]);
        assert_eq!(match_device_name(&available, None), DeviceMatch::Default);
    }

    #[test]
    fn exact_match_returns_index() {
        let available = names(&["Built-in Microphone", "Scarlett 2i2"]);
        assert_eq!(
            match_device_name(&available, Some("Scarlett 2i2")),
            DeviceMatch::Index(1)
        );
    }

    #[test]
    fn exact_match_preferred_over_case_insensitive() {
        // Both an exact and a case-variant entry exist; the exact one must win.
        let available = names(&["scarlett 2i2", "Scarlett 2i2"]);
        assert_eq!(
            match_device_name(&available, Some("Scarlett 2i2")),
            DeviceMatch::Index(1)
        );
    }

    #[test]
    fn case_insensitive_match_returns_index() {
        let available = names(&["Built-in Microphone", "Scarlett 2i2"]);
        assert_eq!(
            match_device_name(&available, Some("scarlett 2I2")),
            DeviceMatch::Index(1)
        );
    }

    #[test]
    fn not_found_carries_available_names() {
        let available = names(&["Built-in Microphone", "Scarlett 2i2"]);
        match match_device_name(&available, Some("Nonexistent Device")) {
            DeviceMatch::NotFound { available: a } => assert_eq!(a, available),
            other => panic!("expected NotFound, got {:?}", other),
        }
    }

    #[test]
    fn not_found_message_lists_available_names() {
        let available = names(&["Built-in Microphone", "Scarlett 2i2"]);
        let msg = not_found_message("input", "Nonexistent Device", &available);
        assert!(msg.contains("Nonexistent Device"));
        assert!(msg.contains("Built-in Microphone"));
        assert!(msg.contains("Scarlett 2i2"));
        assert!(msg.contains("input"));
    }

    #[test]
    fn not_found_message_handles_empty_list() {
        let msg = not_found_message("output", "Foo", &[]);
        assert!(msg.contains("Foo"));
        assert!(msg.contains("No output devices"));
    }

    #[test]
    #[ignore = "requires real audio hardware / cpal host"]
    fn default_input_device_resolves_on_hardware() {
        let manager = AudioManager::new().unwrap();
        manager.find_input_device(None).unwrap();
    }
}
