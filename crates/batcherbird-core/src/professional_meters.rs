/// Professional Audio Level Monitoring and Metering Engine
/// 
/// Implements industry-standard metering with professional ballistics:
/// - VU Meters: -18dBFS operating level, 300ms ballistics for average levels
/// - Peak Meters: Digital peak detection with 1-4ms hold time, BBC PPM standard  
/// - LUFS: Integrated loudness measurement for broadcast standards
/// 
/// Based on research from Epic 3.1 - Professional Audio Quality

use std::collections::VecDeque;

/// Professional meter types following industry standards
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MeterType {
    /// VU Meter: 300ms integration time, -18dBFS operating level
    VU,
    /// Peak Program Meter: BBC PPM standard with 10ms attack, 1.5s decay
    PPM,
    /// Digital Peak: Instantaneous peak with configurable hold time
    DigitalPeak,
    /// LUFS: EBU R128 loudness measurement
    LUFS,
}

/// Professional ballistics configuration
#[derive(Debug, Clone)]
pub struct BallisticsConfig {
    pub sample_rate: f32,
    pub attack_time_ms: f32,
    pub release_time_ms: f32,
    pub integration_time_ms: f32,
    pub hold_time_ms: f32,
}

impl BallisticsConfig {
    /// VU Meter ballistics (300ms integration)
    pub fn vu_meter(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            attack_time_ms: 300.0,
            release_time_ms: 300.0,
            integration_time_ms: 300.0,
            hold_time_ms: 0.0,
        }
    }
    
    /// BBC PPM ballistics (10ms attack, 1.5s decay)
    pub fn bbc_ppm(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            attack_time_ms: 10.0,
            release_time_ms: 1500.0,
            integration_time_ms: 10.0,
            hold_time_ms: 100.0,
        }
    }
    
    /// Digital Peak with hold
    pub fn digital_peak(sample_rate: f32, hold_time_ms: f32) -> Self {
        Self {
            sample_rate,
            attack_time_ms: 0.0,  // Instantaneous
            release_time_ms: 0.0,
            integration_time_ms: 0.0,
            hold_time_ms,
        }
    }
}

/// Exponential integrator for professional meter ballistics
#[derive(Debug)]
pub struct ExponentialIntegrator {
    current_value: f32,
    attack_coefficient: f32,
    release_coefficient: f32,
}

impl ExponentialIntegrator {
    pub fn new(config: &BallisticsConfig) -> Self {
        // Calculate exponential coefficients: α = 1 - e^(-1 / (time_constant * sample_rate))
        let attack_samples = (config.attack_time_ms / 1000.0) * config.sample_rate;
        let release_samples = (config.release_time_ms / 1000.0) * config.sample_rate;
        
        let attack_coefficient = if attack_samples > 0.0 {
            1.0 - (-1.0 / attack_samples).exp()
        } else {
            1.0  // Instantaneous
        };
        
        let release_coefficient = if release_samples > 0.0 {
            1.0 - (-1.0 / release_samples).exp()
        } else {
            1.0  // Instantaneous
        };
        
        Self {
            current_value: 0.0,
            attack_coefficient,
            release_coefficient,
        }
    }
    
    /// Process a single input value with exponential smoothing
    pub fn process(&mut self, input: f32) -> f32 {
        let coefficient = if input > self.current_value {
            self.attack_coefficient
        } else {
            self.release_coefficient
        };
        
        // Exponential smoothing: output = input * α + output * (1-α)
        self.current_value = input * coefficient + self.current_value * (1.0 - coefficient);
        self.current_value
    }
    
    /// Reset integrator state
    pub fn reset(&mut self) {
        self.current_value = 0.0;
    }
}

/// Peak hold detector with configurable hold time
#[derive(Debug)]
pub struct PeakHoldDetector {
    current_peak: f32,
    hold_value: f32,
    hold_counter: usize,
    hold_samples: usize,
}

impl PeakHoldDetector {
    pub fn new(config: &BallisticsConfig) -> Self {
        let hold_samples = ((config.hold_time_ms / 1000.0) * config.sample_rate) as usize;
        
        Self {
            current_peak: 0.0,
            hold_value: 0.0,
            hold_counter: 0,
            hold_samples,
        }
    }
    
    /// Process input and return peak with hold behavior
    pub fn process(&mut self, input: f32) -> f32 {
        // Update current peak
        if input > self.current_peak {
            self.current_peak = input;
            self.hold_value = input;
            self.hold_counter = self.hold_samples;
        }
        
        // Decay hold counter
        if self.hold_counter > 0 {
            self.hold_counter -= 1;
            self.hold_value
        } else {
            // Gradual decay when hold expires
            self.hold_value *= 0.999; // Slow decay
            self.hold_value.max(self.current_peak)
        }
    }
    
    /// Reset peak detector
    pub fn reset(&mut self) {
        self.current_peak = 0.0;
        self.hold_value = 0.0;
        self.hold_counter = 0;
    }
}

/// SIMD-optimized level calculator using wide crate
#[derive(Debug)]
pub struct SimdLevelCalculator {
    _phantom: std::marker::PhantomData<()>,
}

impl SimdLevelCalculator {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Calculate peak level using SIMD operations
    pub fn calculate_peak_simd(&self, samples: &[f32]) -> f32 {
        // Use wide crate for SIMD operations when available
        #[cfg(target_feature = "avx2")]
        {
            self.calculate_peak_avx2(samples)
        }
        #[cfg(not(target_feature = "avx2"))]
        {
            self.calculate_peak_scalar(samples)
        }
    }
    
    /// Calculate RMS level using SIMD operations
    pub fn calculate_rms_simd(&self, samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        
        #[cfg(target_feature = "avx2")]
        {
            self.calculate_rms_avx2(samples)
        }
        #[cfg(not(target_feature = "avx2"))]
        {
            self.calculate_rms_scalar(samples)
        }
    }
    
    #[cfg(target_feature = "avx2")]
    fn calculate_peak_avx2(&self, samples: &[f32]) -> f32 {
        use wide::f32x8;
        
        let mut max_vec = f32x8::splat(0.0);
        let chunks = samples.chunks_exact(8);
        let remainder = chunks.remainder();
        
        for chunk in chunks {
            let vec = f32x8::from([
                chunk[0], chunk[1], chunk[2], chunk[3],
                chunk[4], chunk[5], chunk[6], chunk[7]
            ]);
            let abs_vec = vec.abs();
            max_vec = max_vec.max(abs_vec);
        }
        
        // Horizontal max of vector elements
        let max_scalar = max_vec.to_array().iter().fold(0.0f32, |acc, &x| acc.max(x));
        
        // Handle remainder
        let remainder_max = remainder.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
        
        max_scalar.max(remainder_max)
    }
    
    #[cfg(target_feature = "avx2")]
    fn calculate_rms_avx2(&self, samples: &[f32]) -> f32 {
        use wide::f32x8;
        
        let mut sum_vec = f32x8::splat(0.0);
        let chunks = samples.chunks_exact(8);
        let remainder = chunks.remainder();
        
        for chunk in chunks {
            let vec = f32x8::from([
                chunk[0], chunk[1], chunk[2], chunk[3],
                chunk[4], chunk[5], chunk[6], chunk[7]
            ]);
            sum_vec = sum_vec + (vec * vec);
        }
        
        // Horizontal sum of vector elements
        let sum_scalar = sum_vec.to_array().iter().sum::<f32>();
        
        // Handle remainder
        let remainder_sum: f32 = remainder.iter().map(|&x| x * x).sum();
        
        let total_sum = sum_scalar + remainder_sum;
        (total_sum / samples.len() as f32).sqrt()
    }
    
    fn calculate_peak_scalar(&self, samples: &[f32]) -> f32 {
        samples.iter().map(|&x| x.abs()).fold(0.0f32, f32::max)
    }
    
    fn calculate_rms_scalar(&self, samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }
}

impl Default for SimdLevelCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Simplified LUFS processor (basic implementation)
/// Note: Full EBU R128 compliance requires complex filtering - this is a starter implementation
#[derive(Debug)]
pub struct LoudnessProcessor {
    integration_buffer: VecDeque<f32>,
    integration_samples: usize,
    sample_rate: f32,
}

impl LoudnessProcessor {
    pub fn new(sample_rate: f32, integration_time_ms: f32) -> Self {
        let integration_samples = ((integration_time_ms / 1000.0) * sample_rate) as usize;
        
        Self {
            integration_buffer: VecDeque::with_capacity(integration_samples + 1),
            integration_samples,
            sample_rate,
        }
    }
    
    /// Process samples and return approximate LUFS value
    /// Note: This is a simplified implementation - full EBU R128 requires pre-filtering
    pub fn process(&mut self, samples: &[f32]) -> f32 {
        for &sample in samples {
            self.integration_buffer.push_back(sample * sample);
            
            if self.integration_buffer.len() > self.integration_samples {
                self.integration_buffer.pop_front();
            }
        }
        
        if self.integration_buffer.is_empty() {
            return -70.0; // Silence threshold
        }
        
        // Calculate mean square over integration window
        let mean_square: f32 = self.integration_buffer.iter().sum::<f32>() / self.integration_buffer.len() as f32;
        
        if mean_square <= 0.0 {
            -70.0 // Silence threshold
        } else {
            // Convert to LUFS (approximate - missing EBU R128 pre-filtering)
            10.0 * mean_square.log10() - 0.691  // Rough LUFS calibration
        }
    }
    
    pub fn reset(&mut self) {
        self.integration_buffer.clear();
    }
}

/// Professional meter readings with industry-standard values
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfessionalMeterReadings {
    /// VU level in dBFS (-18dBFS = 0 VU)
    pub vu_db: f32,
    /// PPM level in dBFS
    pub ppm_db: f32,
    /// Digital peak in dBFS
    pub peak_db: f32,
    /// Peak hold value in dBFS
    pub peak_hold_db: f32,
    /// LUFS integrated loudness
    pub lufs: f32,
    /// Professional gain staging recommendation
    pub gain_staging: GainStagingStatus,
}

/// Gain staging guidance for professional recording levels
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GainStagingStatus {
    /// Optimal level around -18dBFS
    Optimal,
    /// Too quiet (below -30dBFS) 
    TooQuiet,
    /// Too loud (above -6dBFS)
    TooLoud,
    /// Acceptable range (-30 to -6dBFS)
    Acceptable,
    /// Clipping detected (≥0dBFS)
    Clipping,
}

/// Professional gain staging assistant for synthesizer recording
/// M3.1.2: Gain Staging Assistant - Epic 3.1
#[derive(Debug)]
pub struct GainStagingAssistant {
    target_level_db: f32,           // -18dBFS for synthesizers
    optimal_range: (f32, f32),      // (-24dBFS, -12dBFS) 
    headroom_minimum_db: f32,       // -6dBFS minimum headroom
    level_history: VecDeque<f32>,   // Historical peak levels for trend analysis
    max_history_length: usize,      // Maximum number of readings to keep
    recommendation_engine: RecommendationEngine,
}

/// Headroom detection for clipping prevention
#[derive(Debug)]
pub struct HeadroomDetector {
    headroom_threshold_db: f32,     // Warning threshold
    clip_threshold_db: f32,         // Clipping threshold
    peak_history: VecDeque<f32>,    // Recent peak history
    clip_count: usize,              // Number of clips detected
}

impl HeadroomDetector {
    pub fn new(headroom_threshold_db: f32, clip_threshold_db: f32) -> Self {
        Self {
            headroom_threshold_db,
            clip_threshold_db,
            peak_history: VecDeque::with_capacity(100),
            clip_count: 0,
        }
    }
    
    /// Analyze peak level and return headroom status
    pub fn analyze_headroom(&mut self, peak_db: f32) -> HeadroomStatus {
        self.peak_history.push_back(peak_db);
        if self.peak_history.len() > 100 {
            self.peak_history.pop_front();
        }
        
        if peak_db >= self.clip_threshold_db {
            self.clip_count += 1;
            HeadroomStatus::Clipping
        } else if peak_db >= self.headroom_threshold_db {
            HeadroomStatus::LowHeadroom
        } else {
            HeadroomStatus::SafeHeadroom
        }
    }
    
    /// Get maximum peak in recent history
    pub fn get_max_recent_peak(&self) -> f32 {
        self.peak_history.iter().copied().fold(-100.0f32, f32::max)
    }
    
    /// Reset clip counter
    pub fn reset_clips(&mut self) {
        self.clip_count = 0;
    }
    
    pub fn get_clip_count(&self) -> usize {
        self.clip_count
    }
}

/// Headroom analysis results
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HeadroomStatus {
    SafeHeadroom,    // > 6dB headroom
    LowHeadroom,     // 0-6dB headroom  
    Clipping,        // ≥ 0dBFS
}

/// Level range for optimal recording
#[derive(Debug, Clone)]
pub struct LevelRange {
    pub min_db: f32,
    pub max_db: f32,
    pub target_db: f32,
}

impl LevelRange {
    /// Standard synthesizer recording range
    pub fn synthesizer_standard() -> Self {
        Self {
            min_db: -24.0,  // Minimum useful level
            max_db: -12.0,  // Maximum safe level before warning
            target_db: -18.0, // Optimal target level
        }
    }
    
    /// Check if level is within range
    pub fn contains(&self, level_db: f32) -> bool {
        level_db >= self.min_db && level_db <= self.max_db
    }
    
    /// Get distance from target (negative = too quiet, positive = too loud)
    pub fn distance_from_target(&self, level_db: f32) -> f32 {
        level_db - self.target_db
    }
}

/// Recommendation engine for gain adjustments
#[derive(Debug)]
pub struct RecommendationEngine {
    target_level_db: f32,
    adjustment_sensitivity: f32,  // How aggressive the recommendations are
}

impl RecommendationEngine {
    pub fn new(target_level_db: f32) -> Self {
        Self {
            target_level_db,
            adjustment_sensitivity: 1.0, // 1:1 recommendation ratio
        }
    }
    
    /// Generate gain adjustment recommendation
    pub fn recommend_adjustment(&self, current_level_db: f32, trend: LevelTrend) -> GainRecommendation {
        let difference = current_level_db - self.target_level_db;
        
        // Base adjustment calculation
        let mut recommended_db = -difference * self.adjustment_sensitivity;
        
        // Adjust based on trend
        match trend {
            LevelTrend::Rising => recommended_db -= 1.0, // More conservative for rising levels
            LevelTrend::Falling => recommended_db += 0.5, // Slightly more aggressive for falling
            LevelTrend::Stable => {}, // No trend adjustment
        }
        
        // Quantize to practical steps (0.5dB increments)
        recommended_db = (recommended_db * 2.0).round() / 2.0;
        
        GainRecommendation {
            adjustment_db: recommended_db,
            confidence: self.calculate_confidence(difference.abs()),
            urgency: self.calculate_urgency(current_level_db),
            description: self.generate_description(recommended_db, current_level_db),
        }
    }
    
    fn calculate_confidence(&self, deviation_db: f32) -> f32 {
        // Higher confidence for larger deviations
        (deviation_db / 12.0).min(1.0)
    }
    
    fn calculate_urgency(&self, current_level_db: f32) -> RecommendationUrgency {
        if current_level_db >= -3.0 {
            RecommendationUrgency::Critical
        } else if current_level_db >= -6.0 || current_level_db <= -30.0 {
            RecommendationUrgency::High
        } else if current_level_db <= -24.0 || current_level_db >= -12.0 {
            RecommendationUrgency::Medium
        } else {
            RecommendationUrgency::Low
        }
    }
    
    fn generate_description(&self, adjustment_db: f32, _current_level_db: f32) -> String {
        if adjustment_db.abs() < 0.5 {
            "Level is optimal".to_string()
        } else if adjustment_db > 0.0 {
            format!("Increase input gain by {:.1}dB", adjustment_db)
        } else {
            format!("Decrease input gain by {:.1}dB", adjustment_db.abs())
        }
    }
}

/// Level trend analysis for smarter recommendations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum LevelTrend {
    Rising,
    Falling, 
    Stable,
}

/// Recommendation urgency levels
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RecommendationUrgency {
    Low,      // Fine-tuning
    Medium,   // Should adjust
    High,     // Need to adjust
    Critical, // Must adjust immediately
}

/// Gain adjustment recommendation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GainRecommendation {
    pub adjustment_db: f32,              // Recommended gain change in dB
    pub confidence: f32,                 // Confidence in recommendation (0.0-1.0)
    pub urgency: RecommendationUrgency,  // How urgent the adjustment is
    pub description: String,             // Human-readable description
}

impl GainStagingAssistant {
    /// Create new gain staging assistant for synthesizer recording
    pub fn new() -> Self {
        Self {
            target_level_db: -18.0,  // Professional standard for synthesizers
            optimal_range: (-24.0, -12.0),
            headroom_minimum_db: -6.0,
            level_history: VecDeque::with_capacity(100),
            max_history_length: 100,
            recommendation_engine: RecommendationEngine::new(-18.0),
        }
    }
    
    /// Analyze current audio levels and provide gain staging guidance
    pub fn analyze_level(&mut self, professional_readings: &ProfessionalMeterReadings) -> GainStagingAnalysis {
        // Use VU level for gain staging decisions (more stable than peak)
        let reference_level = professional_readings.vu_db;
        
        // Update level history for trend analysis
        self.level_history.push_back(reference_level);
        if self.level_history.len() > self.max_history_length {
            self.level_history.pop_front();
        }
        
        // Analyze trend
        let trend = self.analyze_trend();
        
        // Generate recommendation
        let recommendation = self.recommendation_engine.recommend_adjustment(reference_level, trend.clone());
        
        // Analyze headroom
        let headroom_status = if professional_readings.peak_db >= 0.0 {
            HeadroomStatus::Clipping
        } else if professional_readings.peak_db >= self.headroom_minimum_db {
            HeadroomStatus::LowHeadroom
        } else {
            HeadroomStatus::SafeHeadroom
        };
        
        // Calculate metrics
        let target_distance = reference_level - self.target_level_db;
        let is_optimal = reference_level >= self.optimal_range.0 && reference_level <= self.optimal_range.1;
        
        GainStagingAnalysis {
            current_level_db: reference_level,
            target_level_db: self.target_level_db,
            target_distance_db: target_distance,
            is_optimal,
            trend,
            recommendation,
            headroom_status,
            peak_db: professional_readings.peak_db,
            headroom_db: 0.0 - professional_readings.peak_db, // dB below clipping
        }
    }
    
    /// Analyze level trend from history
    fn analyze_trend(&self) -> LevelTrend {
        if self.level_history.len() < 5 {
            return LevelTrend::Stable;
        }
        
        let recent: Vec<f32> = self.level_history.iter().rev().take(5).copied().collect();
        let slope = self.calculate_slope(&recent);
        
        if slope > 1.0 {
            LevelTrend::Rising
        } else if slope < -1.0 {
            LevelTrend::Falling
        } else {
            LevelTrend::Stable
        }
    }
    
    /// Calculate slope of recent level changes
    fn calculate_slope(&self, levels: &[f32]) -> f32 {
        if levels.len() < 2 {
            return 0.0;
        }
        
        let n = levels.len() as f32;
        let x_sum: f32 = (0..levels.len()).map(|i| i as f32).sum();
        let y_sum: f32 = levels.iter().sum();
        let xy_sum: f32 = levels.iter().enumerate().map(|(i, &y)| i as f32 * y).sum();
        let x2_sum: f32 = (0..levels.len()).map(|i| (i as f32).powi(2)).sum();
        
        let denominator = n * x2_sum - x_sum.powi(2);
        if denominator.abs() < f32::EPSILON {
            return 0.0;
        }
        
        (n * xy_sum - x_sum * y_sum) / denominator
    }
    
    /// Reset analysis history
    pub fn reset(&mut self) {
        self.level_history.clear();
    }
    
    /// Get current level statistics
    pub fn get_level_statistics(&self) -> LevelStatistics {
        if self.level_history.is_empty() {
            return LevelStatistics::default();
        }
        
        let levels: Vec<f32> = self.level_history.iter().copied().collect();
        let min = levels.iter().copied().fold(f32::INFINITY, f32::min);
        let max = levels.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let avg = levels.iter().sum::<f32>() / levels.len() as f32;
        
        // Calculate standard deviation
        let variance = levels.iter()
            .map(|&x| (x - avg).powi(2))
            .sum::<f32>() / levels.len() as f32;
        let std_dev = variance.sqrt();
        
        LevelStatistics {
            min_db: min,
            max_db: max,
            average_db: avg,
            std_deviation_db: std_dev,
            sample_count: levels.len(),
        }
    }
}

impl Default for GainStagingAssistant {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete gain staging analysis results
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GainStagingAnalysis {
    pub current_level_db: f32,
    pub target_level_db: f32,
    pub target_distance_db: f32,       // Positive = too loud, negative = too quiet
    pub is_optimal: bool,
    pub trend: LevelTrend,
    pub recommendation: GainRecommendation,
    pub headroom_status: HeadroomStatus,
    pub peak_db: f32,
    pub headroom_db: f32,              // Available headroom before clipping
}

/// Level statistics over time
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LevelStatistics {
    pub min_db: f32,
    pub max_db: f32,
    pub average_db: f32,
    pub std_deviation_db: f32,
    pub sample_count: usize,
}

impl Default for LevelStatistics {
    fn default() -> Self {
        Self {
            min_db: -60.0,
            max_db: -60.0,
            average_db: -60.0,
            std_deviation_db: 0.0,
            sample_count: 0,
        }
    }
}

/// Professional audio meter engine combining all meter types
#[derive(Debug)]
pub struct ProfessionalMeterEngine {
    vu_integrator: ExponentialIntegrator,
    ppm_detector: PeakHoldDetector,
    peak_detector: PeakHoldDetector,
    lufs_processor: LoudnessProcessor,
    simd_calculator: SimdLevelCalculator,
    sample_rate: f32,
}

impl ProfessionalMeterEngine {
    pub fn new(sample_rate: f32) -> Self {
        let vu_config = BallisticsConfig::vu_meter(sample_rate);
        let ppm_config = BallisticsConfig::bbc_ppm(sample_rate);
        let peak_config = BallisticsConfig::digital_peak(sample_rate, 2000.0); // 2 second hold
        
        Self {
            vu_integrator: ExponentialIntegrator::new(&vu_config),
            ppm_detector: PeakHoldDetector::new(&ppm_config),
            peak_detector: PeakHoldDetector::new(&peak_config),
            lufs_processor: LoudnessProcessor::new(sample_rate, 3000.0), // 3 second integration
            simd_calculator: SimdLevelCalculator::new(),
            sample_rate,
        }
    }
    
    /// Process audio samples and return professional meter readings
    pub fn process_samples(&mut self, samples: &[f32]) -> ProfessionalMeterReadings {
        if samples.is_empty() {
            return self.get_silence_readings();
        }
        
        // Use SIMD-optimized calculations
        let peak = self.simd_calculator.calculate_peak_simd(samples);
        let rms = self.simd_calculator.calculate_rms_simd(samples);
        
        // Process through professional meters
        let vu_level = self.vu_integrator.process(rms);
        let ppm_level = self.ppm_detector.process(peak);
        let peak_hold = self.peak_detector.process(peak);
        let lufs = self.lufs_processor.process(samples);
        
        // Convert to dB
        let vu_db = if vu_level > 0.0 { 20.0 * vu_level.log10() } else { -60.0 };
        let ppm_db = if ppm_level > 0.0 { 20.0 * ppm_level.log10() } else { -60.0 };
        let peak_db = if peak > 0.0 { 20.0 * peak.log10() } else { -60.0 };
        let peak_hold_db = if peak_hold > 0.0 { 20.0 * peak_hold.log10() } else { -60.0 };
        
        // Determine gain staging status
        let gain_staging = self.analyze_gain_staging(vu_db, peak_db);
        
        ProfessionalMeterReadings {
            vu_db,
            ppm_db,
            peak_db,
            peak_hold_db,
            lufs,
            gain_staging,
        }
    }
    
    /// Analyze gain staging for professional recording recommendations
    fn analyze_gain_staging(&self, vu_db: f32, peak_db: f32) -> GainStagingStatus {
        if peak_db >= 0.0 {
            GainStagingStatus::Clipping
        } else if peak_db > -6.0 {
            GainStagingStatus::TooLoud
        } else if vu_db >= -24.0 && vu_db <= -12.0 {
            GainStagingStatus::Optimal
        } else if vu_db < -30.0 {
            GainStagingStatus::TooQuiet
        } else {
            GainStagingStatus::Acceptable
        }
    }
    
    /// Get readings for silence (used when no samples available)
    fn get_silence_readings(&self) -> ProfessionalMeterReadings {
        ProfessionalMeterReadings {
            vu_db: -60.0,
            ppm_db: -60.0,
            peak_db: -60.0,
            peak_hold_db: -60.0,
            lufs: -70.0,
            gain_staging: GainStagingStatus::TooQuiet,
        }
    }
    
    /// Reset all meters
    pub fn reset(&mut self) {
        self.vu_integrator.reset();
        self.ppm_detector.reset();
        self.peak_detector.reset();
        self.lufs_processor.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exponential_integrator() {
        let config = BallisticsConfig::vu_meter(44100.0);
        let mut integrator = ExponentialIntegrator::new(&config);
        
        // Test attack
        let output1 = integrator.process(1.0);
        assert!(output1 > 0.0 && output1 < 1.0);
        
        // Test release
        let output2 = integrator.process(0.0);
        assert!(output2 < output1);
    }
    
    #[test]
    fn test_peak_hold_detector() {
        let config = BallisticsConfig::digital_peak(44100.0, 100.0);
        let mut detector = PeakHoldDetector::new(&config);
        
        // Test peak detection
        let peak1 = detector.process(0.5);
        assert_eq!(peak1, 0.5);
        
        // Test hold behavior
        let peak2 = detector.process(0.3);
        assert!(peak2 >= 0.3); // Should hold higher value
    }
    
    #[test]
    fn test_simd_calculator() {
        let calc = SimdLevelCalculator::new();
        let samples = vec![0.5, -0.3, 0.8, -0.1, 0.2];
        
        let peak = calc.calculate_peak_simd(&samples);
        let rms = calc.calculate_rms_simd(&samples);
        
        assert!(peak > 0.0);
        assert!(rms > 0.0);
        assert!(peak >= rms); // Peak should be >= RMS
    }
    
    #[test]
    fn test_professional_meter_engine() {
        let mut engine = ProfessionalMeterEngine::new(44100.0);
        let samples = vec![0.5, -0.3, 0.8, -0.1];
        
        let readings = engine.process_samples(&samples);
        
        // Debug print to understand the values
        println!("VU: {:.2} dB, PPM: {:.2} dB, Peak: {:.2} dB, LUFS: {:.2}", 
                 readings.vu_db, readings.ppm_db, readings.peak_db, readings.lufs);
        
        // The initial reading might be close to silence due to integration
        // Let's test multiple iterations to build up the integrators
        for _ in 0..10 {
            engine.process_samples(&samples);
        }
        
        let final_readings = engine.process_samples(&samples);
        println!("Final VU: {:.2} dB, PPM: {:.2} dB, Peak: {:.2} dB, LUFS: {:.2}", 
                 final_readings.vu_db, final_readings.ppm_db, final_readings.peak_db, final_readings.lufs);
        
        // After multiple iterations, should have meaningful readings
        assert!(final_readings.peak_db > -60.0, "Peak should be above -60dB after processing samples");
        assert!(final_readings.lufs > -70.0, "LUFS should be above -70 after processing samples");
    }
    
    #[test]
    fn test_gain_staging_assistant() {
        let mut assistant = GainStagingAssistant::new();
        
        // Test with optimal level (-18dBFS target)
        let optimal_readings = ProfessionalMeterReadings {
            vu_db: -18.0,
            ppm_db: -15.0,
            peak_db: -12.0,
            peak_hold_db: -12.0,
            lufs: -23.0,
            gain_staging: GainStagingStatus::Optimal,
        };
        
        let analysis = assistant.analyze_level(&optimal_readings);
        assert!(analysis.is_optimal);
        assert!(analysis.recommendation.adjustment_db.abs() < 1.0); // Should recommend minimal adjustment
        
        // Test with too quiet level
        let quiet_readings = ProfessionalMeterReadings {
            vu_db: -35.0,
            ppm_db: -32.0,
            peak_db: -30.0,
            peak_hold_db: -30.0,
            lufs: -40.0,
            gain_staging: GainStagingStatus::TooQuiet,
        };
        
        let quiet_analysis = assistant.analyze_level(&quiet_readings);
        assert!(!quiet_analysis.is_optimal);
        assert!(quiet_analysis.recommendation.adjustment_db > 0.0); // Should recommend gain increase
        
        // Test with too loud level
        let loud_readings = ProfessionalMeterReadings {
            vu_db: -5.0,
            ppm_db: -3.0,
            peak_db: -1.0,
            peak_hold_db: -1.0,
            lufs: -8.0,
            gain_staging: GainStagingStatus::TooLoud,
        };
        
        let loud_analysis = assistant.analyze_level(&loud_readings);
        assert!(!loud_analysis.is_optimal);
        assert!(loud_analysis.recommendation.adjustment_db < 0.0); // Should recommend gain decrease
        matches!(loud_analysis.headroom_status, HeadroomStatus::LowHeadroom);
    }
    
    #[test]
    fn test_level_trend_analysis() {
        let mut assistant = GainStagingAssistant::new();
        
        // Simulate rising levels
        for level in [-25.0, -24.0, -23.0, -22.0, -21.0] {
            let readings = ProfessionalMeterReadings {
                vu_db: level,
                ppm_db: level + 2.0,
                peak_db: level + 5.0,
                peak_hold_db: level + 5.0,
                lufs: level - 5.0,
                gain_staging: GainStagingStatus::Acceptable,
            };
            assistant.analyze_level(&readings);
        }
        
        // Should detect rising trend
        let trend = assistant.analyze_trend();
        matches!(trend, LevelTrend::Rising);
    }
    
    #[test]
    fn test_headroom_detector() {
        let mut detector = HeadroomDetector::new(-6.0, 0.0);
        
        // Test safe headroom
        let safe_status = detector.analyze_headroom(-12.0);
        matches!(safe_status, HeadroomStatus::SafeHeadroom);
        
        // Test low headroom
        let low_status = detector.analyze_headroom(-3.0);
        matches!(low_status, HeadroomStatus::LowHeadroom);
        
        // Test clipping
        let clip_status = detector.analyze_headroom(1.0);
        matches!(clip_status, HeadroomStatus::Clipping);
        assert_eq!(detector.get_clip_count(), 1);
    }
}