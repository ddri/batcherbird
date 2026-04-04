use crate::Result;
use rustfft::{num_complex::Complex, FftPlanner};
use std::collections::HashMap;

/// Configuration for advanced loop detection
#[derive(Debug, Clone)]
pub struct AdvancedLoopConfig {
    /// Minimum loop length in seconds
    pub min_loop_length: f32,

    /// Maximum loop length in seconds  
    pub max_loop_length: f32,

    /// Correlation threshold for loop candidate validation (0.0 to 1.0)
    pub correlation_threshold: f32,

    /// Spectral coherence threshold for frequency domain matching
    pub spectral_threshold: f32,

    /// Number of loop candidates to evaluate
    pub candidate_count: usize,

    /// Enable multi-scale analysis
    pub multi_scale: bool,

    /// Crossfade length in milliseconds for loop optimization
    pub crossfade_ms: f32,
}

impl Default for AdvancedLoopConfig {
    fn default() -> Self {
        Self {
            min_loop_length: 0.5,       // 500ms minimum
            max_loop_length: 16.0,      // 16 seconds maximum
            correlation_threshold: 0.7, // 70% correlation required
            spectral_threshold: 0.8,    // 80% spectral coherence
            candidate_count: 10,        // Evaluate top 10 candidates
            multi_scale: true,          // Enable multi-scale analysis
            crossfade_ms: 50.0,         // 50ms crossfades
        }
    }
}

/// Quality metrics for loop point assessment
#[derive(Debug, Clone)]
pub struct LoopQualityMetrics {
    /// Autocorrelation coefficient (0.0 to 1.0)
    pub correlation: f32,

    /// Spectral coherence score (0.0 to 1.0)
    pub spectral_coherence: f32,

    /// Phase alignment quality (0.0 to 1.0)
    pub phase_alignment: f32,

    /// Overall quality score (weighted combination)
    pub overall_score: f32,

    /// RMS error at loop boundary
    pub boundary_error: f32,
}

/// Loop candidate with quality assessment
#[derive(Debug, Clone)]
pub struct LoopCandidate {
    /// Start sample index
    pub start_sample: usize,

    /// End sample index  
    pub end_sample: usize,

    /// Loop length in samples
    pub length_samples: usize,

    /// Loop length in seconds
    pub length_seconds: f32,

    /// Quality metrics
    pub quality: LoopQualityMetrics,

    /// Detection confidence (0.0 to 1.0)
    pub confidence: f32,
}

/// Advanced loop detection result
#[derive(Debug, Clone)]
pub struct AdvancedLoopResult {
    /// Best loop candidate found
    pub best_loop: Option<LoopCandidate>,

    /// All evaluated candidates (sorted by quality)
    pub candidates: Vec<LoopCandidate>,

    /// Whether detection was successful
    pub success: bool,

    /// Processing time in milliseconds
    pub processing_time_ms: f32,

    /// Algorithm used for detection
    pub algorithm_used: String,
}

/// Cached FFT buffers: forward and inverse complex buffers keyed by block size
type FftBufferCache = HashMap<usize, (Vec<Complex<f32>>, Vec<Complex<f32>>)>;

/// FFT-based autocorrelation processor using Wiener-Khinchin theorem
pub struct FftCorrelator {
    planner: FftPlanner<f32>,
    cache: FftBufferCache,
}

impl Default for FftCorrelator {
    fn default() -> Self {
        Self::new()
    }
}

impl FftCorrelator {
    pub fn new() -> Self {
        Self {
            planner: FftPlanner::new(),
            cache: HashMap::new(),
        }
    }

    /// Compute FFT-based autocorrelation using Wiener-Khinchin theorem
    /// This provides O(n log n) performance vs O(n²) for direct correlation
    pub fn fft_autocorrelation(&mut self, signal: &[f32]) -> Result<Vec<f32>> {
        if signal.is_empty() {
            return Ok(vec![]);
        }

        let n = signal.len();
        let fft_size = n.next_power_of_two() * 2; // Zero-pad to twice length

        // Get or create FFT plans for this size
        let (mut fft_buffer, mut spectrum) =
            if let Some((fft_buf, spec)) = self.cache.remove(&fft_size) {
                (fft_buf, spec)
            } else {
                (
                    vec![Complex::new(0.0, 0.0); fft_size],
                    vec![Complex::new(0.0, 0.0); fft_size],
                )
            };

        let fft = self.planner.plan_fft_forward(fft_size);
        let ifft = self.planner.plan_fft_inverse(fft_size);

        // Zero-pad signal to FFT size
        fft_buffer.fill(Complex::new(0.0, 0.0));
        for (i, &sample) in signal.iter().enumerate() {
            fft_buffer[i] = Complex::new(sample, 0.0);
        }

        // Forward FFT
        spectrum.copy_from_slice(&fft_buffer);
        fft.process(&mut spectrum);

        // Multiply by complex conjugate (power spectrum)
        for bin in spectrum.iter_mut() {
            *bin *= bin.conj();
        }

        // Inverse FFT to get autocorrelation
        ifft.process(&mut spectrum);

        // Extract real part and normalize
        let mut autocorr: Vec<f32> = spectrum.iter().take(n).map(|c| c.re / n as f32).collect();

        // Normalize by the zero-lag value
        if autocorr[0] > 0.0 {
            let norm_factor = autocorr[0];
            for val in autocorr.iter_mut() {
                *val /= norm_factor;
            }
        }

        // Cache buffers for reuse
        self.cache.insert(fft_size, (fft_buffer, spectrum));

        Ok(autocorr)
    }
}

/// Spectral coherence analyzer for frequency domain loop quality assessment
pub struct SpectralCoherence {
    planner: FftPlanner<f32>,
    window_size: usize,
}

impl SpectralCoherence {
    pub fn new(window_size: usize) -> Self {
        Self {
            planner: FftPlanner::new(),
            window_size: window_size.next_power_of_two(),
        }
    }

    /// Calculate spectral coherence between loop start and end segments
    pub fn calculate_coherence(
        &mut self,
        start_segment: &[f32],
        end_segment: &[f32],
    ) -> Result<f32> {
        if start_segment.len() != end_segment.len() || start_segment.is_empty() {
            return Ok(0.0);
        }

        let segment_len = start_segment.len().min(self.window_size);
        let fft = self.planner.plan_fft_forward(self.window_size);

        // Calculate FFT for both segments
        let start_spectrum = self.compute_spectrum(&fft, &start_segment[..segment_len])?;
        let end_spectrum = self.compute_spectrum(&fft, &end_segment[..segment_len])?;

        // Calculate magnitude spectral correlation
        let mut correlation_sum = 0.0;
        let mut start_mag_sum = 0.0;
        let mut end_mag_sum = 0.0;

        for i in 0..self.window_size / 2 {
            let start_mag = start_spectrum[i].norm();
            let end_mag = end_spectrum[i].norm();

            correlation_sum += start_mag * end_mag;
            start_mag_sum += start_mag * start_mag;
            end_mag_sum += end_mag * end_mag;
        }

        // Normalize correlation
        let coherence = if start_mag_sum > 0.0 && end_mag_sum > 0.0 {
            correlation_sum / (start_mag_sum.sqrt() * end_mag_sum.sqrt())
        } else {
            0.0
        };

        Ok(coherence.clamp(0.0, 1.0))
    }

    fn compute_spectrum(
        &self,
        fft: &std::sync::Arc<dyn rustfft::Fft<f32>>,
        signal: &[f32],
    ) -> Result<Vec<Complex<f32>>> {
        let mut buffer = vec![Complex::new(0.0, 0.0); self.window_size];

        // Copy signal and apply Hann window
        for (i, &sample) in signal.iter().enumerate() {
            let window_val = 0.5
                * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / self.window_size as f32).cos());
            buffer[i] = Complex::new(sample * window_val, 0.0);
        }

        fft.process(&mut buffer);
        Ok(buffer)
    }
}

/// Multi-scale analysis detector for complex loop patterns
pub struct MultiscaleAnalysis {
    scales: Vec<usize>,
}

impl MultiscaleAnalysis {
    pub fn new(sample_rate: u32, min_length: f32, max_length: f32) -> Self {
        let min_samples = (min_length * sample_rate as f32) as usize;
        let max_samples = (max_length * sample_rate as f32) as usize;

        // Generate logarithmically spaced scales
        let mut scales = Vec::new();
        let mut current = min_samples;

        while current <= max_samples {
            scales.push(current);
            current = (current as f32 * 1.2) as usize; // 20% increment
        }

        Self { scales }
    }

    /// Analyze signal at multiple time scales
    pub fn analyze_at_scales(
        &self,
        correlator: &mut FftCorrelator,
        signal: &[f32],
    ) -> Result<Vec<(usize, f32)>> {
        let autocorr = correlator.fft_autocorrelation(signal)?;
        let mut scale_results = Vec::new();

        for &scale in &self.scales {
            if scale < autocorr.len() {
                let correlation = autocorr[scale];
                scale_results.push((scale, correlation));
            }
        }

        // Sort by correlation strength
        scale_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scale_results)
    }
}

/// Advanced loop detector with FFT-based algorithms
pub struct AdvancedLoopDetector {
    config: AdvancedLoopConfig,
    correlator: FftCorrelator,
    spectral_analyzer: SpectralCoherence,
}

impl AdvancedLoopDetector {
    pub fn new(config: AdvancedLoopConfig) -> Self {
        Self {
            correlator: FftCorrelator::new(),
            spectral_analyzer: SpectralCoherence::new(2048), // 2K FFT for spectral analysis
            config,
        }
    }

    /// Detect loop points using advanced FFT-based algorithms
    pub fn detect_loops(
        &mut self,
        audio_data: &[f32],
        sample_rate: u32,
    ) -> Result<AdvancedLoopResult> {
        let start_time = std::time::Instant::now();

        if audio_data.is_empty() {
            return Ok(AdvancedLoopResult {
                best_loop: None,
                candidates: vec![],
                success: false,
                processing_time_ms: 0.0,
                algorithm_used: "None".to_string(),
            });
        }

        // Multi-scale analysis if enabled
        let candidates = if self.config.multi_scale {
            self.detect_multiscale_loops(audio_data, sample_rate)?
        } else {
            self.detect_correlation_loops(audio_data, sample_rate)?
        };

        // Sort candidates by overall quality score
        let mut sorted_candidates = candidates;
        sorted_candidates.sort_by(|a, b| {
            b.quality
                .overall_score
                .partial_cmp(&a.quality.overall_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Take top candidates and validate
        sorted_candidates.truncate(self.config.candidate_count);

        let best_loop = sorted_candidates
            .first()
            .cloned()
            .filter(|c| c.quality.overall_score >= self.config.correlation_threshold);
        let success = best_loop.is_some();

        let processing_time = start_time.elapsed().as_millis() as f32;

        Ok(AdvancedLoopResult {
            best_loop,
            candidates: sorted_candidates,
            success,
            processing_time_ms: processing_time,
            algorithm_used: if self.config.multi_scale {
                "MultiScale-FFT"
            } else {
                "FFT-Autocorr"
            }
            .to_string(),
        })
    }

    fn detect_multiscale_loops(
        &mut self,
        audio_data: &[f32],
        sample_rate: u32,
    ) -> Result<Vec<LoopCandidate>> {
        let multiscale = MultiscaleAnalysis::new(
            sample_rate,
            self.config.min_loop_length,
            self.config.max_loop_length,
        );
        let scale_results = multiscale.analyze_at_scales(&mut self.correlator, audio_data)?;

        let mut candidates = Vec::new();

        for (scale, correlation) in scale_results
            .into_iter()
            .take(self.config.candidate_count * 2)
        {
            if correlation >= self.config.correlation_threshold {
                let candidate =
                    self.create_loop_candidate(audio_data, sample_rate, scale, correlation)?;
                if let Some(c) = candidate {
                    candidates.push(c);
                }
            }
        }

        Ok(candidates)
    }

    fn detect_correlation_loops(
        &mut self,
        audio_data: &[f32],
        sample_rate: u32,
    ) -> Result<Vec<LoopCandidate>> {
        let autocorr = self.correlator.fft_autocorrelation(audio_data)?;
        let mut candidates = Vec::new();

        let min_samples = (self.config.min_loop_length * sample_rate as f32) as usize;
        let max_samples = (self.config.max_loop_length * sample_rate as f32) as usize;

        // Find peaks in autocorrelation
        for i in min_samples..autocorr.len().min(max_samples) {
            if autocorr[i] >= self.config.correlation_threshold {
                // Check if this is a local maximum
                let is_peak = (i == 0 || autocorr[i] > autocorr[i - 1])
                    && (i + 1 >= autocorr.len() || autocorr[i] > autocorr[i + 1]);

                if is_peak {
                    let candidate =
                        self.create_loop_candidate(audio_data, sample_rate, i, autocorr[i])?;
                    if let Some(c) = candidate {
                        candidates.push(c);
                    }
                }
            }
        }

        Ok(candidates)
    }

    fn create_loop_candidate(
        &mut self,
        audio_data: &[f32],
        sample_rate: u32,
        loop_length: usize,
        correlation: f32,
    ) -> Result<Option<LoopCandidate>> {
        if loop_length >= audio_data.len() {
            return Ok(None);
        }

        // Use middle section of audio for loop analysis
        let start_sample = audio_data.len() / 4;
        let end_sample = start_sample + loop_length;

        if end_sample >= audio_data.len() {
            return Ok(None);
        }

        // Calculate spectral coherence
        let crossfade_samples = ((self.config.crossfade_ms / 1000.0) * sample_rate as f32) as usize;
        let analysis_length = crossfade_samples.max(512);

        let start_segment =
            &audio_data[start_sample..start_sample + analysis_length.min(loop_length)];
        let end_segment = &audio_data[end_sample - analysis_length.min(loop_length)..end_sample];

        let spectral_coherence = self
            .spectral_analyzer
            .calculate_coherence(start_segment, end_segment)?;

        // Calculate phase alignment (simplified as RMS difference)
        let phase_alignment = self.calculate_phase_alignment(start_segment, end_segment);

        // Calculate boundary error
        let boundary_error = self.calculate_boundary_error(start_segment, end_segment);

        // Calculate overall quality score (weighted combination)
        let overall_score =
            (correlation * 0.4) + (spectral_coherence * 0.3) + (phase_alignment * 0.3);

        let quality = LoopQualityMetrics {
            correlation,
            spectral_coherence,
            phase_alignment,
            overall_score,
            boundary_error,
        };

        let candidate = LoopCandidate {
            start_sample,
            end_sample,
            length_samples: loop_length,
            length_seconds: loop_length as f32 / sample_rate as f32,
            quality,
            confidence: overall_score,
        };

        Ok(Some(candidate))
    }

    fn calculate_phase_alignment(&self, start_segment: &[f32], end_segment: &[f32]) -> f32 {
        if start_segment.len() != end_segment.len() || start_segment.is_empty() {
            return 0.0;
        }

        let mut sum_diff = 0.0;
        let mut sum_mag = 0.0;

        for (s, e) in start_segment.iter().zip(end_segment.iter()) {
            sum_diff += (s - e).abs();
            sum_mag += s.abs() + e.abs();
        }

        if sum_mag > 0.0 {
            1.0 - (sum_diff / sum_mag).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    fn calculate_boundary_error(&self, start_segment: &[f32], end_segment: &[f32]) -> f32 {
        if start_segment.len() != end_segment.len() || start_segment.is_empty() {
            return 1.0;
        }

        let mut sum_squares = 0.0;
        for (s, e) in start_segment.iter().zip(end_segment.iter()) {
            sum_squares += (s - e).powi(2);
        }

        (sum_squares / start_segment.len() as f32).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fft_correlator_basic() {
        let mut correlator = FftCorrelator::new();

        // Simple sine wave with known period
        let sample_rate = 44100;
        let frequency = 440.0; // A4
        let duration = 1.0; // 1 second
        let samples = (duration * sample_rate as f32) as usize;

        let signal: Vec<f32> = (0..samples)
            .map(|i| (2.0 * std::f32::consts::PI * frequency * i as f32 / sample_rate as f32).sin())
            .collect();

        let autocorr = correlator.fft_autocorrelation(&signal).unwrap();

        assert!(!autocorr.is_empty());
        assert_eq!(autocorr.len(), signal.len());
        assert!((autocorr[0] - 1.0).abs() < 0.01); // Normalized autocorr should be 1.0 at zero lag

        // Check for periodicity peak at expected lag
        let expected_period = (sample_rate as f32 / frequency) as usize;
        assert!(autocorr[expected_period] > 0.8); // High correlation at period
    }

    #[test]
    fn test_spectral_coherence() {
        let mut analyzer = SpectralCoherence::new(1024);

        // Identical segments should have perfect coherence
        let segment1 = vec![1.0, 0.5, -0.5, -1.0, 0.0];
        let segment2 = segment1.clone();

        let coherence = analyzer.calculate_coherence(&segment1, &segment2).unwrap();
        assert!(coherence > 0.95); // Should be nearly perfect

        // Completely different segments should have low coherence
        let segment3 = vec![0.1, -0.2, 0.3, -0.4, 0.5];
        let coherence2 = analyzer.calculate_coherence(&segment1, &segment3).unwrap();
        assert!(coherence2 < coherence); // Should be lower
    }

    #[test]
    fn test_advanced_loop_detector() {
        let config = AdvancedLoopConfig {
            min_loop_length: 0.1,
            max_loop_length: 2.0,
            correlation_threshold: 0.3,
            ..Default::default()
        };

        let mut detector = AdvancedLoopDetector::new(config);

        // Create a simple repeating pattern
        let pattern = vec![1.0, 0.5, -0.5, -1.0, 0.0];
        let mut signal = Vec::new();
        for _ in 0..20 {
            signal.extend_from_slice(&pattern);
        }

        let result = detector.detect_loops(&signal, 100).unwrap();

        assert!(result.success);
        assert!(result.best_loop.is_some());
        assert!(!result.candidates.is_empty());
        assert!(result.processing_time_ms > 0.0);

        let best_loop = result.best_loop.unwrap();
        assert!(best_loop.quality.overall_score >= 0.3);
        assert!(best_loop.length_samples > 0);
    }

    #[test]
    fn test_multiscale_analysis() {
        let sample_rate = 1000;
        let multiscale = MultiscaleAnalysis::new(sample_rate, 0.05, 1.0);
        let mut correlator = FftCorrelator::new();

        // Simple repeating signal with longer pattern for better detection
        let pattern = vec![1.0, 0.5, -0.5, -1.0, 0.0]; // 5-sample pattern
        let mut signal = Vec::new();
        for _ in 0..100 {
            signal.extend_from_slice(&pattern);
        }

        let results = multiscale
            .analyze_at_scales(&mut correlator, &signal)
            .unwrap();

        assert!(!results.is_empty());

        // Results should be sorted by correlation strength
        for i in 1..results.len() {
            assert!(results[i - 1].1 >= results[i].1);
        }

        // Should find high correlation at some reasonable scale
        assert!(results.first().unwrap().1 > 0.5); // At least 50% correlation at best scale

        // Should have multiple scales analyzed
        assert!(results.len() > 1);
    }

    #[test]
    fn test_loop_quality_metrics() {
        let config = AdvancedLoopConfig::default();
        let mut detector = AdvancedLoopDetector::new(config);

        // Perfect loop (identical segments)
        let segment = vec![1.0, 0.5, 0.0, -0.5, -1.0];
        let mut perfect_signal = segment.clone();
        perfect_signal.extend_from_slice(&segment);
        perfect_signal.extend_from_slice(&segment);

        let result = detector.detect_loops(&perfect_signal, 100).unwrap();

        if let Some(best_loop) = result.best_loop {
            assert!(best_loop.quality.correlation > 0.8);
            assert!(best_loop.quality.overall_score > 0.6);
            assert!(best_loop.quality.boundary_error < 0.1);
        }
    }
}
