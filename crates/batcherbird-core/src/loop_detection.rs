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
    /// Start frame index (inclusive)
    pub start_sample: usize,
    /// End frame index (exclusive); RIFF/SFZ export subtracts one.
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

        // A weak or phase-inverted match must not silently become a sustain loop.
        let best_candidate = evaluated_candidates
            .iter()
            .find(|candidate| candidate.correlation >= self.config.correlation_threshold)
            .cloned();
        let success = best_candidate.is_some();
        LoopDetectionResult {
            success,
            best_candidate,
            all_candidates: evaluated_candidates,
            failure_reason: if success {
                None
            } else {
                Some("No candidate meets the correlation threshold".into())
            },
        }
    }

    /// Detect interleaved audio in frame coordinates, validating all channels.
    pub fn detect_loop_points_channels(
        &self,
        audio_data: &[f32],
        sample_rate: u32,
        channels: u16,
    ) -> LoopDetectionResult {
        let channels = channels as usize;
        if channels == 0 || sample_rate == 0 || !audio_data.len().is_multiple_of(channels) {
            return LoopDetectionResult {
                success: false,
                best_candidate: None,
                all_candidates: vec![],
                failure_reason: Some("Invalid audio frames".into()),
            };
        }
        // Use the highest-energy channel for candidates, avoiding phase cancellation in stereo.
        let channel = (0..channels)
            .max_by(|&a, &b| {
                let energy = |channel| {
                    audio_data
                        .iter()
                        .skip(channel)
                        .step_by(channels)
                        .map(|v| v * v)
                        .sum::<f32>()
                };
                energy(a).total_cmp(&energy(b))
            })
            .unwrap_or(0);
        let reference: Vec<f32> = audio_data
            .iter()
            .skip(channel)
            .step_by(channels)
            .copied()
            .collect();
        let mut result = self.detect_loop_points(&reference, sample_rate);
        // Check every audible channel at identical frame positions.
        for candidate in &mut result.all_candidates {
            let mut correlation = candidate.correlation;
            for channel in 0..channels {
                let mono: Vec<f32> = audio_data
                    .iter()
                    .skip(channel)
                    .step_by(channels)
                    .copied()
                    .collect();
                if mono.iter().any(|value| value.abs() > 1e-6) {
                    correlation = correlation.min(self.calculate_region_correlation(
                        &mono,
                        candidate.start_sample,
                        candidate.end_sample,
                    ));
                }
            }
            candidate.correlation = correlation;
            candidate.quality_score = self.calculate_quality_score(candidate);
        }
        result
            .all_candidates
            .sort_by(|a, b| b.quality_score.total_cmp(&a.quality_score));
        result.best_candidate = result
            .all_candidates
            .iter()
            .find(|candidate| candidate.correlation >= self.config.correlation_threshold)
            .cloned();
        result.success = result.best_candidate.is_some();
        result.failure_reason = if result.success {
            None
        } else {
            Some("No matching loop across all channels".into())
        };
        result
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

    /// First frame after the attack settles.
    ///
    /// Uses a 100 ms RMS envelope (longer than one cycle of even the lowest
    /// notes, so a window never lands on a quiet part of a cycle): after its
    /// peak, the attack ends at the first
    /// window within 10% of the median level of the rest of the note. For a
    /// steady tone that is the peak itself; for a decaying sound it is partway
    /// through the decay.
    fn sustain_start(audio_data: &[f32], sample_rate: u32) -> usize {
        let window = (sample_rate as usize / 10).max(1);
        let envelope: Vec<f32> = audio_data
            .chunks(window)
            .map(|chunk| (chunk.iter().map(|v| v * v).sum::<f32>() / chunk.len() as f32).sqrt())
            .collect();
        let Some(peak) = envelope
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(index, _)| index)
        else {
            return 0;
        };
        let mut after_peak = envelope[peak..].to_vec();
        after_peak.sort_by(f32::total_cmp);
        let median = after_peak[after_peak.len() / 2];
        let settled = envelope[peak..]
            .iter()
            .position(|&level| level <= median * 1.1)
            .unwrap_or(0);
        (peak + settled) * window
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

        // Loop only the sustained part of the note: a loop that starts inside
        // the attack replays the attack transient on every pass.
        let sustain = Self::sustain_start(audio_data, sample_rate);
        let first = zero_crossings.partition_point(|&crossing| crossing < sustain);
        let zero_crossings = if zero_crossings.len() - first >= 4 {
            &zero_crossings[first..]
        } else {
            zero_crossings
        };

        // Try different combinations of zero crossings as loop points
        // Use step size to create more diverse candidates
        let step_size = (zero_crossings.len() / (self.config.max_candidates.max(1) * 3)).max(1);

        for (i, &start_crossing) in zero_crossings.iter().enumerate().step_by(step_size) {
            for &end_crossing in zero_crossings.iter().skip(i + step_size).step_by(step_size) {
                let length = end_crossing - start_crossing;

                // Check if length is within acceptable range
                if length >= min_samples
                    && length <= max_samples
                    && length < audio_data.len()
                    && (audio_data[start_crossing] > 0.0) == (audio_data[end_crossing] > 0.0)
                {
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

                    if length >= min_samples
                        && length <= max_samples
                        && length < audio_data.len()
                        && (audio_data[start_crossing] > 0.0) == (audio_data[end_crossing] > 0.0)
                    {
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

        let length = window_size.min(audio_data.len().saturating_sub(end_sample));
        if length < 2 {
            return 0.0;
        }
        let start_window = &audio_data[start_sample..start_sample + length];
        let end_window = &audio_data[end_sample..end_sample + length];

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
            (numerator / denominator).clamp(-1.0, 1.0)
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

    /// Crossfade the end of the loop into the audio before the loop start, across
    /// all channels, so the loop wraps without a click. Skipped when the loop is
    /// too short or starts too close to the beginning for the fade length.
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

        // The material just before the loop start is what the end must turn
        // into, so the fade needs that many frames of pre-roll.
        if start < crossfade_frames {
            return Ok(());
        }

        // Fade the end of the loop into the audio that precedes the loop start.
        // The last looped frame then equals the frame before `start`, so the
        // wrap end-1 -> start is as smooth as the recording at start-1 -> start.
        // Nothing before the fade region changes, so the first pass into the
        // loop plays exactly as recorded.
        //
        // Loop points are chosen for strong correlation, so a linear
        // (equal-gain) blend keeps the level steady; an equal-power blend would
        // raise matching material by up to 3 dB mid-fade.
        for i in 0..crossfade_frames {
            let into_start = (i + 1) as f32 / crossfade_frames as f32;
            let tail_frame = end - crossfade_frames + i;
            let source_frame = start - crossfade_frames + i;
            for c in 0..num_channels {
                let tail = tail_frame * num_channels + c;
                let source = source_frame * num_channels + c;
                audio_data[tail] =
                    audio_data[tail] * (1.0 - into_start) + audio_data[source] * into_start;
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
    fn rejected_loop_is_not_promoted_to_success() {
        let detector = LoopDetector::new(LoopDetectionConfig {
            correlation_threshold: 1.1,
            ..Default::default()
        });
        let audio: Vec<f32> = (0..1000)
            .map(|frame| (std::f32::consts::TAU * frame as f32 / 20.0).sin())
            .collect();
        let result = detector.detect_loop_points(&audio, 1000);
        assert!(!result.all_candidates.is_empty());
        assert!(!result.success);
        assert!(result.best_candidate.is_none());
        assert!(result.failure_reason.is_some());
    }

    #[test]
    fn inverted_waveforms_are_not_matching_loop_boundaries() {
        let detector = LoopDetector::new(Default::default());
        assert!(detector.normalized_cross_correlation(&[0.0, 0.5, 1.0], &[0.0, -0.5, -1.0]) < 0.0);
    }

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

    /// One second of loud attack (with an extra partial) that drops to a
    /// steady, quieter tone for the remaining five seconds.
    fn attack_then_sustain(sample_rate: usize) -> Vec<f32> {
        (0..sample_rate * 6)
            .map(|frame| {
                let t = frame as f32 / sample_rate as f32;
                let tone = (std::f32::consts::TAU * 10.0 * t).sin();
                if frame < sample_rate {
                    tone + 0.5 * (std::f32::consts::TAU * 37.0 * t).sin()
                } else {
                    0.3 * tone
                }
            })
            .collect()
    }

    #[test]
    fn loop_starts_after_the_attack() {
        let audio = attack_then_sustain(1000);
        let result = LoopDetector::new(Default::default()).detect_loop_points(&audio, 1000);
        let best = result.best_candidate.expect("steady tone loops");
        assert!(
            best.start_sample >= 1000,
            "loop starts at frame {} inside the attack",
            best.start_sample
        );
        for candidate in &result.all_candidates {
            assert!(candidate.start_sample >= 1000, "{candidate:?}");
        }
    }

    fn loop_at(start: usize, end: usize) -> LoopCandidate {
        LoopCandidate {
            start_sample: start,
            end_sample: end,
            length_samples: end - start,
            quality_score: 0.9,
            zero_crossing_aligned: true,
            correlation: 0.95,
        }
    }

    #[test]
    fn crossfade_does_not_change_a_steady_level() {
        // Loop boundaries are chosen for strong correlation, so the blend must
        // not raise the level of matching material.
        let detector = LoopDetector::new(LoopDetectionConfig {
            crossfade_ms: 10.0,
            ..Default::default()
        });
        let mut audio = vec![0.5f32; 1000];
        detector
            .apply_loop_with_crossfade(&mut audio, &loop_at(200, 700), 10_000)
            .unwrap();
        for (frame, value) in audio.iter().enumerate() {
            assert!((value - 0.5).abs() < 1e-6, "frame {frame} is {value}");
        }
    }

    #[test]
    fn crossfade_makes_the_loop_wrap_continuous_without_touching_the_first_pass() {
        let detector = LoopDetector::new(LoopDetectionConfig {
            crossfade_ms: 10.0,
            ..Default::default()
        });
        // A ramp makes any jump obvious: the wrap from frame 699 to 200 would
        // otherwise drop by ~0.5.
        let original: Vec<f32> = (0..1000).map(|frame| frame as f32 / 1000.0).collect();
        let mut audio = original.clone();
        let (start, end, fade) = (200, 700, 100);
        detector
            .apply_loop_with_crossfade(&mut audio, &loop_at(start, end), 10_000)
            .unwrap();

        // Everything up to the fade region plays exactly as recorded, including
        // the loop start, so the first pass into the loop has no jump.
        assert_eq!(audio[..end - fade], original[..end - fade]);
        assert_eq!(audio[end..], original[end..]);
        // The last looped frame now leads into the start the way the frame
        // before the start does.
        assert!((audio[end - 1] - original[start - 1]).abs() < 1e-6);
        // No step inside the fade is larger than a normal step plus the blend.
        let normal_step = 1.0 / 1000.0;
        for frame in end - fade..end {
            let step = (audio[frame] - audio[frame - 1]).abs();
            assert!(
                step < normal_step + 0.5 / fade as f32 + 1e-4,
                "jump at {frame}"
            );
        }
    }

    #[test]
    fn crossfade_keeps_stereo_channels_separate() {
        let detector = LoopDetector::new(LoopDetectionConfig {
            crossfade_ms: 10.0,
            ..Default::default()
        });
        let mut stereo: Vec<f32> = (0..1000).flat_map(|_| [0.4f32, -0.8]).collect();
        detector
            .apply_loop_with_crossfade_channels(&mut stereo, &loop_at(200, 700), 10_000, 2)
            .unwrap();
        for frame in stereo.chunks_exact(2) {
            assert!((frame[0] - 0.4).abs() < 1e-6 && (frame[1] + 0.8).abs() < 1e-6);
        }
    }
}
