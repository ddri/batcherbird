use crate::{Result, BatcherbirdError};

/// Sample detection configuration for automatic trimming
#[derive(Debug, Clone)]
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
            threshold_db: -40.0,        // Conservative threshold
            window_size_ms: 10.0,       // 10ms windows (good balance)
            min_sample_length_ms: 100.0, // Minimum 100ms samples
            pre_trigger_ms: 20.0,       // 20ms pre-trigger
            post_trigger_ms: 200.0,     // 200ms for reverb tails
            confirmation_windows: 3,    // 3 consecutive windows for stability
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
#[derive(Debug, Clone)]
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

impl SampleDetector {
    pub fn new(config: DetectionConfig) -> Self {
        Self { config }
    }
    
    /// Create detector with default settings
    pub fn default() -> Self {
        Self::new(DetectionConfig::default())
    }
    
    /// Analyze audio and detect sample boundaries
    pub fn detect_boundaries(&self, audio_data: &[f32], sample_rate: u32) -> Result<DetectionResult> {
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
        
        println!("🔍 Starting sample detection on {} samples at {}Hz", audio_data.len(), sample_rate);
        
        // Calculate window size in samples
        let window_size_samples = ((self.config.window_size_ms / 1000.0) * sample_rate as f32) as usize;
        if window_size_samples == 0 {
            return Err(BatcherbirdError::Audio("Window size too small".to_string()));
        }
        
        // Calculate RMS values for each window
        let rms_values = self.calculate_rms_windows(audio_data, window_size_samples);
        
        // Convert threshold from dB to linear
        let threshold_linear = self.db_to_linear(self.config.threshold_db);
        
        println!("   Threshold: {}dB ({:.6} linear)", self.config.threshold_db, threshold_linear);
        println!("   Window size: {}ms ({} samples)", self.config.window_size_ms, window_size_samples);
        println!("   Calculated {} RMS windows", rms_values.len());
        
        // Find start and end points using RMS analysis
        let (detected_start_window, detected_end_window) = self.find_signal_boundaries(&rms_values, threshold_linear)?;
        
        // Convert window indices back to sample indices
        let detected_start_sample = detected_start_window * window_size_samples;
        let detected_end_sample = ((detected_end_window + 1) * window_size_samples).min(audio_data.len());
        
        // Apply pre/post trigger adjustments
        let pre_trigger_samples = ((self.config.pre_trigger_ms / 1000.0) * sample_rate as f32) as usize;
        let post_trigger_samples = ((self.config.post_trigger_ms / 1000.0) * sample_rate as f32) as usize;
        
        let final_start = detected_start_sample.saturating_sub(pre_trigger_samples);
        let final_end = (detected_end_sample + post_trigger_samples).min(audio_data.len());
        
        // Validate minimum length
        let final_length_samples = final_end - final_start;
        let min_length_samples = ((self.config.min_sample_length_ms / 1000.0) * sample_rate as f32) as usize;
        
        if final_length_samples < min_length_samples {
            println!("⚠️  Detected sample too short: {}ms < {}ms minimum", 
                (final_length_samples as f32 / sample_rate as f32) * 1000.0,
                self.config.min_sample_length_ms);
            
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
        
        println!("✅ Detection successful:");
        println!("   Raw detection: samples {}-{} ({:.1}ms-{:.1}ms)", 
            detected_start_sample, detected_end_sample,
            (detected_start_sample as f32 / sample_rate as f32) * 1000.0,
            (detected_end_sample as f32 / sample_rate as f32) * 1000.0);
        println!("   With triggers: samples {}-{} ({:.1}ms-{:.1}ms)",
            final_start, final_end,
            (final_start as f32 / sample_rate as f32) * 1000.0,
            (final_end as f32 / sample_rate as f32) * 1000.0);
        
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
    
    /// Calculate RMS energy for each window
    fn calculate_rms_windows(&self, audio_data: &[f32], window_size: usize) -> Vec<f32> {
        if window_size > audio_data.len() {
            // If window is larger than audio, return single RMS value
            let sum_squares: f32 = audio_data.iter().map(|&x| x * x).sum();
            return vec![(sum_squares / audio_data.len() as f32).sqrt()];
        }
        
        audio_data
            .windows(window_size)
            .step_by(window_size / 2) // 50% overlap for smoother analysis
            .map(|window| {
                let sum_squares: f32 = window.iter().map(|&x| x * x).sum();
                (sum_squares / window.len() as f32).sqrt()
            })
            .collect()
    }
    
    /// Find signal boundaries using RMS analysis with confirmation windows
    fn find_signal_boundaries(&self, rms_values: &[f32], threshold: f32) -> Result<(usize, usize)> {
        if rms_values.is_empty() {
            return Err(BatcherbirdError::Audio("No RMS values to analyze".to_string()));
        }
        
        // Find start: first position where we have enough consecutive windows above threshold
        let start_window = self.find_start_boundary(rms_values, threshold)?;
        
        // Find end: last position where we have enough consecutive windows above threshold  
        let end_window = self.find_end_boundary(rms_values, threshold, start_window)?;
        
        println!("   Signal boundaries: windows {}-{} of {}", start_window, end_window, rms_values.len());
        
        Ok((start_window, end_window))
    }
    
    /// Find start boundary with confirmation windows
    fn find_start_boundary(&self, rms_values: &[f32], threshold: f32) -> Result<usize> {
        for i in 0..rms_values.len() {
            // Check if we have enough consecutive windows above threshold
            let mut consecutive_count = 0;
            for j in i..rms_values.len().min(i + self.config.confirmation_windows) {
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
    fn find_end_boundary(&self, rms_values: &[f32], threshold: f32, start_window: usize) -> Result<usize> {
        // Search backwards from the end
        for i in (start_window..rms_values.len()).rev() {
            // Check if we have enough consecutive windows above threshold working backwards
            let mut consecutive_count = 0;
            for j in (i.saturating_sub(self.config.confirmation_windows - 1)..=i).rev() {
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
            println!("⚠️  Detection failed, returning original audio");
            return audio_data.to_vec();
        }
        
        let start = detection.start_sample.min(audio_data.len());
        let end = detection.end_sample.min(audio_data.len());
        
        if start >= end {
            println!("⚠️  Invalid detection boundaries, returning original audio");
            return audio_data.to_vec();
        }
        
        println!("✂️  Trimming audio: {} -> {} samples ({:.1}% reduction)",
            audio_data.len(),
            end - start,
            ((audio_data.len() - (end - start)) as f32 / audio_data.len() as f32) * 100.0);
        
        audio_data[start..end].to_vec()
    }
}