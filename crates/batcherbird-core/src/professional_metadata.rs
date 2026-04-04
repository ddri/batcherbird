use crate::{BatcherbirdError, Result};
use std::collections::HashMap;

/// SMPL chunk for WAV files - industry standard sampler metadata
#[derive(Debug, Clone)]
pub struct SmplChunk {
    /// Manufacturer ID (0 for unknown)
    pub manufacturer: u32,

    /// Product ID (0 for unknown)  
    pub product: u32,

    /// Sample period in nanoseconds (1/sample_rate * 1e9)
    pub sample_period: u32,

    /// MIDI unity note (60 = middle C)
    pub midi_unity_note: u32,

    /// MIDI pitch fraction (0 to 0xFFFFFFFF for fractional semitone)
    pub midi_pitch_fraction: u32,

    /// SMPTE format (0 = no SMPTE offset)
    pub smpte_format: u32,

    /// SMPTE offset in subframes
    pub smpte_offset: u32,

    /// Sample loops defined in this chunk
    pub loops: Vec<SampleLoop>,

    /// Additional sampler-specific data
    pub sampler_data: Vec<u8>,
}

/// Individual sample loop within SMPL chunk
#[derive(Debug, Clone)]
pub struct SampleLoop {
    /// Unique identifier for this loop
    pub cue_point_id: u32,

    /// Loop type: 0 = forward, 1 = alternating, 2 = backward
    pub loop_type: u32,

    /// Loop start position in sample frames
    pub start: u32,

    /// Loop end position in sample frames  
    pub end: u32,

    /// Fractional sample adjustment (0 to 0xFFFFFFFF)
    pub fraction: u32,

    /// Number of times to play loop (0 = infinite)
    pub play_count: u32,
}

impl SmplChunk {
    /// Create new SMPL chunk with standard defaults
    pub fn new(sample_rate: u32) -> Self {
        Self {
            manufacturer: 0,
            product: 0,
            sample_period: (1_000_000_000.0 / sample_rate as f64) as u32,
            midi_unity_note: 60, // Middle C
            midi_pitch_fraction: 0,
            smpte_format: 0,
            smpte_offset: 0,
            loops: Vec::new(),
            sampler_data: Vec::new(),
        }
    }

    /// Add a loop to this SMPL chunk
    pub fn add_loop(&mut self, start_sample: u32, end_sample: u32, loop_type: u32) {
        let loop_obj = SampleLoop {
            cue_point_id: self.loops.len() as u32,
            loop_type,
            start: start_sample,
            end: end_sample,
            fraction: 0,
            play_count: 0, // Infinite
        };

        self.loops.push(loop_obj);
    }

    /// Serialize SMPL chunk to bytes for WAV file embedding
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // SMPL chunk header
        bytes.extend_from_slice(b"smpl");

        // Calculate chunk size (excluding chunk ID and size fields)
        let base_size = 36; // Fixed SMPL data
        let loops_size = self.loops.len() * 24; // 24 bytes per loop
        let sampler_data_size = self.sampler_data.len();
        let chunk_size = base_size + loops_size + sampler_data_size;

        bytes.extend_from_slice(&(chunk_size as u32).to_le_bytes());

        // SMPL chunk data
        bytes.extend_from_slice(&self.manufacturer.to_le_bytes());
        bytes.extend_from_slice(&self.product.to_le_bytes());
        bytes.extend_from_slice(&self.sample_period.to_le_bytes());
        bytes.extend_from_slice(&self.midi_unity_note.to_le_bytes());
        bytes.extend_from_slice(&self.midi_pitch_fraction.to_le_bytes());
        bytes.extend_from_slice(&self.smpte_format.to_le_bytes());
        bytes.extend_from_slice(&self.smpte_offset.to_le_bytes());
        bytes.extend_from_slice(&(self.loops.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(self.sampler_data.len() as u32).to_le_bytes());

        // Loop data
        for loop_data in &self.loops {
            bytes.extend_from_slice(&loop_data.cue_point_id.to_le_bytes());
            bytes.extend_from_slice(&loop_data.loop_type.to_le_bytes());
            bytes.extend_from_slice(&loop_data.start.to_le_bytes());
            bytes.extend_from_slice(&loop_data.end.to_le_bytes());
            bytes.extend_from_slice(&loop_data.fraction.to_le_bytes());
            bytes.extend_from_slice(&loop_data.play_count.to_le_bytes());
        }

        // Sampler-specific data
        bytes.extend_from_slice(&self.sampler_data);

        // Pad to even byte boundary
        if bytes.len() % 2 != 0 {
            bytes.push(0);
        }

        bytes
    }
}

/// Broadcast WAV metadata for professional applications
#[derive(Debug, Clone)]
pub struct BroadcastWavChunk {
    /// Description of the sound (256 chars max)
    pub description: String,

    /// Name of the originator (32 chars max)
    pub originator: String,

    /// Reference of the originator (32 chars max)
    pub originator_reference: String,

    /// Date of creation (YYYY-MM-DD format)
    pub origination_date: String,

    /// Time of creation (HH:MM:SS format)
    pub origination_time: String,

    /// First sample count since midnight (64-bit)
    pub time_reference: u64,

    /// BWF version (0x0001 for version 1)
    pub version: u16,

    /// UMID (Unique Material Identifier)
    pub umid: [u8; 64],

    /// Reserved for future use
    pub reserved: [u8; 190],

    /// History/coding information
    pub coding_history: String,
}

impl BroadcastWavChunk {
    /// Create new Broadcast WAV chunk with current timestamp
    pub fn new(description: &str, originator: &str) -> Self {
        let now = chrono::Utc::now();

        Self {
            description: description.chars().take(256).collect(),
            originator: originator.chars().take(32).collect(),
            originator_reference: format!(
                "BBIRD_{}",
                uuid::Uuid::new_v4()
                    .simple()
                    .to_string()
                    .chars()
                    .take(26)
                    .collect::<String>()
            ),
            origination_date: now.format("%Y-%m-%d").to_string(),
            origination_time: now.format("%H:%M:%S").to_string(),
            time_reference: 0,
            version: 1,
            umid: [0; 64],
            reserved: [0; 190],
            coding_history: "A=PCM,F=44100,W=16,M=stereo,T=BatcherBird\r\n".to_string(),
        }
    }

    /// Serialize Broadcast WAV chunk to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // BEXT chunk header
        bytes.extend_from_slice(b"bext");

        // Calculate chunk size
        let base_size = 602; // Fixed BEXT data size
        let coding_history_size = self.coding_history.len();
        let chunk_size = base_size + coding_history_size;

        bytes.extend_from_slice(&(chunk_size as u32).to_le_bytes());

        // Description (256 bytes, null-padded)
        let mut desc_bytes = self.description.as_bytes().to_vec();
        desc_bytes.resize(256, 0);
        bytes.extend_from_slice(&desc_bytes);

        // Originator (32 bytes, null-padded)
        let mut orig_bytes = self.originator.as_bytes().to_vec();
        orig_bytes.resize(32, 0);
        bytes.extend_from_slice(&orig_bytes);

        // Originator reference (32 bytes, null-padded)
        let mut orig_ref_bytes = self.originator_reference.as_bytes().to_vec();
        orig_ref_bytes.resize(32, 0);
        bytes.extend_from_slice(&orig_ref_bytes);

        // Origination date (10 bytes, null-padded)
        let mut date_bytes = self.origination_date.as_bytes().to_vec();
        date_bytes.resize(10, 0);
        bytes.extend_from_slice(&date_bytes);

        // Origination time (8 bytes, null-padded)
        let mut time_bytes = self.origination_time.as_bytes().to_vec();
        time_bytes.resize(8, 0);
        bytes.extend_from_slice(&time_bytes);

        // Time reference (8 bytes)
        bytes.extend_from_slice(&self.time_reference.to_le_bytes());

        // Version (2 bytes)
        bytes.extend_from_slice(&self.version.to_le_bytes());

        // UMID (64 bytes)
        bytes.extend_from_slice(&self.umid);

        // Reserved (190 bytes)
        bytes.extend_from_slice(&self.reserved);

        // Coding history (variable length, null-terminated)
        bytes.extend_from_slice(self.coding_history.as_bytes());
        bytes.push(0); // Null terminator

        // Pad to even byte boundary
        if bytes.len() % 2 != 0 {
            bytes.push(0);
        }

        bytes
    }
}

/// Quality metrics for professional loop assessment
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    /// Signal-to-noise ratio in dB
    pub snr_db: f32,

    /// Dynamic range in dB
    pub dynamic_range_db: f32,

    /// Peak-to-average ratio
    pub peak_to_average_ratio: f32,

    /// Spectral centroid (frequency content center)
    pub spectral_centroid_hz: f32,

    /// Harmonic-to-noise ratio
    pub harmonic_noise_ratio: f32,

    /// Overall quality score (0.0 to 1.0)
    pub overall_quality: f32,
}

impl QualityMetrics {
    /// Calculate quality metrics from audio data
    pub fn analyze(audio_data: &[f32], sample_rate: u32) -> Self {
        let mut metrics = Self {
            snr_db: 0.0,
            dynamic_range_db: 0.0,
            peak_to_average_ratio: 0.0,
            spectral_centroid_hz: 0.0,
            harmonic_noise_ratio: 0.0,
            overall_quality: 0.0,
        };

        if audio_data.is_empty() {
            return metrics;
        }

        // Calculate RMS and peak values
        let rms = (audio_data.iter().map(|&x| x * x).sum::<f32>() / audio_data.len() as f32).sqrt();
        let peak = audio_data.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);

        // Peak-to-average ratio
        metrics.peak_to_average_ratio = if rms > 0.0 { peak / rms } else { 0.0 };

        // Estimate noise floor (bottom 10% of magnitude values)
        let mut magnitudes: Vec<f32> = audio_data.iter().map(|&x| x.abs()).collect();
        magnitudes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let noise_floor = magnitudes[magnitudes.len() / 10];

        // Signal-to-noise ratio
        metrics.snr_db = if noise_floor > 0.0 {
            20.0 * (rms / noise_floor).log10()
        } else {
            60.0 // Assume good SNR if no detectable noise
        };

        // Dynamic range (difference between peak and noise floor)
        metrics.dynamic_range_db = if noise_floor > 0.0 {
            20.0 * (peak / noise_floor).log10()
        } else {
            80.0 // Assume good dynamic range
        };

        // Simple spectral centroid estimation (would normally require FFT)
        // For now, use a placeholder based on signal characteristics
        metrics.spectral_centroid_hz = sample_rate as f32 * 0.25; // Rough estimate

        // Harmonic-to-noise ratio (simplified)
        metrics.harmonic_noise_ratio = metrics.snr_db * 0.8; // Approximate relationship

        // Overall quality score (weighted combination)
        let normalized_snr = (metrics.snr_db / 60.0).clamp(0.0, 1.0);
        let normalized_dynamic_range = (metrics.dynamic_range_db / 80.0).clamp(0.0, 1.0);
        let normalized_par = (1.0 / metrics.peak_to_average_ratio.max(1.0)).clamp(0.0, 1.0);

        metrics.overall_quality =
            (normalized_snr * 0.4) + (normalized_dynamic_range * 0.3) + (normalized_par * 0.3);

        metrics
    }
}

/// Sampler format compatibility flags
#[derive(Debug, Clone)]
pub struct SamplerCompatibility {
    /// Compatible with DecentSampler format
    pub decent_sampler: bool,

    /// Compatible with SFZ format
    pub sfz: bool,

    /// Compatible with Kontakt format (basic)
    pub kontakt: bool,

    /// Compatible with Logic EXS24
    pub exs24: bool,

    /// Compatible with HALion
    pub halion: bool,
}

impl Default for SamplerCompatibility {
    fn default() -> Self {
        Self {
            decent_sampler: true,
            sfz: true,
            kontakt: true,
            exs24: false,  // Requires additional work
            halion: false, // Requires additional work
        }
    }
}

/// Professional metadata container for sample libraries
#[derive(Debug, Clone)]
pub struct ProfessionalMetadata {
    /// SMPL chunk for sampler compatibility
    pub smpl_chunk: Option<SmplChunk>,

    /// Broadcast WAV metadata for professional workflows
    pub broadcast_wav: Option<BroadcastWavChunk>,

    /// Quality assessment metrics
    pub quality_metrics: Option<QualityMetrics>,

    /// Sampler format compatibility information
    pub compatibility: SamplerCompatibility,

    /// Custom metadata fields
    pub custom_fields: HashMap<String, String>,
}

impl Default for ProfessionalMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfessionalMetadata {
    /// Create new professional metadata container
    pub fn new() -> Self {
        Self {
            smpl_chunk: None,
            broadcast_wav: None,
            quality_metrics: None,
            compatibility: SamplerCompatibility::default(),
            custom_fields: HashMap::new(),
        }
    }

    /// Set loop points with SMPL chunk
    pub fn set_loop_points(&mut self, sample_rate: u32, start_sample: u32, end_sample: u32) {
        let mut smpl = self
            .smpl_chunk
            .take()
            .unwrap_or_else(|| SmplChunk::new(sample_rate));
        smpl.add_loop(start_sample, end_sample, 0); // Forward loop
        self.smpl_chunk = Some(smpl);
    }

    /// Set broadcast WAV metadata
    pub fn set_broadcast_metadata(&mut self, description: &str, originator: &str) {
        self.broadcast_wav = Some(BroadcastWavChunk::new(description, originator));
    }

    /// Analyze and set quality metrics
    pub fn analyze_quality(&mut self, audio_data: &[f32], sample_rate: u32) {
        self.quality_metrics = Some(QualityMetrics::analyze(audio_data, sample_rate));
    }

    /// Add custom metadata field
    pub fn add_custom_field(&mut self, key: String, value: String) {
        self.custom_fields.insert(key, value);
    }

    /// Generate complete metadata for WAV file embedding
    pub fn generate_wav_chunks(&self) -> Vec<Vec<u8>> {
        let mut chunks = Vec::new();

        if let Some(ref smpl) = self.smpl_chunk {
            chunks.push(smpl.to_bytes());
        }

        if let Some(ref bext) = self.broadcast_wav {
            chunks.push(bext.to_bytes());
        }

        chunks
    }

    /// Validate metadata for professional standards compliance
    pub fn validate(&self) -> Result<()> {
        // Check SMPL chunk validity
        if let Some(ref smpl) = self.smpl_chunk {
            for loop_data in &smpl.loops {
                if loop_data.start >= loop_data.end {
                    return Err(BatcherbirdError::Audio(
                        "Invalid loop points: start >= end".to_string(),
                    ));
                }
            }
        }

        // Check Broadcast WAV validity
        if let Some(ref bext) = self.broadcast_wav {
            if bext.description.is_empty() {
                return Err(BatcherbirdError::Audio(
                    "Broadcast WAV description cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Metadata engine for professional sample library creation
pub struct MetadataEngine {
    default_originator: String,
}

impl MetadataEngine {
    pub fn new(originator: &str) -> Self {
        Self {
            default_originator: originator.to_string(),
        }
    }

    /// Create metadata for a sample with loop points
    pub fn create_sample_metadata(
        &self,
        audio_data: &[f32],
        sample_rate: u32,
        loop_start: Option<u32>,
        loop_end: Option<u32>,
        description: &str,
    ) -> Result<ProfessionalMetadata> {
        let mut metadata = ProfessionalMetadata::new();

        // Set broadcast WAV metadata
        metadata.set_broadcast_metadata(description, &self.default_originator);

        // Add loop points if provided
        if let (Some(start), Some(end)) = (loop_start, loop_end) {
            metadata.set_loop_points(sample_rate, start, end);
        }

        // Analyze quality
        metadata.analyze_quality(audio_data, sample_rate);

        // Add standard custom fields
        metadata.add_custom_field("generator".to_string(), "BatcherBird".to_string());
        metadata.add_custom_field("version".to_string(), env!("CARGO_PKG_VERSION").to_string());

        // Validate before returning
        metadata.validate()?;

        Ok(metadata)
    }

    /// Create metadata for a complete sample library
    pub fn create_library_metadata(
        &self,
        samples: &[(String, &[f32], u32)], // (name, audio_data, sample_rate)
        library_name: &str,
    ) -> Result<Vec<(String, ProfessionalMetadata)>> {
        let mut library_metadata = Vec::new();

        for (name, audio_data, sample_rate) in samples {
            let description = format!("{} - {}", library_name, name);
            let metadata =
                self.create_sample_metadata(audio_data, *sample_rate, None, None, &description)?;
            library_metadata.push((name.clone(), metadata));
        }

        Ok(library_metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smpl_chunk_creation() {
        let mut smpl = SmplChunk::new(44100);
        smpl.add_loop(1000, 5000, 0);

        assert_eq!(smpl.sample_period, 22675); // 1/44100 * 1e9
        assert_eq!(smpl.midi_unity_note, 60);
        assert_eq!(smpl.loops.len(), 1);

        let loop_data = &smpl.loops[0];
        assert_eq!(loop_data.start, 1000);
        assert_eq!(loop_data.end, 5000);
        assert_eq!(loop_data.loop_type, 0);
    }

    #[test]
    fn test_smpl_chunk_serialization() {
        let mut smpl = SmplChunk::new(44100);
        smpl.add_loop(1000, 5000, 0);

        let bytes = smpl.to_bytes();

        // Check chunk header
        assert_eq!(&bytes[0..4], b"smpl");

        // Check chunk size (little-endian)
        let chunk_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        assert_eq!(chunk_size, 60); // 36 base + 24 for one loop

        // Verify structure
        assert!(bytes.len() >= 68); // Header + chunk size + data
    }

    #[test]
    fn test_broadcast_wav_chunk() {
        let bext = BroadcastWavChunk::new("Test sample", "BatcherBird");

        assert_eq!(bext.description, "Test sample");
        assert_eq!(bext.originator, "BatcherBird");
        assert!(bext.originator_reference.starts_with("BBIRD_"));
        assert_eq!(bext.version, 1);

        let bytes = bext.to_bytes();

        // Check chunk header
        assert_eq!(&bytes[0..4], b"bext");

        // Check chunk has reasonable size
        assert!(bytes.len() > 600); // At least the base size
    }

    #[test]
    fn test_quality_metrics() {
        // Create test signal with known characteristics
        let sample_rate = 44100;
        let duration = 1.0; // 1 second
        let frequency = 440.0; // A4
        let samples = (duration * sample_rate as f32) as usize;

        let audio_data: Vec<f32> = (0..samples)
            .map(|i| {
                0.5 * (2.0 * std::f32::consts::PI * frequency * i as f32 / sample_rate as f32).sin()
            })
            .collect();

        let metrics = QualityMetrics::analyze(&audio_data, sample_rate);

        assert!(metrics.peak_to_average_ratio > 1.0); // Sine wave should have clear peak
        assert!(metrics.snr_db > 0.0); // Should have positive SNR
        assert!(metrics.overall_quality > 0.3); // Should be reasonable quality
    }

    #[test]
    fn test_professional_metadata() {
        let sample_rate = 44100;
        let audio_data = vec![0.5, -0.5, 0.3, -0.3, 0.0]; // Simple test signal

        let mut metadata = ProfessionalMetadata::new();
        metadata.set_broadcast_metadata("Test", "BatcherBird");
        metadata.set_loop_points(sample_rate, 100, 4000);
        metadata.analyze_quality(&audio_data, sample_rate);
        metadata.add_custom_field("genre".to_string(), "Synthesizer".to_string());

        assert!(metadata.smpl_chunk.is_some());
        assert!(metadata.broadcast_wav.is_some());
        assert!(metadata.quality_metrics.is_some());
        assert_eq!(
            metadata.custom_fields.get("genre"),
            Some(&"Synthesizer".to_string())
        );

        // Test chunk generation
        let chunks = metadata.generate_wav_chunks();
        assert_eq!(chunks.len(), 2); // SMPL + BEXT

        // Test validation
        assert!(metadata.validate().is_ok());
    }

    #[test]
    fn test_metadata_engine() {
        let engine = MetadataEngine::new("TestUser");
        let audio_data = vec![1.0, 0.5, 0.0, -0.5, -1.0];

        let metadata = engine
            .create_sample_metadata(
                &audio_data,
                44100,
                Some(100),
                Some(4000),
                "Test synthesizer sample",
            )
            .unwrap();

        assert!(metadata.smpl_chunk.is_some());
        assert!(metadata.broadcast_wav.is_some());
        assert!(metadata.quality_metrics.is_some());

        let smpl = metadata.smpl_chunk.unwrap();
        assert_eq!(smpl.loops.len(), 1);
        assert_eq!(smpl.loops[0].start, 100);
        assert_eq!(smpl.loops[0].end, 4000);
    }

    #[test]
    fn test_metadata_validation() {
        let mut metadata = ProfessionalMetadata::new();

        // Valid metadata should pass
        metadata.set_broadcast_metadata("Valid description", "BatcherBird");
        metadata.set_loop_points(44100, 100, 4000);
        assert!(metadata.validate().is_ok());

        // Invalid loop points should fail
        metadata.set_loop_points(44100, 4000, 100); // start > end
        assert!(metadata.validate().is_err());
    }
}
