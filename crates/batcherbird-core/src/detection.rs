use crate::{BatcherbirdError, Result};

/// Sample detection configuration for automatic trimming
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DetectionConfig {
    /// Threshold in dB below which audio is considered silence (-60dB to -10dB)
    pub threshold_db: f32,

    /// Window size for RMS analysis in milliseconds (5ms to 50ms)
    pub window_size_ms: f32,

    /// Minimum sample length in milliseconds (prevents tiny fragments)
    pub min_sample_length_ms: f32,

    /// Extra time to capture before detected start (pre-trigger)
    pub pre_trigger_ms: f32,

    /// Extra time to capture after detected end (reverb tail)
    pub post_trigger_ms: f32,

    /// Number of consecutive windows required to confirm start/end
    pub confirmation_windows: usize,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            threshold_db: -40.0,         // Conservative threshold
            window_size_ms: 10.0,        // 10ms windows (good balance)
            min_sample_length_ms: 100.0, // Minimum 100ms samples
            pre_trigger_ms: 20.0,        // 20ms pre-trigger
            post_trigger_ms: 200.0,      // 200ms for reverb tails
            confirmation_windows: 3,     // 3 consecutive windows for stability
        }
    }
}

impl DetectionConfig {
    /// Preset for percussive content (drums, plucks)
    pub fn percussive() -> Self {
        Self {
            threshold_db: -30.0,
            window_size_ms: 5.0,
            min_sample_length_ms: 50.0,
            pre_trigger_ms: 10.0,
            post_trigger_ms: 50.0,
            confirmation_windows: 2,
        }
    }

    /// Preset for pad/string content (sustained notes)
    pub fn sustained() -> Self {
        Self {
            threshold_db: -50.0,
            window_size_ms: 20.0,
            min_sample_length_ms: 500.0,
            pre_trigger_ms: 50.0,
            post_trigger_ms: 500.0,
            confirmation_windows: 4,
        }
    }

    /// Preset for vintage synthesizers (more noise-tolerant)
    pub fn vintage_synth() -> Self {
        Self {
            threshold_db: -35.0,
            window_size_ms: 15.0,
            min_sample_length_ms: 200.0,
            pre_trigger_ms: 30.0,
            post_trigger_ms: 300.0,
            confirmation_windows: 3,
        }
    }
}

/// Result of sample detection analysis
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DetectionResult {
    /// Start sample index (after applying pre-trigger)
    pub start_sample: usize,

    /// End sample index (after applying post-trigger)  
    pub end_sample: usize,

    /// Original detected start (before pre-trigger)
    pub detected_start: usize,

    /// Original detected end (before post-trigger)
    pub detected_end: usize,

    /// RMS energy values for each window (for debugging/visualization)
    pub rms_values: Vec<f32>,

    /// Whether detection was successful
    pub success: bool,

    /// Reason for failure (if any)
    pub failure_reason: Option<String>,
}

/// Professional sample detection engine using RMS window analysis
pub struct SampleDetector {
    config: DetectionConfig,
}

impl Default for SampleDetector {
    fn default() -> Self {
        Self::new(DetectionConfig::default())
    }
}

impl SampleDetector {
    pub fn new(config: DetectionConfig) -> Self {
        Self { config }
    }

    /// Analyze audio and detect sample boundaries
    pub fn detect_boundaries(
        &self,
        audio_data: &[f32],
        sample_rate: u32,
    ) -> Result<DetectionResult> {
        if audio_data.is_empty() {
            return Ok(DetectionResult {
                start_sample: 0,
                end_sample: 0,
                detected_start: 0,
                detected_end: 0,
                rms_values: vec![],
                success: false,
                failure_reason: Some("Empty audio data".to_string()),
            });
        }

        // Calculate window size in samples
        let window_size_samples =
            ((self.config.window_size_ms / 1000.0) * sample_rate as f32) as usize;
        if window_size_samples == 0 {
            return Err(BatcherbirdError::Audio("Window size too small".to_string()));
        }

        // Window stride: 50% overlap. `.max(1)` prevents a zero step when
        // window_size_samples == 1 (step_by(0) panics).
        let stride = (window_size_samples / 2).max(1);

        // Calculate RMS values for each window
        let rms_values = self.calculate_rms_windows(audio_data, window_size_samples, stride);

        // Convert threshold from dB to linear
        let threshold_linear = self.db_to_linear(self.config.threshold_db);

        // Find start and end points using RMS analysis
        let (detected_start_window, detected_end_window) =
            self.find_signal_boundaries(&rms_values, threshold_linear)?;

        // Convert window indices back to sample indices using the same stride
        // that produced the windows (windows overlap by 50%, so index * stride,
        // NOT index * window_size)
        let detected_start_sample = (detected_start_window * stride).min(audio_data.len());
        let detected_end_sample =
            (detected_end_window * stride + window_size_samples).min(audio_data.len());

        // Apply pre/post trigger adjustments
        let pre_trigger_samples =
            ((self.config.pre_trigger_ms / 1000.0) * sample_rate as f32) as usize;
        let post_trigger_samples =
            ((self.config.post_trigger_ms / 1000.0) * sample_rate as f32) as usize;

        let final_start = detected_start_sample.saturating_sub(pre_trigger_samples);
        let final_end = (detected_end_sample + post_trigger_samples).min(audio_data.len());

        // Validate minimum length
        let final_length_samples = final_end - final_start;
        let min_length_samples =
            ((self.config.min_sample_length_ms / 1000.0) * sample_rate as f32) as usize;

        if final_length_samples < min_length_samples {
            return Ok(DetectionResult {
                start_sample: 0,
                end_sample: audio_data.len(),
                detected_start: detected_start_sample,
                detected_end: detected_end_sample,
                rms_values,
                success: false,
                failure_reason: Some("Sample too short after detection".to_string()),
            });
        }

        Ok(DetectionResult {
            start_sample: final_start,
            end_sample: final_end,
            detected_start: detected_start_sample,
            detected_end: detected_end_sample,
            rms_values,
            success: true,
            failure_reason: None,
        })
    }

    /// Calculate RMS energy for each window (windows advance by `stride` samples)
    fn calculate_rms_windows(&self, audio_data: &[f32], window_size: usize, stride: usize) -> Vec<f32> {
        if window_size > audio_data.len() {
            // If window is larger than audio, return single RMS value
            let sum_squares: f32 = audio_data.iter().map(|&x| x * x).sum();
            return vec![(sum_squares / audio_data.len() as f32).sqrt()];
        }

        audio_data
            .windows(window_size)
            .step_by(stride) // 50% overlap for smoother analysis
            .map(|window| {
                let sum_squares: f32 = window.iter().map(|&x| x * x).sum();
                (sum_squares / window.len() as f32).sqrt()
            })
            .collect()
    }

    /// Find signal boundaries using RMS analysis with confirmation windows
    fn find_signal_boundaries(&self, rms_values: &[f32], threshold: f32) -> Result<(usize, usize)> {
        if rms_values.is_empty() {
            return Err(BatcherbirdError::Audio(
                "No RMS values to analyze".to_string(),
            ));
        }

        // Find start: first position where we have enough consecutive windows above threshold
        let start_window = self.find_start_boundary(rms_values, threshold)?;

        // Find end: last position where we have enough consecutive windows above threshold
        let end_window = self.find_end_boundary(rms_values, threshold, start_window)?;

        Ok((start_window, end_window))
    }

    /// Find start boundary with confirmation windows
    fn find_start_boundary(&self, rms_values: &[f32], threshold: f32) -> Result<usize> {
        for i in 0..rms_values.len() {
            // Check if we have enough consecutive windows above threshold
            let mut consecutive_count = 0;
            for &rms in rms_values[i..rms_values.len().min(i + self.config.confirmation_windows)].iter() {
                if rms > threshold {
                    consecutive_count += 1;
                } else {
                    break;
                }
            }

            if consecutive_count >= self.config.confirmation_windows {
                return Ok(i);
            }
        }

        // If no clear start found, use first window above threshold
        for (i, &rms) in rms_values.iter().enumerate() {
            if rms > threshold {
                return Ok(i);
            }
        }

        // Fallback: use beginning of audio
        Ok(0)
    }

    /// Find end boundary with confirmation windows
    fn find_end_boundary(
        &self,
        rms_values: &[f32],
        threshold: f32,
        start_window: usize,
    ) -> Result<usize> {
        // Search backwards from the end
        for i in (start_window..rms_values.len()).rev() {
            // Check if we have enough consecutive windows above threshold working backwards
            let mut consecutive_count = 0;
            for j in (i.saturating_sub(self.config.confirmation_windows.saturating_sub(1))..=i).rev() {
                if rms_values[j] > threshold {
                    consecutive_count += 1;
                } else {
                    break;
                }
            }

            if consecutive_count >= self.config.confirmation_windows {
                return Ok(i);
            }
        }

        // If no clear end found, use last window above threshold
        for i in (start_window..rms_values.len()).rev() {
            if rms_values[i] > threshold {
                return Ok(i);
            }
        }

        // Fallback: use end of audio
        Ok(rms_values.len().saturating_sub(1))
    }

    /// Convert decibels to linear amplitude
    fn db_to_linear(&self, db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }

    /// Trim audio data based on detection result
    pub fn trim_audio(&self, audio_data: &[f32], detection: &DetectionResult) -> Vec<f32> {
        if !detection.success {
            return audio_data.to_vec();
        }

        let start = detection.start_sample.min(audio_data.len());
        let end = detection.end_sample.min(audio_data.len());

        if start >= end {
            return audio_data.to_vec();
        }

        audio_data[start..end].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: u32 = 44100;

    /// Build silence + sine + silence test audio
    fn silence_sine_silence(lead: usize, signal: usize, tail: usize) -> Vec<f32> {
        let mut audio = vec![0.0f32; lead];
        for i in 0..signal {
            let t = i as f32 / SAMPLE_RATE as f32;
            audio.push(0.5 * (2.0 * std::f32::consts::PI * 440.0 * t).sin());
        }
        audio.resize(audio.len() + tail, 0.0);
        audio
    }

    #[test]
    fn test_detected_boundaries_align_with_signal() {
        // 0.5s silence + 1s sine + 0.5s silence at 44.1kHz
        let lead = 22050;
        let signal = 44100;
        let audio = silence_sine_silence(lead, signal, 22050);

        let config = DetectionConfig {
            pre_trigger_ms: 0.0,
            post_trigger_ms: 0.0,
            ..DetectionConfig::default()
        };
        let window_size_samples =
            ((config.window_size_ms / 1000.0) * SAMPLE_RATE as f32) as usize;

        let detector = SampleDetector::new(config);
        let result = detector.detect_boundaries(&audio, SAMPLE_RATE).unwrap();

        assert!(result.success, "detection failed: {:?}", result.failure_reason);

        // Detected start should be within one window of the actual signal start
        let start_error = (result.detected_start as i64 - lead as i64).unsigned_abs() as usize;
        assert!(
            start_error <= window_size_samples,
            "detected_start {} is {} samples from actual start {} (tolerance {})",
            result.detected_start,
            start_error,
            lead,
            window_size_samples
        );

        // Detected end should be within one window of the actual signal end
        let signal_end = lead + signal;
        let end_error = (result.detected_end as i64 - signal_end as i64).unsigned_abs() as usize;
        assert!(
            end_error <= window_size_samples,
            "detected_end {} is {} samples from actual end {} (tolerance {})",
            result.detected_end,
            end_error,
            signal_end,
            window_size_samples
        );
    }

    #[test]
    fn test_window_size_of_one_sample_does_not_panic() {
        // window_size_ms small enough that window_size_samples == 1
        let audio = silence_sine_silence(100, 500, 100);
        let config = DetectionConfig {
            window_size_ms: 0.03, // 0.03ms at 44.1kHz -> 1 sample
            pre_trigger_ms: 0.0,
            post_trigger_ms: 0.0,
            min_sample_length_ms: 1.0,
            ..DetectionConfig::default()
        };
        let detector = SampleDetector::new(config);
        let result = detector.detect_boundaries(&audio, SAMPLE_RATE);
        assert!(result.is_ok());
    }

    #[test]
    fn test_window_larger_than_audio_is_sane() {
        // Window larger than the whole buffer -> single RMS value, sane conversion
        let audio = silence_sine_silence(0, 200, 0);
        let config = DetectionConfig {
            window_size_ms: 50.0, // 2205 samples > 200
            pre_trigger_ms: 0.0,
            post_trigger_ms: 0.0,
            min_sample_length_ms: 1.0,
            confirmation_windows: 1,
            ..DetectionConfig::default()
        };
        let detector = SampleDetector::new(config);
        let result = detector.detect_boundaries(&audio, SAMPLE_RATE).unwrap();
        assert_eq!(result.detected_start, 0);
        assert_eq!(result.detected_end, audio.len());
    }

    #[test]
    fn test_zero_confirmation_windows_does_not_panic() {
        let audio = silence_sine_silence(1000, 2000, 1000);
        let config = DetectionConfig {
            confirmation_windows: 0,
            pre_trigger_ms: 0.0,
            post_trigger_ms: 0.0,
            min_sample_length_ms: 1.0,
            ..DetectionConfig::default()
        };
        let detector = SampleDetector::new(config);
        let result = detector.detect_boundaries(&audio, SAMPLE_RATE);
        assert!(result.is_ok());
    }
}
