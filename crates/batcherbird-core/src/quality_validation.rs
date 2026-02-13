use crate::{Result, SampleData};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Quality validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityValidationConfig {
    /// Enable audio quality validation
    pub audio_quality_check: bool,
    
    /// Enable metadata validation
    pub metadata_validation: bool,
    
    /// Enable format compatibility testing
    pub format_compatibility: bool,
    
    /// Enable automated testing with DAWs
    pub daw_compatibility_test: bool,
    
    /// Quality thresholds
    pub thresholds: QualityThresholds,
    
    /// Validation timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for QualityValidationConfig {
    fn default() -> Self {
        Self {
            audio_quality_check: true,
            metadata_validation: true,
            format_compatibility: true,
            daw_compatibility_test: false, // Requires external DAW installations
            thresholds: QualityThresholds::default(),
            timeout_seconds: 300, // 5 minutes
        }
    }
}

/// Quality validation thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    /// Minimum SNR in dB
    pub min_snr_db: f32,
    
    /// Maximum distortion (THD+N) percentage
    pub max_distortion_percent: f32,
    
    /// Minimum dynamic range in dB
    pub min_dynamic_range_db: f32,
    
    /// Maximum level variation between channels in dB
    pub max_channel_imbalance_db: f32,
    
    /// Minimum correlation for loop quality
    pub min_loop_correlation: f32,
    
    /// Maximum click/pop detection threshold
    pub max_click_threshold: f32,
    
    /// Minimum overall quality score (0.0 to 1.0)
    pub min_overall_quality: f32,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            min_snr_db: 60.0,              // Professional quality
            max_distortion_percent: 0.1,    // Very low distortion
            min_dynamic_range_db: 80.0,     // High dynamic range
            max_channel_imbalance_db: 0.5,  // Tight stereo balance
            min_loop_correlation: 0.95,     // Very high loop quality
            max_click_threshold: 0.01,      // Minimal clicks/pops
            min_overall_quality: 0.85,      // High overall quality
        }
    }
}

/// Comprehensive quality validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Sample or instrument identifier
    pub sample_id: String,
    
    /// Overall validation status
    pub status: ValidationStatus,
    
    /// Audio quality metrics
    pub audio_quality: AudioQualityMetrics,
    
    /// Metadata validation results
    pub metadata_validation: MetadataValidationResult,
    
    /// Format compatibility results
    pub format_compatibility: FormatCompatibilityResult,
    
    /// DAW compatibility results (if tested)
    pub daw_compatibility: Option<DawCompatibilityResult>,
    
    /// Overall quality score (0.0 to 1.0)
    pub overall_score: f32,
    
    /// Validation recommendations
    pub recommendations: Vec<QualityRecommendation>,
    
    /// Processing time
    pub processing_time: Duration,
}

/// Validation status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationStatus {
    /// Passes all quality checks
    Passed,
    
    /// Passes with minor warnings
    PassedWithWarnings,
    
    /// Failed quality validation
    Failed,
    
    /// Validation could not be completed
    Error,
}

/// Comprehensive audio quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioQualityMetrics {
    /// Signal-to-noise ratio in dB
    pub snr_db: f32,
    
    /// Total harmonic distortion + noise percentage
    pub thd_n_percent: f32,
    
    /// Dynamic range in dB
    pub dynamic_range_db: f32,
    
    /// Stereo channel balance in dB
    pub channel_imbalance_db: f32,
    
    /// Peak level in dBFS
    pub peak_level_dbfs: f32,
    
    /// RMS level in dBFS
    pub rms_level_dbfs: f32,
    
    /// Loudness in LUFS
    pub loudness_lufs: f32,
    
    /// Click/pop detection score
    pub click_detection_score: f32,
    
    /// Frequency response analysis
    pub frequency_response: FrequencyResponseMetrics,
    
    /// Loop quality metrics (if applicable)
    pub loop_quality: Option<LoopQualityMetrics>,
}

/// Frequency response analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyResponseMetrics {
    /// Low frequency rolloff (-3dB point) in Hz
    pub low_freq_rolloff_hz: f32,
    
    /// High frequency rolloff (-3dB point) in Hz
    pub high_freq_rolloff_hz: f32,
    
    /// Maximum frequency response deviation in dB
    pub max_deviation_db: f32,
    
    /// Spectral centroid in Hz
    pub spectral_centroid_hz: f32,
    
    /// Spectral flatness (0.0 to 1.0)
    pub spectral_flatness: f32,
}

/// Loop quality assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopQualityMetrics {
    /// Autocorrelation at loop point
    pub correlation: f32,
    
    /// Spectral continuity across loop boundary
    pub spectral_continuity: f32,
    
    /// Phase alignment quality
    pub phase_alignment: f32,
    
    /// Boundary smoothness
    pub boundary_smoothness: f32,
    
    /// Overall loop quality score
    pub overall_loop_score: f32,
}

/// Metadata validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataValidationResult {
    /// Whether required metadata is present
    pub required_metadata_present: bool,
    
    /// SMPL chunk validation
    pub smpl_chunk_valid: bool,
    
    /// Broadcast WAV metadata validation
    pub broadcast_wav_valid: bool,
    
    /// Custom metadata validation
    pub custom_metadata_valid: bool,
    
    /// Metadata consistency score
    pub consistency_score: f32,
    
    /// Missing metadata fields
    pub missing_fields: Vec<String>,
    
    /// Invalid metadata fields
    pub invalid_fields: Vec<String>,
}

/// Format compatibility test result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct FormatCompatibilityResult {
    /// File format validation
    pub file_format_valid: bool,
    
    /// Sample rate compatibility
    pub sample_rate_compatible: bool,
    
    /// Bit depth compatibility
    pub bit_depth_compatible: bool,
    
    /// Channel count compatibility
    pub channel_count_compatible: bool,
    
    /// Metadata format compatibility
    pub metadata_format_compatible: bool,
    
    /// Target format compatibility scores
    pub format_scores: HashMap<String, f32>,
}

/// DAW compatibility test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DawCompatibilityResult {
    /// Tested DAWs and their compatibility scores
    pub daw_scores: HashMap<String, f32>,
    
    /// Overall DAW compatibility score
    pub overall_daw_score: f32,
    
    /// Import success rate
    pub import_success_rate: f32,
    
    /// Playback quality score
    pub playback_quality_score: f32,
    
    /// Metadata preservation score
    pub metadata_preservation_score: f32,
}

/// Quality validation recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRecommendation {
    /// Recommendation category
    pub category: RecommendationCategory,
    
    /// Recommendation severity
    pub severity: RecommendationSeverity,
    
    /// Human-readable description
    pub description: String,
    
    /// Suggested action
    pub suggested_action: String,
    
    /// Impact if not addressed
    pub impact: String,
}

/// Recommendation categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    AudioQuality,
    Metadata,
    FormatCompatibility,
    Performance,
    Workflow,
}

/// Recommendation severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Professional quality validator
pub struct ProfessionalQualityValidator {
    config: QualityValidationConfig,
    fft_analyzer: Option<FftAnalyzer>,
}

impl ProfessionalQualityValidator {
    pub fn new(config: QualityValidationConfig) -> Self {
        Self {
            fft_analyzer: if config.audio_quality_check {
                Some(FftAnalyzer::new(2048)) // 2K FFT for analysis
            } else {
                None
            },
            config,
        }
    }
    
    /// Validate a single sample
    pub fn validate_sample(&mut self, sample: &SampleData) -> Result<ValidationResult> {
        let start_time = Instant::now();

        let mut result = ValidationResult {
            sample_id: sample.id.clone(),
            status: ValidationStatus::Passed,
            audio_quality: AudioQualityMetrics::default(),
            metadata_validation: MetadataValidationResult::default(),
            format_compatibility: FormatCompatibilityResult::default(),
            daw_compatibility: None,
            overall_score: 0.0,
            recommendations: Vec::new(),
            processing_time: Duration::default(),
        };
        
        // Audio quality validation
        if self.config.audio_quality_check {
            result.audio_quality = self.analyze_audio_quality(&sample.audio_data, sample.sample_rate)?;
        }
        
        // Metadata validation
        if self.config.metadata_validation {
            result.metadata_validation = self.validate_metadata(&sample.metadata)?;
        }
        
        // Format compatibility validation
        if self.config.format_compatibility {
            result.format_compatibility = self.validate_format_compatibility(sample)?;
        }
        
        // Calculate overall score and status
        result.overall_score = self.calculate_overall_score(&result);
        result.status = self.determine_validation_status(&result);
        
        // Generate recommendations
        result.recommendations = self.generate_recommendations(&result);
        
        result.processing_time = start_time.elapsed();

        Ok(result)
    }
    
    /// Validate multiple samples in batch
    pub fn validate_samples(&mut self, samples: &[SampleData]) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();
        
        for sample in samples {
            match self.validate_sample(sample) {
                Ok(result) => results.push(result),
                Err(error) => {
                    // Create error result
                    let error_result = ValidationResult {
                        sample_id: sample.id.clone(),
                        status: ValidationStatus::Error,
                        audio_quality: AudioQualityMetrics::default(),
                        metadata_validation: MetadataValidationResult::default(),
                        format_compatibility: FormatCompatibilityResult::default(),
                        daw_compatibility: None,
                        overall_score: 0.0,
                        recommendations: vec![QualityRecommendation {
                            category: RecommendationCategory::AudioQuality,
                            severity: RecommendationSeverity::Critical,
                            description: format!("Validation error: {}", error),
                            suggested_action: "Review sample data and configuration".to_string(),
                            impact: "Sample cannot be validated for quality".to_string(),
                        }],
                        processing_time: Duration::from_millis(0),
                    };
                    results.push(error_result);
                }
            }
        }
        
        Ok(results)
    }
    
    fn analyze_audio_quality(&mut self, audio_data: &[f32], sample_rate: u32) -> Result<AudioQualityMetrics> {
        if audio_data.is_empty() {
            return Ok(AudioQualityMetrics::default());
        }
        
        // Basic statistics
        let peak_level = audio_data.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
        let rms_level = (audio_data.iter().map(|&x| x * x).sum::<f32>() / audio_data.len() as f32).sqrt();
        
        // Convert to dBFS
        let peak_level_dbfs = if peak_level > 0.0 { 20.0 * peak_level.log10() } else { -100.0 };
        let rms_level_dbfs = if rms_level > 0.0 { 20.0 * rms_level.log10() } else { -100.0 };
        
        // Estimate noise floor (bottom 10% of magnitude values)
        let mut magnitudes: Vec<f32> = audio_data.iter().map(|&x| x.abs()).collect();
        magnitudes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let noise_floor = magnitudes[magnitudes.len() / 10];
        
        // Calculate SNR
        let snr_db = if noise_floor > 0.0 {
            20.0 * (rms_level / noise_floor).log10()
        } else {
            80.0 // Assume good SNR if no detectable noise
        };
        
        // Dynamic range estimation
        let dynamic_range_db = if noise_floor > 0.0 {
            20.0 * (peak_level / noise_floor).log10()
        } else {
            96.0 // 16-bit equivalent
        };
        
        // Simple THD+N estimation (would require more sophisticated analysis in production)
        let thd_n_percent = self.estimate_thd_n(audio_data, sample_rate);
        
        // Click detection
        let click_detection_score = self.detect_clicks_pops(audio_data);
        
        // Frequency response analysis
        let frequency_response = if let Some(ref mut analyzer) = self.fft_analyzer {
            analyzer.analyze_frequency_response(audio_data, sample_rate)?
        } else {
            FrequencyResponseMetrics::default()
        };
        
        // Channel imbalance (for stereo, would need channel separation)
        let channel_imbalance_db = 0.0; // Mono signal
        
        // LUFS estimation (simplified)
        let loudness_lufs = rms_level_dbfs - 23.0; // Rough approximation
        
        Ok(AudioQualityMetrics {
            snr_db,
            thd_n_percent,
            dynamic_range_db,
            channel_imbalance_db,
            peak_level_dbfs,
            rms_level_dbfs,
            loudness_lufs,
            click_detection_score,
            frequency_response,
            loop_quality: None, // Would be filled if loop points are present
        })
    }
    
    fn estimate_thd_n(&self, audio_data: &[f32], _sample_rate: u32) -> f32 {
        // Simplified THD+N estimation using high-frequency content
        let mut high_freq_energy = 0.0;
        let mut total_energy = 0.0;
        
        for window in audio_data.windows(2) {
            let diff = (window[1] - window[0]).abs();
            high_freq_energy += diff * diff;
            total_energy += window[0] * window[0];
        }
        
        if total_energy > 0.0 {
            (high_freq_energy / total_energy * 100.0).min(10.0) // Cap at 10%
        } else {
            0.0
        }
    }
    
    fn detect_clicks_pops(&self, audio_data: &[f32]) -> f32 {
        let mut click_score = 0.0;
        let threshold = 0.1; // 10% of full scale
        
        for window in audio_data.windows(3) {
            let center = window[1];
            let avg_neighbors = (window[0] + window[2]) / 2.0;
            let deviation = (center - avg_neighbors).abs();
            
            if deviation > threshold {
                click_score += deviation;
            }
        }
        
        // Normalize by audio length
        click_score / audio_data.len() as f32
    }
    
    fn validate_metadata(&self, metadata: &HashMap<String, String>) -> Result<MetadataValidationResult> {
        let mut result = MetadataValidationResult::default();
        
        // Check for required fields
        let required_fields = ["generator", "version"];
        let mut missing_fields = Vec::new();
        
        for field in &required_fields {
            if !metadata.contains_key(*field) {
                missing_fields.push(field.to_string());
            }
        }
        
        result.required_metadata_present = missing_fields.is_empty();
        result.missing_fields = missing_fields;
        
        // Calculate consistency score
        let total_fields = required_fields.len();
        let present_fields = total_fields - result.missing_fields.len();
        result.consistency_score = present_fields as f32 / total_fields as f32;
        
        // Validate specific metadata values
        if let Some(generator) = metadata.get("generator") {
            if generator != "BatcherBird" {
                result.invalid_fields.push("generator".to_string());
            }
        }
        
        result.custom_metadata_valid = result.invalid_fields.is_empty();
        
        Ok(result)
    }
    
    fn validate_format_compatibility(&self, sample: &SampleData) -> Result<FormatCompatibilityResult> {
        let mut result = FormatCompatibilityResult::default();
        
        // Validate sample rate
        result.sample_rate_compatible = matches!(sample.sample_rate, 44100 | 48000 | 88200 | 96000);
        
        // Validate audio data format (f32 is good)
        result.file_format_valid = true;
        
        // Channel count (mono is compatible)
        result.channel_count_compatible = true;
        
        // Bit depth (f32 provides excellent bit depth)
        result.bit_depth_compatible = true;
        
        // Metadata format
        result.metadata_format_compatible = !sample.metadata.is_empty();
        
        // Format-specific scores
        let mut format_scores = HashMap::new();
        format_scores.insert("WAV".to_string(), 1.0);
        format_scores.insert("DecentSampler".to_string(), if result.sample_rate_compatible { 1.0 } else { 0.7 });
        format_scores.insert("SFZ".to_string(), if result.sample_rate_compatible { 1.0 } else { 0.8 });
        format_scores.insert("Kontakt".to_string(), 0.9); // Good compatibility
        
        result.format_scores = format_scores;
        
        Ok(result)
    }
    
    fn calculate_overall_score(&self, result: &ValidationResult) -> f32 {
        let mut score = 0.0;
        let mut weight_sum = 0.0;
        
        // Audio quality (50% weight)
        if self.config.audio_quality_check {
            let audio_score = self.calculate_audio_quality_score(&result.audio_quality);
            score += audio_score * 0.5;
            weight_sum += 0.5;
        }
        
        // Metadata quality (25% weight)
        if self.config.metadata_validation {
            let metadata_score = result.metadata_validation.consistency_score;
            score += metadata_score * 0.25;
            weight_sum += 0.25;
        }
        
        // Format compatibility (25% weight)
        if self.config.format_compatibility {
            let format_score = self.calculate_format_compatibility_score(&result.format_compatibility);
            score += format_score * 0.25;
            weight_sum += 0.25;
        }
        
        if weight_sum > 0.0 {
            score / weight_sum
        } else {
            0.0
        }
    }
    
    fn calculate_audio_quality_score(&self, metrics: &AudioQualityMetrics) -> f32 {
        let mut score = 0.0;
        let mut count = 0;
        
        // SNR score
        if metrics.snr_db >= self.config.thresholds.min_snr_db {
            score += 1.0;
        } else {
            score += (metrics.snr_db / self.config.thresholds.min_snr_db).clamp(0.0, 1.0);
        }
        count += 1;
        
        // Distortion score
        if metrics.thd_n_percent <= self.config.thresholds.max_distortion_percent {
            score += 1.0;
        } else {
            score += (self.config.thresholds.max_distortion_percent / metrics.thd_n_percent).clamp(0.0, 1.0);
        }
        count += 1;
        
        // Dynamic range score
        if metrics.dynamic_range_db >= self.config.thresholds.min_dynamic_range_db {
            score += 1.0;
        } else {
            score += (metrics.dynamic_range_db / self.config.thresholds.min_dynamic_range_db).clamp(0.0, 1.0);
        }
        count += 1;
        
        // Click detection score
        if metrics.click_detection_score <= self.config.thresholds.max_click_threshold {
            score += 1.0;
        } else {
            score += (self.config.thresholds.max_click_threshold / metrics.click_detection_score).clamp(0.0, 1.0);
        }
        count += 1;
        
        score / count as f32
    }
    
    fn calculate_format_compatibility_score(&self, compatibility: &FormatCompatibilityResult) -> f32 {
        let mut score = 0.0;
        let mut count = 0;
        
        if compatibility.file_format_valid { score += 1.0; }
        count += 1;
        
        if compatibility.sample_rate_compatible { score += 1.0; }
        count += 1;
        
        if compatibility.bit_depth_compatible { score += 1.0; }
        count += 1;
        
        if compatibility.channel_count_compatible { score += 1.0; }
        count += 1;
        
        if compatibility.metadata_format_compatible { score += 1.0; }
        count += 1;
        
        score / count as f32
    }
    
    fn determine_validation_status(&self, result: &ValidationResult) -> ValidationStatus {
        if result.overall_score >= self.config.thresholds.min_overall_quality {
            ValidationStatus::Passed
        } else if result.overall_score >= self.config.thresholds.min_overall_quality * 0.8 {
            ValidationStatus::PassedWithWarnings
        } else {
            ValidationStatus::Failed
        }
    }
    
    fn generate_recommendations(&self, result: &ValidationResult) -> Vec<QualityRecommendation> {
        let mut recommendations = Vec::new();
        
        // Audio quality recommendations
        if result.audio_quality.snr_db < self.config.thresholds.min_snr_db {
            recommendations.push(QualityRecommendation {
                category: RecommendationCategory::AudioQuality,
                severity: RecommendationSeverity::High,
                description: format!("SNR ({:.1}dB) below recommended minimum ({:.1}dB)", 
                    result.audio_quality.snr_db, self.config.thresholds.min_snr_db),
                suggested_action: "Check recording environment for noise sources".to_string(),
                impact: "May affect playback quality in quiet passages".to_string(),
            });
        }
        
        if result.audio_quality.thd_n_percent > self.config.thresholds.max_distortion_percent {
            recommendations.push(QualityRecommendation {
                category: RecommendationCategory::AudioQuality,
                severity: RecommendationSeverity::Medium,
                description: format!("THD+N ({:.2}%) above recommended maximum ({:.2}%)", 
                    result.audio_quality.thd_n_percent, self.config.thresholds.max_distortion_percent),
                suggested_action: "Check gain staging and recording levels".to_string(),
                impact: "May introduce audible distortion".to_string(),
            });
        }
        
        // Metadata recommendations
        if !result.metadata_validation.required_metadata_present {
            recommendations.push(QualityRecommendation {
                category: RecommendationCategory::Metadata,
                severity: RecommendationSeverity::Medium,
                description: "Missing required metadata fields".to_string(),
                suggested_action: "Add missing metadata fields for better compatibility".to_string(),
                impact: "May reduce compatibility with some applications".to_string(),
            });
        }
        
        // Format compatibility recommendations
        if !result.format_compatibility.sample_rate_compatible {
            recommendations.push(QualityRecommendation {
                category: RecommendationCategory::FormatCompatibility,
                severity: RecommendationSeverity::Low,
                description: "Sample rate may not be optimal for all targets".to_string(),
                suggested_action: "Consider using 44.1kHz or 48kHz for maximum compatibility".to_string(),
                impact: "Some applications may need to resample".to_string(),
            });
        }
        
        recommendations
    }
}

/// FFT-based frequency analyzer
struct FftAnalyzer {
    fft_size: usize,
}

impl FftAnalyzer {
    fn new(fft_size: usize) -> Self {
        Self { fft_size }
    }
    
    fn analyze_frequency_response(&mut self, audio_data: &[f32], sample_rate: u32) -> Result<FrequencyResponseMetrics> {
        // Simplified frequency analysis - in production would use proper FFT
        let nyquist = sample_rate as f32 / 2.0;
        
        // Estimate frequency content by analyzing signal characteristics
        let mut high_freq_content = 0.0;
        let mut total_energy = 0.0;
        
        for window in audio_data.windows(2) {
            let diff = (window[1] - window[0]).abs();
            high_freq_content += diff;
            total_energy += window[0].abs();
        }
        
        let high_freq_ratio = if total_energy > 0.0 {
            high_freq_content / total_energy
        } else {
            0.0
        };
        
        // Estimate rolloff points
        let low_freq_rolloff_hz = 20.0; // Assume good low-end
        let high_freq_rolloff_hz = nyquist * (1.0 - high_freq_ratio * 0.3); // Estimate based on content
        
        Ok(FrequencyResponseMetrics {
            low_freq_rolloff_hz,
            high_freq_rolloff_hz,
            max_deviation_db: 3.0, // Assume reasonable flatness
            spectral_centroid_hz: nyquist * 0.3, // Rough estimate
            spectral_flatness: (1.0 - high_freq_ratio).clamp(0.0, 1.0),
        })
    }
}

impl Default for AudioQualityMetrics {
    fn default() -> Self {
        Self {
            snr_db: 0.0,
            thd_n_percent: 0.0,
            dynamic_range_db: 0.0,
            channel_imbalance_db: 0.0,
            peak_level_dbfs: -100.0,
            rms_level_dbfs: -100.0,
            loudness_lufs: -100.0,
            click_detection_score: 0.0,
            frequency_response: FrequencyResponseMetrics::default(),
            loop_quality: None,
        }
    }
}

impl Default for FrequencyResponseMetrics {
    fn default() -> Self {
        Self {
            low_freq_rolloff_hz: 20.0,
            high_freq_rolloff_hz: 20000.0,
            max_deviation_db: 3.0,
            spectral_centroid_hz: 1000.0,
            spectral_flatness: 0.5,
        }
    }
}

impl Default for MetadataValidationResult {
    fn default() -> Self {
        Self {
            required_metadata_present: false,
            smpl_chunk_valid: false,
            broadcast_wav_valid: false,
            custom_metadata_valid: false,
            consistency_score: 0.0,
            missing_fields: Vec::new(),
            invalid_fields: Vec::new(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quality_validation_config() {
        let config = QualityValidationConfig::default();
        
        assert!(config.audio_quality_check);
        assert!(config.metadata_validation);
        assert!(config.format_compatibility);
        assert!(!config.daw_compatibility_test);
        assert_eq!(config.timeout_seconds, 300);
    }
    
    #[test]
    fn test_quality_thresholds() {
        let thresholds = QualityThresholds::default();
        
        assert_eq!(thresholds.min_snr_db, 60.0);
        assert_eq!(thresholds.max_distortion_percent, 0.1);
        assert_eq!(thresholds.min_dynamic_range_db, 80.0);
        assert_eq!(thresholds.min_overall_quality, 0.85);
    }
    
    #[test]
    fn test_professional_quality_validator() {
        let config = QualityValidationConfig::default();
        let mut validator = ProfessionalQualityValidator::new(config);
        
        // Create test sample
        let mut sample = SampleData::new(
            "test_sample".to_string(),
            vec![0.5, -0.5, 0.3, -0.3, 0.0],
            44100,
        );
        
        sample.metadata.insert("generator".to_string(), "BatcherBird".to_string());
        sample.metadata.insert("version".to_string(), "1.0".to_string());
        
        let result = validator.validate_sample(&sample).unwrap();
        
        assert_eq!(result.sample_id, "test_sample");
        assert!(matches!(result.status, ValidationStatus::Passed | ValidationStatus::PassedWithWarnings));
        assert!(result.overall_score > 0.0);
        assert!(result.processing_time.as_nanos() > 0);
    }
    
    #[test]
    fn test_audio_quality_analysis() {
        let config = QualityValidationConfig::default();
        let mut validator = ProfessionalQualityValidator::new(config);
        
        // Create clean sine wave
        let sample_rate = 44100;
        let frequency = 440.0;
        let duration = 1.0;
        let samples = (duration * sample_rate as f32) as usize;
        
        let audio_data: Vec<f32> = (0..samples)
            .map(|i| 0.5 * (2.0 * std::f32::consts::PI * frequency * i as f32 / sample_rate as f32).sin())
            .collect();
        
        let metrics = validator.analyze_audio_quality(&audio_data, sample_rate).unwrap();
        
        assert!(metrics.snr_db > 0.0); // Should have positive SNR
        assert!(metrics.thd_n_percent < 5.0); // Reasonable distortion for test
        assert!(metrics.dynamic_range_db > 0.0); // Should have some dynamic range
        assert!(metrics.peak_level_dbfs > -20.0); // Reasonable level
    }
    
    #[test]
    fn test_metadata_validation() {
        let config = QualityValidationConfig::default();
        let validator = ProfessionalQualityValidator::new(config);
        
        let mut metadata = HashMap::new();
        metadata.insert("generator".to_string(), "BatcherBird".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());
        
        let result = validator.validate_metadata(&metadata).unwrap();
        
        assert!(result.required_metadata_present);
        assert!(result.custom_metadata_valid);
        assert_eq!(result.consistency_score, 1.0);
        assert!(result.missing_fields.is_empty());
        assert!(result.invalid_fields.is_empty());
    }
    
    #[test]
    fn test_format_compatibility() {
        let config = QualityValidationConfig::default();
        let validator = ProfessionalQualityValidator::new(config);
        
        let sample = SampleData::new(
            "test".to_string(),
            vec![1.0, -1.0, 0.5, -0.5],
            44100,
        );
        
        let result = validator.validate_format_compatibility(&sample).unwrap();
        
        assert!(result.file_format_valid);
        assert!(result.sample_rate_compatible);
        assert!(result.bit_depth_compatible);
        assert!(result.channel_count_compatible);
        assert!(!result.format_scores.is_empty());
    }
    
    #[test]
    fn test_validation_status_determination() {
        let config = QualityValidationConfig::default();
        let validator = ProfessionalQualityValidator::new(config);
        
        // Test different quality scores
        let high_quality_result = ValidationResult {
            sample_id: "test".to_string(),
            status: ValidationStatus::Passed, // Will be overridden
            audio_quality: AudioQualityMetrics::default(),
            metadata_validation: MetadataValidationResult::default(),
            format_compatibility: FormatCompatibilityResult::default(),
            daw_compatibility: None,
            overall_score: 0.9,
            recommendations: Vec::new(),
            processing_time: Duration::from_millis(100),
        };
        
        let status = validator.determine_validation_status(&high_quality_result);
        assert_eq!(status, ValidationStatus::Passed);
        
        let medium_quality_result = ValidationResult {
            overall_score: 0.7,
            ..high_quality_result.clone()
        };
        
        let status = validator.determine_validation_status(&medium_quality_result);
        assert_eq!(status, ValidationStatus::PassedWithWarnings);
        
        let low_quality_result = ValidationResult {
            overall_score: 0.5,
            ..high_quality_result
        };
        
        let status = validator.determine_validation_status(&low_quality_result);
        assert_eq!(status, ValidationStatus::Failed);
    }
}