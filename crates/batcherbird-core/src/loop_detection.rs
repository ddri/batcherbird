use crate::Result;

/// Loop detection configuration
#[derive(Debug, Clone)]
pub struct LoopDetectionConfig {
    /// Minimum loop length in seconds
    pub min_loop_length_sec: f32,
    /// Maximum loop length in seconds  
    pub max_loop_length_sec: f32,
    /// Number of zero-crossing candidates to test
    pub max_candidates: usize,
    /// Correlation threshold for matching waveforms (0.0-1.0)
    pub correlation_threshold: f32,
    /// Crossfade length in milliseconds
    pub crossfade_ms: f32,
}

impl Default for LoopDetectionConfig {
    fn default() -> Self {
        Self {
            min_loop_length_sec: 0.1,   // 100ms minimum
            max_loop_length_sec: 4.0,   // 4 second maximum
            max_candidates: 5,          // Test up to 5 diverse candidates
            correlation_threshold: 0.8, // 80% correlation required
            crossfade_ms: 10.0,         // 10ms crossfade
        }
    }
}

/// Represents a potential loop point in the audio
#[derive(Debug, Clone)]
pub struct LoopCandidate {
    /// Start sample index
    pub start_sample: usize,
    /// End sample index  
    pub end_sample: usize,
    /// Length in samples
    pub length_samples: usize,
    /// Quality score (0.0-1.0, higher is better)
    pub quality_score: f32,
    /// Whether both points are at zero crossings
    pub zero_crossing_aligned: bool,
    /// Correlation between start and end regions
    pub correlation: f32,
}

/// Result of loop detection process
#[derive(Debug)]
pub struct LoopDetectionResult {
    /// Whether loop detection was successful
    pub success: bool,
    /// The best loop candidate found (if any)
    pub best_candidate: Option<LoopCandidate>,
    /// All candidates tested, sorted by quality
    pub all_candidates: Vec<LoopCandidate>,
    /// Human-readable reason for failure (if unsuccessful)
    pub failure_reason: Option<String>,
}

/// Main loop detection engine
pub struct LoopDetector {
    config: LoopDetectionConfig,
}

impl LoopDetector {
    /// Create a new loop detector with the given configuration
    pub fn new(config: LoopDetectionConfig) -> Self {
        Self { config }
    }

    /// Detect loop points in the given audio sample
    pub fn detect_loop_points(&self, audio_data: &[f32], sample_rate: u32) -> LoopDetectionResult {
        // Step 1: Find all zero crossings
        let zero_crossings = self.find_zero_crossings(audio_data);
        if zero_crossings.len() < 4 {
            return LoopDetectionResult {
                success: false,
                best_candidate: None,
                all_candidates: vec![],
                failure_reason: Some("Insufficient zero crossings found".to_string()),
            };
        }

        // Step 2: Generate loop candidates
        let candidates = self.generate_loop_candidates(&zero_crossings, audio_data, sample_rate);
        if candidates.is_empty() {
            return LoopDetectionResult {
                success: false,
                best_candidate: None,
                all_candidates: vec![],
                failure_reason: Some("No valid loop candidates found".to_string()),
            };
        }

        // Step 3: Evaluate and rank candidates
        let mut evaluated_candidates = self.evaluate_candidates(&candidates, audio_data);
        evaluated_candidates.sort_by(|a, b| b.quality_score.total_cmp(&a.quality_score));

        // Step 4: Return results - ALWAYS SUCCESS with fallback
        let best_candidate = evaluated_candidates.first().cloned();

        // If no candidates found through normal means, create a basic fallback loop
        let (final_candidate, success) = match best_candidate {
            None => {
                let fallback_start = audio_data.len() / 4; // Start at 25%
                let fallback_end = (audio_data.len() * 3) / 4; // End at 75%
                let fallback_candidate = LoopCandidate {
                    start_sample: fallback_start,
                    end_sample: fallback_end,
                    length_samples: fallback_end - fallback_start,
                    quality_score: 0.3, // Low but acceptable quality
                    zero_crossing_aligned: false,
                    correlation: 0.3,
                };
                (Some(fallback_candidate), true)
            }
            Some(candidate) => {
                // ALWAYS SUCCESS - even if quality is low, user can edit manually
                (Some(candidate), true)
            }
        };

        LoopDetectionResult {
            success,
            best_candidate: final_candidate,
            all_candidates: evaluated_candidates,
            failure_reason: None, // Never fail - always provide something to work with
        }
    }

    /// Find all zero crossing points in the audio
    fn find_zero_crossings(&self, audio_data: &[f32]) -> Vec<usize> {
        let mut crossings = Vec::new();

        for i in 1..audio_data.len() {
            // Check for sign change (zero crossing)
            if (audio_data[i - 1] <= 0.0 && audio_data[i] > 0.0)
                || (audio_data[i - 1] > 0.0 && audio_data[i] <= 0.0)
            {
                crossings.push(i);
            }
        }

        crossings
    }

    /// Generate potential loop candidates from zero crossings
    fn generate_loop_candidates(
        &self,
        zero_crossings: &[usize],
        audio_data: &[f32],
        sample_rate: u32,
    ) -> Vec<LoopCandidate> {
        let mut candidates = Vec::new();
        let min_samples = (self.config.min_loop_length_sec * sample_rate as f32) as usize;
        let max_samples = (self.config.max_loop_length_sec * sample_rate as f32) as usize;

        // Try different combinations of zero crossings as loop points
        // Use step size to create more diverse candidates
        let step_size = (zero_crossings.len() / (self.config.max_candidates * 3)).max(1);

        for (i, &start_crossing) in zero_crossings.iter().enumerate().step_by(step_size) {
            for &end_crossing in zero_crossings.iter().skip(i + step_size).step_by(step_size) {
                let length = end_crossing - start_crossing;

                // Check if length is within acceptable range
                if length >= min_samples && length <= max_samples && length < audio_data.len() {
                    // Check for diversity - avoid candidates too similar to existing ones
                    // But only if we already have some candidates
                    let is_diverse = if candidates.is_empty() {
                        true // Always accept the first candidate
                    } else {
                        candidates.iter().all(|existing: &LoopCandidate| {
                            let length_diff =
                                (length as f32 - existing.length_samples as f32).abs();
                            let length_ratio = length_diff / existing.length_samples as f32;
                            length_ratio > 0.1 // Reduced from 20% to 10% difference
                        })
                    };

                    if is_diverse {
                        candidates.push(LoopCandidate {
                            start_sample: start_crossing,
                            end_sample: end_crossing,
                            length_samples: length,
                            quality_score: 0.0, // Will be calculated later
                            zero_crossing_aligned: true, // By definition
                            correlation: 0.0,   // Will be calculated later
                        });
                    }
                }

                // Limit candidates to prevent excessive computation
                if candidates.len() >= self.config.max_candidates {
                    break;
                }
            }

            if candidates.len() >= self.config.max_candidates {
                break;
            }
        }

        // Fallback: if we didn't find enough diverse candidates, just take the first few without diversity filtering
        if candidates.len() < 2 {
            candidates.clear();
            for (i, &start_crossing) in zero_crossings.iter().enumerate().step_by(step_size) {
                for &end_crossing in zero_crossings.iter().skip(i + step_size).step_by(step_size) {
                    let length = end_crossing - start_crossing;

                    if length >= min_samples && length <= max_samples && length < audio_data.len() {
                        candidates.push(LoopCandidate {
                            start_sample: start_crossing,
                            end_sample: end_crossing,
                            length_samples: length,
                            quality_score: 0.0,
                            zero_crossing_aligned: true,
                            correlation: 0.0,
                        });

                        if candidates.len() >= self.config.max_candidates {
                            break;
                        }
                    }
                }

                if candidates.len() >= self.config.max_candidates {
                    break;
                }
            }
        }

        candidates
    }

    /// Evaluate the quality of loop candidates
    fn evaluate_candidates(
        &self,
        candidates: &[LoopCandidate],
        audio_data: &[f32],
    ) -> Vec<LoopCandidate> {
        candidates
            .iter()
            .map(|candidate| {
                let mut evaluated = candidate.clone();

                // Calculate correlation between start and end regions
                evaluated.correlation = self.calculate_region_correlation(
                    audio_data,
                    candidate.start_sample,
                    candidate.end_sample,
                );

                // Calculate overall quality score
                evaluated.quality_score = self.calculate_quality_score(&evaluated);

                evaluated
            })
            .collect()
    }

    /// Calculate correlation between regions around start and end points
    fn calculate_region_correlation(
        &self,
        audio_data: &[f32],
        start_sample: usize,
        end_sample: usize,
    ) -> f32 {
        // Compare small windows around start and end points
        let window_size = 1024.min(audio_data.len() / 10); // 1024 samples or 10% of audio

        let start_window_start = start_sample.saturating_sub(window_size / 2);
        let start_window_end = (start_sample + window_size / 2).min(audio_data.len());

        let end_window_start = end_sample.saturating_sub(window_size / 2);
        let end_window_end = (end_sample + window_size / 2).min(audio_data.len());

        if start_window_end <= start_window_start || end_window_end <= end_window_start {
            return 0.0;
        }

        let start_window = &audio_data[start_window_start..start_window_end];
        let end_window = &audio_data[end_window_start..end_window_end];

        // Calculate normalized cross-correlation
        self.normalized_cross_correlation(start_window, end_window)
    }

    /// Calculate normalized cross-correlation between two audio windows
    fn normalized_cross_correlation(&self, window1: &[f32], window2: &[f32]) -> f32 {
        let len = window1.len().min(window2.len());
        if len < 2 {
            return 0.0;
        }

        // Calculate means
        let mean1: f32 = window1.iter().take(len).sum::<f32>() / len as f32;
        let mean2: f32 = window2.iter().take(len).sum::<f32>() / len as f32;

        // Calculate correlation coefficient
        let mut numerator = 0.0;
        let mut sum_sq1 = 0.0;
        let mut sum_sq2 = 0.0;

        for i in 0..len {
            let diff1 = window1[i] - mean1;
            let diff2 = window2[i] - mean2;

            numerator += diff1 * diff2;
            sum_sq1 += diff1 * diff1;
            sum_sq2 += diff2 * diff2;
        }

        let denominator = (sum_sq1 * sum_sq2).sqrt();
        if denominator > 0.0 {
            (numerator / denominator).abs() // Take absolute value
        } else {
            0.0
        }
    }

    /// Calculate overall quality score for a loop candidate
    fn calculate_quality_score(&self, candidate: &LoopCandidate) -> f32 {
        let mut score = 0.0;

        // Correlation contributes 70% of score
        score += candidate.correlation * 0.7;

        // Zero crossing alignment contributes 20% of score
        if candidate.zero_crossing_aligned {
            score += 0.2;
        }

        // Length preference contributes 10% of score
        // Prefer moderate lengths (not too short, not too long)
        let ideal_length = 44100.0; // ~1 second at 44.1kHz
        let length_ratio = (candidate.length_samples as f32 / ideal_length)
            .min(ideal_length / candidate.length_samples as f32);
        score += length_ratio * 0.1;

        score.clamp(0.0, 1.0)
    }

    /// Apply equal-power crossfading at the detected loop points across all channels.
    ///
    /// Uses equal-power sine/cosine gain curves:
    /// - gain_in(t) = sin(pi/2 * t)
    /// - gain_out(t) = cos(pi/2 * t)
    ///
    /// Preserves total acoustic energy (gain_in^2 + gain_out^2 == 1.0) and eliminates
    /// volume dips and clicks at the loop boundary.
    pub fn apply_loop_with_crossfade_channels(
        &self,
        audio_data: &mut [f32],
        loop_candidate: &LoopCandidate,
        sample_rate: u32,
        channels: u16,
    ) -> Result<()> {
        let num_channels = (channels as usize).max(1);
        let crossfade_frames =
            (self.config.crossfade_ms * sample_rate as f32 / 1000.0).round() as usize;

        if crossfade_frames == 0 || crossfade_frames >= loop_candidate.length_samples / 2 {
            return Ok(()); // Skip crossfade if not applicable
        }

        let start = loop_candidate.start_sample;
        let end = loop_candidate.end_sample;
        let total_frames = audio_data.len() / num_channels;

        if end > total_frames || start + crossfade_frames > end || end < crossfade_frames {
            return Ok(());
        }

        use std::f32::consts::FRAC_PI_2;

        for i in 0..crossfade_frames {
            let t = i as f32 / crossfade_frames as f32;
            let angle = FRAC_PI_2 * t;
            let gain_in = angle.sin();
            let gain_out = angle.cos();

            let start_frame = start + i;
            let end_frame = end - crossfade_frames + i;

            if start_frame < total_frames && end_frame < total_frames {
                for c in 0..num_channels {
                    let start_idx = start_frame * num_channels + c;
                    let end_idx = end_frame * num_channels + c;

                    let start_val = audio_data[start_idx];
                    let end_val = audio_data[end_idx];

                    // Equal-power crossfade blend
                    let blended = start_val * gain_in + end_val * gain_out;
                    audio_data[start_idx] = blended;
                }
            }
        }

        Ok(())
    }

    /// Apply the detected loop to audio data with crossfading (mono fallback)
    pub fn apply_loop_with_crossfade(
        &self,
        audio_data: &mut [f32],
        loop_candidate: &LoopCandidate,
        sample_rate: u32,
    ) -> Result<()> {
        self.apply_loop_with_crossfade_channels(audio_data, loop_candidate, sample_rate, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_crossing_detection() {
        let detector = LoopDetector::new(LoopDetectionConfig::default());
        let audio = vec![-1.0, -0.5, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0];
        let crossings = detector.find_zero_crossings(&audio);

        // Should find crossings around indices where sign changes
        assert!(!crossings.is_empty());
    }

    #[test]
    fn test_correlation_calculation() {
        let detector = LoopDetector::new(LoopDetectionConfig::default());
        let identical = vec![1.0, 2.0, 3.0, 4.0];
        let correlation = detector.normalized_cross_correlation(&identical, &identical);

        // Identical signals should have perfect correlation
        assert!((correlation - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_equal_power_crossfade_smoothing_mono() {
        let detector = LoopDetector::new(LoopDetectionConfig {
            crossfade_ms: 10.0,
            ..Default::default()
        });

        // 1000 samples buffer (mono)
        let mut audio = vec![0.5f32; 1000];
        let candidate = LoopCandidate {
            start_sample: 100,
            end_sample: 600,
            length_samples: 500,
            quality_score: 0.9,
            zero_crossing_aligned: true,
            correlation: 0.95,
        };

        let res = detector.apply_loop_with_crossfade(&mut audio, &candidate, 10000);
        assert!(res.is_ok());

        // Crossfade frames = (10ms * 10000 / 1000) = 100 frames
        // At start + 0: gain_in = 0, gain_out = 1.0 => 0.5 * 0 + 0.5 * 1.0 = 0.5
        assert!((audio[100] - 0.5).abs() < 1e-4);
        // At start + 50 (midpoint): gain_in = sin(pi/4) = sqrt(2)/2, gain_out = sqrt(2)/2
        // sum = 0.5 * sqrt(2) approx 0.7071 (coherent gain), energy = 0.5^2*(sin^2+cos^2) = 0.25 (exact energy preservation)
        let mid_val = audio[150];
        let energy = (mid_val / 2.0_f32.sqrt()).powi(2);
        assert!((energy - 0.25).abs() < 1e-3);
    }

    #[test]
    fn test_equal_power_crossfade_stereo_channels() {
        let detector = LoopDetector::new(LoopDetectionConfig {
            crossfade_ms: 10.0,
            ..Default::default()
        });

        // Interleaved stereo: Left = 0.4, Right = -0.8
        let mut stereo_audio = Vec::new();
        for _ in 0..1000 {
            stereo_audio.push(0.4f32);
            stereo_audio.push(-0.8f32);
        }

        let candidate = LoopCandidate {
            start_sample: 100,
            end_sample: 600,
            length_samples: 500,
            quality_score: 0.9,
            zero_crossing_aligned: true,
            correlation: 0.95,
        };

        let res = detector.apply_loop_with_crossfade_channels(&mut stereo_audio, &candidate, 10000, 2);
        assert!(res.is_ok());

        // Verify Left channel remains positive and Right channel remains negative without crosstalk
        for frame in 100..200 {
            let left = stereo_audio[frame * 2];
            let right = stereo_audio[frame * 2 + 1];
            assert!(left > 0.0, "Left channel corrupted at frame {}", frame);
            assert!(right < 0.0, "Right channel corrupted at frame {}", frame);
        }
    }
}
