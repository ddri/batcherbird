use crate::{BatcherbirdError, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Host, SampleFormat, SampleRate, StreamConfig,
};
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
        let available: Vec<String> = devices.iter().filter_map(|d| d.name().ok()).collect();

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
            DeviceMatch::NotFound { available } => Err(BatcherbirdError::Audio(not_found_message(
                "input", name, &available,
            ))),
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
        let available: Vec<String> = devices.iter().filter_map(|d| d.name().ok()).collect();

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
            DeviceMatch::NotFound { available } => Err(BatcherbirdError::Audio(not_found_message(
                "output", name, &available,
            ))),
        }
    }

    pub fn record_test_audio(&self, duration_secs: u64) -> Result<Vec<f32>> {
        let device = self.get_default_input_device()?;
        let config = device
            .default_input_config()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to get input config: {}", e)))?;

        // Preallocate the whole recording so the audio callback never locks or
        // allocates; it only pushes into this lock-free ring buffer.
        let capacity = duration_secs
            .checked_mul(config.sample_rate().0 as u64)
            .and_then(|frames| frames.checked_mul(config.channels() as u64))
            .and_then(|values| usize::try_from(values).ok())
            .filter(|&values| values > 0)
            .ok_or_else(|| BatcherbirdError::Config("Invalid test recording length".into()))?;
        let (producer, mut consumer) = rtrb::RingBuffer::<f32>::new(capacity);

        // Open the stream with the device's own configuration, so the sample
        // format and channel count match what the callback receives.
        let stream_config = config.config();
        macro_rules! open {
            ($sample:ty) => {
                build_test_capture_stream::<$sample>(&device, &stream_config, producer)
            };
        }
        let stream = match config.sample_format() {
            SampleFormat::F32 => open!(f32),
            SampleFormat::F64 => open!(f64),
            SampleFormat::I8 => open!(i8),
            SampleFormat::I16 => open!(i16),
            SampleFormat::I32 => open!(i32),
            SampleFormat::I64 => open!(i64),
            SampleFormat::U8 => open!(u8),
            SampleFormat::U16 => open!(u16),
            SampleFormat::U32 => open!(u32),
            SampleFormat::U64 => open!(u64),
            format => Err(BatcherbirdError::Audio(format!(
                "Unsupported sample format: {format:?}"
            ))),
        }?;

        stream
            .play()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to start stream: {}", e)))?;
        std::thread::sleep(Duration::from_secs(duration_secs));
        // Dropping the stream stops the callback before the buffer is drained.
        drop(stream);

        let mut samples = Vec::with_capacity(capacity);
        while let Ok(value) = consumer.pop() {
            samples.push(value);
        }
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

/// Audio-thread body of the test recording: convert to f32 and push without
/// locking or allocating. Values that do not fit are dropped.
fn push_test_capture<T>(producer: &mut rtrb::Producer<f32>, data: &[T])
where
    T: cpal::Sample,
    f32: cpal::FromSample<T>,
{
    for &value in data {
        if producer
            .push(<f32 as cpal::FromSample<T>>::from_sample_(value))
            .is_err()
        {
            break;
        }
    }
}

fn build_test_capture_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    mut producer: rtrb::Producer<f32>,
) -> Result<cpal::Stream>
where
    T: cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    device
        .build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| push_test_capture(&mut producer, data),
            |err| tracing::error!("Audio input error: {}", err),
            None,
        )
        .map_err(|e| BatcherbirdError::Audio(format!("Failed to build input stream: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_converts_integer_formats_to_full_scale_floats() {
        let (mut producer, mut consumer) = rtrb::RingBuffer::<f32>::new(16);
        push_test_capture(&mut producer, &[i16::MIN, 0, i16::MAX]);
        push_test_capture(&mut producer, &[i32::MIN, i32::MAX]);
        push_test_capture(&mut producer, &[0u16, 32768, u16::MAX]);
        let values: Vec<f32> = std::iter::from_fn(|| consumer.pop().ok()).collect();
        let expected = [-1.0, 0.0, 1.0, -1.0, 1.0, -1.0, 0.0, 1.0];
        assert_eq!(values.len(), expected.len());
        for (value, expected) in values.iter().zip(expected) {
            assert!((value - expected).abs() < 1e-3, "{values:?}");
        }
    }

    #[test]
    fn test_capture_drops_overflow_instead_of_growing() {
        // The callback runs on the audio thread, so it must never allocate.
        let (mut producer, mut consumer) = rtrb::RingBuffer::<f32>::new(4);
        push_test_capture(&mut producer, &[0.1f32; 10]);
        assert_eq!(std::iter::from_fn(|| consumer.pop().ok()).count(), 4);
    }

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
