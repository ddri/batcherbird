use crate::{Result, BatcherbirdError};
use std::f32::consts::PI;

/// Configuration for crossfading operations
#[derive(Debug, Clone)]
pub struct CrossfadeConfig {
    /// Crossfade length in samples
    pub length_samples: usize,
    
    /// Crossfade curve type
    pub curve_type: CrossfadeCurve,
    
    /// Enable phase alignment before crossfading
    pub phase_alignment: bool,
    
    /// Enable spectral smoothing
    pub spectral_smoothing: bool,
    
    /// Quality validation threshold (0.0 to 1.0)
    pub quality_threshold: f32,
}

impl Default for CrossfadeConfig {
    fn default() -> Self {
        Self {
            length_samples: 2048, // ~46ms at 44.1kHz
            curve_type: CrossfadeCurve::EqualPower,
            phase_alignment: true,
            spectral_smoothing: false,
            quality_threshold: 0.8,
        }
    }
}

/// Crossfade curve types for different applications
#[derive(Debug, Clone, Copy)]
pub enum CrossfadeCurve {
    /// Linear crossfade (simple but may cause amplitude dips)
    Linear,
    
    /// Equal-power crossfade (maintains constant energy)
    EqualPower,
    
    /// Sinusoidal crossfade (smooth transition)
    Sinusoidal,
    
    /// Logarithmic crossfade (perceived loudness constant)
    Logarithmic,
    
    /// Custom curve with adjustable shape
    Custom(f32), // Shape parameter 0.0-2.0
}

/// Result of crossfade operation with quality metrics
#[derive(Debug, Clone)]
pub struct CrossfadeResult {
    /// Crossfaded audio data
    pub audio_data: Vec<f32>,
    
    /// Quality metrics for the crossfade
    pub quality_metrics: CrossfadeQuality,
    
    /// Whether phase alignment was applied
    pub phase_aligned: bool,
    
    /// Processing time in milliseconds
    pub processing_time_ms: f32,
}

/// Quality metrics for crossfade assessment
#[derive(Debug, Clone)]
pub struct CrossfadeQuality {
    /// RMS error at crossfade boundaries
    pub boundary_error: f32,
    
    /// Phase coherence score (0.0 to 1.0)
    pub phase_coherence: f32,
    
    /// Spectral continuity score (0.0 to 1.0)
    pub spectral_continuity: f32,
    
    /// Overall crossfade quality (0.0 to 1.0)
    pub overall_quality: f32,
    
    /// Whether crossfade meets quality threshold
    pub meets_threshold: bool,
}

/// Phase alignment detector for optimal crossfade positioning
pub struct PhaseAlignment {
    search_window: usize,
}

impl PhaseAlignment {
    pub fn new(search_window_samples: usize) -> Self {
        Self {
            search_window: search_window_samples,
        }
    }
    
    /// Find optimal phase alignment between two audio segments
    pub fn find_optimal_alignment(&self, segment1: &[f32], segment2: &[f32]) -> Result<i32> {
        if segment1.is_empty() || segment2.is_empty() {
            return Ok(0);
        }
        
        let max_search = self.search_window.min(segment1.len()).min(segment2.len());
        let mut best_correlation = -1.0;
        let mut best_offset = 0i32;
        
        // Search for best correlation within the search window
        for offset in -(max_search as i32 / 2)..=(max_search as i32 / 2) {
            let correlation = self.calculate_correlation(segment1, segment2, offset)?;
            
            if correlation > best_correlation {
                best_correlation = correlation;
                best_offset = offset;
            }
        }
        
        Ok(best_offset)
    }
    
    fn calculate_correlation(&self, segment1: &[f32], segment2: &[f32], offset: i32) -> Result<f32> {
        let len1 = segment1.len() as i32;
        let len2 = segment2.len() as i32;
        
        // Determine overlap region
        let start1 = if offset >= 0 { 0 } else { -offset };
        let start2 = if offset >= 0 { offset } else { 0 };
        let overlap_len = ((len1 - start1).min(len2 - start2)).max(0) as usize;
        
        if overlap_len == 0 {
            return Ok(0.0);
        }
        
        let mut correlation = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;
        
        for i in 0..overlap_len {
            let val1 = segment1[(start1 + i as i32) as usize];
            let val2 = segment2[(start2 + i as i32) as usize];
            
            correlation += val1 * val2;
            norm1 += val1 * val1;
            norm2 += val2 * val2;
        }
        
        let norm_product = (norm1 * norm2).sqrt();
        if norm_product > 0.0 {
            Ok(correlation / norm_product)
        } else {
            Ok(0.0)
        }
    }
}

/// Equal-power crossfading engine for seamless loop transitions
pub struct EqualPowerCrossfader {
    config: CrossfadeConfig,
    phase_aligner: PhaseAlignment,
}

impl EqualPowerCrossfader {
    pub fn new(config: CrossfadeConfig) -> Self {
        Self {
            phase_aligner: PhaseAlignment::new(config.length_samples * 2),
            config,
        }
    }
    
    /// Create seamless crossfade between loop start and end
    pub fn create_loop_crossfade(
        &mut self,
        loop_start: &[f32],
        loop_end: &[f32],
    ) -> Result<CrossfadeResult> {
        let start_time = std::time::Instant::now();
        
        if loop_start.len() < self.config.length_samples || loop_end.len() < self.config.length_samples {
            return Err(BatcherbirdError::Audio("Insufficient audio data for crossfade".to_string()));
        }

        // Extract crossfade regions
        let fade_out_region = &loop_start[loop_start.len() - self.config.length_samples..];
        let fade_in_region = &loop_end[..self.config.length_samples];
        
        // Phase alignment if enabled
        let phase_offset = if self.config.phase_alignment {
            self.phase_aligner.find_optimal_alignment(fade_out_region, fade_in_region)?
        } else {
            0
        };
        
        // Apply crossfade with selected curve
        let crossfaded = self.apply_crossfade(fade_out_region, fade_in_region, phase_offset)?;
        
        // Calculate quality metrics
        let quality = self.assess_crossfade_quality(&crossfaded, fade_out_region, fade_in_region)?;
        
        let processing_time = start_time.elapsed().as_millis() as f32;

        Ok(CrossfadeResult {
            audio_data: crossfaded,
            quality_metrics: quality,
            phase_aligned: self.config.phase_alignment && phase_offset != 0,
            processing_time_ms: processing_time,
        })
    }
    
    /// Apply crossfade with specified curve type
    fn apply_crossfade(&self, fade_out: &[f32], fade_in: &[f32], phase_offset: i32) -> Result<Vec<f32>> {
        let length = self.config.length_samples;
        let mut result = vec![0.0; length];
        
        // Generate crossfade curves
        let (out_curve, in_curve) = self.generate_crossfade_curves(length);
        
        for i in 0..length {
            let out_sample = fade_out[i];
            
            // Apply phase offset to fade-in sample
            let in_index = (i as i32 + phase_offset).clamp(0, fade_in.len() as i32 - 1) as usize;
            let in_sample = fade_in[in_index];
            
            // Apply crossfade curves
            result[i] = out_sample * out_curve[i] + in_sample * in_curve[i];
        }
        
        Ok(result)
    }
    
    /// Generate crossfade curves based on configuration
    fn generate_crossfade_curves(&self, length: usize) -> (Vec<f32>, Vec<f32>) {
        let mut out_curve = vec![0.0; length];
        let mut in_curve = vec![0.0; length];
        
        for i in 0..length {
            let progress = i as f32 / (length - 1) as f32;
            
            let (out_gain, in_gain) = match self.config.curve_type {
                CrossfadeCurve::Linear => {
                    (1.0 - progress, progress)
                }
                
                CrossfadeCurve::EqualPower => {
                    let out_gain = (1.0 - progress).sqrt();
                    let in_gain = progress.sqrt();
                    (out_gain, in_gain)
                }
                
                CrossfadeCurve::Sinusoidal => {
                    let out_gain = ((1.0 - progress) * PI / 2.0).cos();
                    let in_gain = (progress * PI / 2.0).sin();
                    (out_gain, in_gain)
                }
                
                CrossfadeCurve::Logarithmic => {
                    let out_gain = if progress < 1.0 { 
                        (1.0 - progress).powf(0.5)
                    } else { 
                        0.0 
                    };
                    let in_gain = if progress > 0.0 { 
                        progress.powf(0.5)
                    } else { 
                        0.0 
                    };
                    (out_gain, in_gain)
                }
                
                CrossfadeCurve::Custom(shape) => {
                    let shaped_progress = progress.powf(shape);
                    let out_gain = (1.0 - shaped_progress).sqrt();
                    let in_gain = shaped_progress.sqrt();
                    (out_gain, in_gain)
                }
            };
            
            out_curve[i] = out_gain;
            in_curve[i] = in_gain;
        }
        
        (out_curve, in_curve)
    }
    
    /// Assess quality of the crossfade result
    fn assess_crossfade_quality(
        &self,
        crossfaded: &[f32],
        original_out: &[f32],
        original_in: &[f32],
    ) -> Result<CrossfadeQuality> {
        // Calculate boundary error (RMS difference at boundaries)
        let boundary_error = self.calculate_boundary_error(crossfaded, original_out, original_in);
        
        // Calculate phase coherence
        let phase_coherence = self.calculate_phase_coherence(crossfaded);
        
        // Calculate spectral continuity (simplified)
        let spectral_continuity = self.calculate_spectral_continuity(crossfaded, original_out, original_in);
        
        // Overall quality (weighted combination)
        let overall_quality = (phase_coherence * 0.4) + (spectral_continuity * 0.4) + ((1.0 - boundary_error) * 0.2);
        
        let meets_threshold = overall_quality >= self.config.quality_threshold;
        
        Ok(CrossfadeQuality {
            boundary_error,
            phase_coherence,
            spectral_continuity,
            overall_quality,
            meets_threshold,
        })
    }
    
    fn calculate_boundary_error(&self, crossfaded: &[f32], original_out: &[f32], original_in: &[f32]) -> f32 {
        let boundary_samples = (self.config.length_samples / 10).max(1);
        
        // Check start boundary (should match original fade-out)
        let mut start_error = 0.0;
        for i in 0..boundary_samples {
            if original_out.len() >= self.config.length_samples + i {
                let diff = crossfaded[i] - original_out[original_out.len() - self.config.length_samples + i];
                start_error += diff * diff;
            }
        }
        start_error = (start_error / boundary_samples as f32).sqrt();
        
        // Check end boundary (should match original fade-in)
        let mut end_error = 0.0;
        for i in 0..boundary_samples {
            let crossfade_idx = self.config.length_samples - boundary_samples + i;
            let diff = crossfaded[crossfade_idx] - original_in[i];
            end_error += diff * diff;
        }
        end_error = (end_error / boundary_samples as f32).sqrt();
        
        (start_error + end_error) / 2.0
    }
    
    fn calculate_phase_coherence(&self, crossfaded: &[f32]) -> f32 {
        if crossfaded.len() < 4 {
            return 1.0;
        }
        
        // Simplified phase coherence: measure smoothness of transitions
        let mut coherence_sum = 0.0;
        let mut count = 0;
        
        for i in 1..crossfaded.len() - 1 {
            let prev_diff = crossfaded[i] - crossfaded[i - 1];
            let next_diff = crossfaded[i + 1] - crossfaded[i];
            
            // Measure smoothness as inverse of second derivative
            let second_derivative = (next_diff - prev_diff).abs();
            coherence_sum += 1.0 / (1.0 + second_derivative * 10.0);
            count += 1;
        }
        
        if count > 0 {
            coherence_sum / count as f32
        } else {
            1.0
        }
    }
    
    fn calculate_spectral_continuity(&self, crossfaded: &[f32], original_out: &[f32], original_in: &[f32]) -> f32 {
        // Simplified spectral continuity: compare energy distribution
        let crossfade_energy = crossfaded.iter().map(|&x| x * x).sum::<f32>() / crossfaded.len() as f32;
        
        let out_region_energy = original_out[original_out.len() - self.config.length_samples..]
            .iter()
            .map(|&x| x * x)
            .sum::<f32>() / self.config.length_samples as f32;
            
        let in_region_energy = original_in[..self.config.length_samples]
            .iter()
            .map(|&x| x * x)
            .sum::<f32>() / self.config.length_samples as f32;
        
        let expected_energy = (out_region_energy + in_region_energy) / 2.0;
        
        if expected_energy > 0.0 {
            1.0 - (crossfade_energy - expected_energy).abs() / expected_energy
        } else {
            1.0
        }
    }
}

/// Spectral smoothing processor for artifact reduction
pub struct SpectralSmoother {
    window_size: usize,
}

impl SpectralSmoother {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size: window_size.next_power_of_two(),
        }
    }
    
    /// Apply spectral smoothing to reduce crossfade artifacts
    pub fn smooth_crossfade(&self, audio_data: &[f32]) -> Result<Vec<f32>> {
        if audio_data.len() < self.window_size {
            return Ok(audio_data.to_vec());
        }
        
        // For now, apply simple moving average smoothing
        // In a full implementation, this would use FFT-based processing
        let mut smoothed = audio_data.to_vec();
        let kernel_size = (self.window_size / 32).max(3);
        
        for i in kernel_size..smoothed.len() - kernel_size {
            let mut sum = 0.0;
            for j in 0..kernel_size {
                sum += audio_data[i - kernel_size / 2 + j];
            }
            smoothed[i] = sum / kernel_size as f32;
        }
        
        Ok(smoothed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_phase_alignment() {
        let aligner = PhaseAlignment::new(100);
        
        // Create two identical segments with known offset
        let base_signal = vec![1.0, 0.5, -0.5, -1.0, 0.0, 0.5, 1.0, 0.0];
        let mut offset_signal = vec![0.0, 0.0]; // 2-sample delay
        offset_signal.extend_from_slice(&base_signal);
        
        let alignment = aligner.find_optimal_alignment(&base_signal, &offset_signal).unwrap();
        
        // Should detect the 2-sample offset
        assert_eq!(alignment, 2);
    }
    
    #[test]
    fn test_crossfade_curves() {
        let config = CrossfadeConfig {
            length_samples: 100,
            curve_type: CrossfadeCurve::EqualPower,
            ..Default::default()
        };
        
        let crossfader = EqualPowerCrossfader::new(config);
        let (out_curve, in_curve) = crossfader.generate_crossfade_curves(100);
        
        // Check equal-power property: out²+ in² should be constant
        for i in 0..100 {
            let power_sum = out_curve[i] * out_curve[i] + in_curve[i] * in_curve[i];
            assert!((power_sum - 1.0).abs() < 0.01); // Should be approximately 1.0
        }
        
        // Check endpoints
        assert!((out_curve[0] - 1.0).abs() < 0.01); // Start at full gain
        assert!((in_curve[0]).abs() < 0.01); // Start at zero gain
        assert!((out_curve[99]).abs() < 0.01); // End at zero gain
        assert!((in_curve[99] - 1.0).abs() < 0.01); // End at full gain
    }
    
    #[test]
    fn test_equal_power_crossfade() {
        let config = CrossfadeConfig {
            length_samples: 50,
            curve_type: CrossfadeCurve::EqualPower,
            phase_alignment: false,
            ..Default::default()
        };
        
        let mut crossfader = EqualPowerCrossfader::new(config);
        
        // Create test signals
        let loop_start = vec![1.0; 100]; // Constant signal
        let loop_end = vec![0.5; 100]; // Different constant signal
        
        let result = crossfader.create_loop_crossfade(&loop_start, &loop_end).unwrap();
        
        assert_eq!(result.audio_data.len(), 50);
        
        // Check smooth transition from 1.0 to 0.5
        assert!((result.audio_data[0] - 1.0).abs() < 0.1); // Should start near 1.0
        assert!((result.audio_data[49] - 0.5).abs() < 0.1); // Should end near 0.5
        
        // Check quality metrics
        assert!(result.quality_metrics.overall_quality > 0.5);
    }
    
    #[test]
    fn test_crossfade_quality_assessment() {
        let config = CrossfadeConfig {
            length_samples: 50, // Smaller crossfade to match test data
            ..Default::default()
        };
        let crossfader = EqualPowerCrossfader::new(config);
        
        // Perfect crossfade (no discontinuities)
        let crossfaded = (0..50).map(|i| i as f32 / 50.0).collect::<Vec<f32>>();
        let original_out = vec![0.0; 100]; // Longer to ensure sufficient data
        let original_in = vec![1.0; 100];
        
        let quality = crossfader.assess_crossfade_quality(&crossfaded, &original_out, &original_in).unwrap();
        
        assert!(quality.phase_coherence > 0.5); // Should have reasonable coherence
        assert!(quality.overall_quality > 0.0); // Should have reasonable quality
    }
    
    #[test]
    fn test_different_crossfade_curves() {
        let curves = vec![
            CrossfadeCurve::Linear,
            CrossfadeCurve::EqualPower,
            CrossfadeCurve::Sinusoidal,
            CrossfadeCurve::Logarithmic,
            CrossfadeCurve::Custom(1.5),
        ];
        
        for curve_type in curves {
            let config = CrossfadeConfig {
                length_samples: 50,
                curve_type,
                ..Default::default()
            };
            
            let mut crossfader = EqualPowerCrossfader::new(config);
            let loop_start = vec![1.0; 100];
            let loop_end = vec![0.0; 100];
            
            let result = crossfader.create_loop_crossfade(&loop_start, &loop_end).unwrap();
            
            assert_eq!(result.audio_data.len(), 50);
            assert!(result.quality_metrics.overall_quality >= 0.0);
        }
    }
    
    #[test]
    fn test_spectral_smoother() {
        let smoother = SpectralSmoother::new(64);
        
        // Create noisy signal
        let mut signal = vec![0.0; 200];
        for i in 0..200 {
            signal[i] = (i as f32 * 0.1).sin() + 0.1 * (i as f32 * 0.5).sin(); // Add some noise
        }
        
        let smoothed = smoother.smooth_crossfade(&signal).unwrap();
        
        assert_eq!(smoothed.len(), signal.len());
        
        // Smoothed signal should be less "noisy" in the middle region
        let mid_start = signal.len() / 4;
        let mid_end = 3 * signal.len() / 4;
        
        let original_variance = signal[mid_start..mid_end].windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .sum::<f32>();
            
        let smoothed_variance = smoothed[mid_start..mid_end].windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .sum::<f32>();
        
        assert!(smoothed_variance <= original_variance); // Should be smoother
    }
}