use crate::{BatcherbirdError, ProfessionalMetadata, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Velocity layer configuration for multi-sample instruments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityLayer {
    /// Minimum velocity for this layer (0-127)
    pub velocity_min: u8,

    /// Maximum velocity for this layer (0-127)
    pub velocity_max: u8,

    /// Crossfade range in velocity units
    pub crossfade_range: u8,

    /// Audio sample path for this layer
    pub sample_path: PathBuf,

    /// Gain offset in dB for level matching
    pub gain_offset: f32,

    /// Loop points for this layer
    pub loop_points: Option<LoopPoints>,

    /// Additional layer-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Loop point definition for sampler formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopPoints {
    /// Loop start position in samples
    pub start_sample: u32,

    /// Loop end position in samples
    pub end_sample: u32,

    /// Loop type: 0 = forward, 1 = alternating, 2 = backward
    pub loop_type: u8,

    /// Crossfade length in samples for seamless loops
    pub crossfade_samples: u32,

    /// Loop tuning offset in cents
    pub tune_cents: f32,
}

/// Round-robin sample group for realistic performance variation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundRobinGroup {
    /// Group identifier
    pub group_id: String,

    /// Sample paths in this round-robin group
    pub sample_paths: Vec<PathBuf>,

    /// Current round-robin position (for playback engines)
    pub current_position: usize,

    /// Randomization amount (0.0 = sequential, 1.0 = fully random)
    pub randomization: f32,
}

/// Release sample configuration for note-off behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseSample {
    /// Path to release sample audio file
    pub sample_path: PathBuf,

    /// Trigger velocity threshold for this release sample
    pub velocity_threshold: u8,

    /// Fade-in time in milliseconds
    pub fade_in_ms: f32,

    /// Offset time from note-off in milliseconds
    pub offset_ms: f32,

    /// Volume adjustment in dB
    pub volume_db: f32,
}

/// Articulation switching for multiple playing techniques
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Articulation {
    /// Articulation name (e.g., "legato", "staccato", "tremolo")
    pub name: String,

    /// MIDI CC or keyswitch trigger
    pub trigger: ArticulationTrigger,

    /// Velocity layers for this articulation
    pub velocity_layers: Vec<VelocityLayer>,

    /// Round-robin groups for this articulation
    pub round_robin_groups: Vec<RoundRobinGroup>,

    /// Release samples for this articulation
    pub release_samples: Vec<ReleaseSample>,
}

/// Trigger mechanism for articulation switching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArticulationTrigger {
    /// MIDI CC controller
    MidiCC {
        controller: u8,
        value_range: (u8, u8),
    },

    /// Keyswitch (specific MIDI note)
    Keyswitch { note: u8, channel: Option<u8> },

    /// Program change
    ProgramChange { program: u8 },

    /// Velocity range
    VelocityRange { min_velocity: u8, max_velocity: u8 },
}

/// Advanced instrument definition with multiple articulations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedInstrument {
    /// Instrument name
    pub name: String,

    /// Instrument category/type
    pub category: String,

    /// MIDI note range
    pub note_range: (u8, u8), // (min_note, max_note)

    /// Default articulation
    pub default_articulation: String,

    /// All available articulations
    pub articulations: HashMap<String, Articulation>,

    /// Global instrument settings
    pub global_settings: InstrumentSettings,

    /// Professional metadata (non-serializable, managed separately)
    #[serde(skip, default = "ProfessionalMetadata::new")]
    pub metadata: ProfessionalMetadata,
}

/// Global instrument settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentSettings {
    /// Master volume in dB
    pub master_volume_db: f32,

    /// Master tuning in cents
    pub master_tune_cents: f32,

    /// Global ADSR envelope
    pub global_envelope: Option<ADSREnvelope>,

    /// Global filter settings
    pub global_filter: Option<FilterSettings>,

    /// Polyphony limit
    pub polyphony_limit: Option<u32>,

    /// Voice allocation mode
    pub voice_allocation: VoiceAllocation,
}

/// ADSR envelope configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ADSREnvelope {
    /// Attack time in seconds
    pub attack_time: f32,

    /// Decay time in seconds
    pub decay_time: f32,

    /// Sustain level (0.0 to 1.0)
    pub sustain_level: f32,

    /// Release time in seconds
    pub release_time: f32,
}

/// Filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterSettings {
    /// Filter type
    pub filter_type: FilterType,

    /// Cutoff frequency in Hz
    pub cutoff_hz: f32,

    /// Resonance/Q factor
    pub resonance: f32,

    /// Filter envelope amount
    pub envelope_amount: f32,
}

/// Filter type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    BandStop,
    AllPass,
}

/// Voice allocation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoiceAllocation {
    /// Oldest voice gets replaced
    Oldest,

    /// Quietest voice gets replaced
    Quietest,

    /// Same note gets replaced
    SameNote,

    /// No replacement (notes get dropped)
    NoReplace,
}

/// Velocity layer generation engine
pub struct VelocityLayerEngine {
    /// Number of velocity layers to generate
    layer_count: usize,

    /// Crossfade overlap percentage
    crossfade_overlap: f32,

    /// Automatic gain compensation
    gain_compensation: bool,
}

impl VelocityLayerEngine {
    pub fn new(layer_count: usize) -> Self {
        Self {
            layer_count,
            crossfade_overlap: 0.1, // 10% overlap
            gain_compensation: true,
        }
    }

    /// Generate velocity layers from a set of samples
    pub fn generate_velocity_layers(&self, samples: &[SampleInfo]) -> Result<Vec<VelocityLayer>> {
        if samples.is_empty() {
            return Ok(Vec::new());
        }

        let velocity_step = 127.0 / self.layer_count as f32;
        let crossfade_range = (velocity_step * self.crossfade_overlap) as u8;

        let mut layers = Vec::new();

        for (i, sample_info) in samples.iter().enumerate().take(self.layer_count) {
            let velocity_min = (i as f32 * velocity_step) as u8;
            let velocity_max = ((i + 1) as f32 * velocity_step).min(127.0) as u8;

            // Calculate gain offset for level matching
            let gain_offset = if self.gain_compensation {
                self.calculate_gain_offset(sample_info, i, samples.len())
            } else {
                0.0
            };

            let layer = VelocityLayer {
                velocity_min,
                velocity_max,
                crossfade_range,
                sample_path: sample_info.path.clone(),
                gain_offset,
                loop_points: sample_info.loop_points.clone(),
                metadata: sample_info.metadata.clone(),
            };

            layers.push(layer);
        }

        Ok(layers)
    }

    fn calculate_gain_offset(
        &self,
        sample_info: &SampleInfo,
        layer_index: usize,
        total_layers: usize,
    ) -> f32 {
        // Simple gain compensation based on layer position
        // Lower velocity layers typically need more gain
        let position_factor = layer_index as f32 / total_layers as f32;
        let base_gain = -6.0 * position_factor; // Up to -6dB reduction for higher velocities

        // Add sample-specific gain adjustment if available
        let sample_gain = sample_info
            .metadata
            .get("gain_offset")
            .and_then(|g| g.parse::<f32>().ok())
            .unwrap_or(0.0);

        base_gain + sample_gain
    }
}

/// Sample information for layer generation
#[derive(Debug, Clone)]
pub struct SampleInfo {
    /// Path to the sample file
    pub path: PathBuf,

    /// Loop points if available
    pub loop_points: Option<LoopPoints>,

    /// Sample metadata
    pub metadata: HashMap<String, String>,

    /// Analyzed peak level in dB
    pub peak_level_db: Option<f32>,

    /// Analyzed RMS level in dB
    pub rms_level_db: Option<f32>,
}

impl SampleInfo {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            loop_points: None,
            metadata: HashMap::new(),
            peak_level_db: None,
            rms_level_db: None,
        }
    }
}

/// Round-robin processor for natural variation
pub struct RoundRobinProcessor {
    max_samples_per_group: usize,
    randomization_enabled: bool,
}

impl RoundRobinProcessor {
    pub fn new(max_samples_per_group: usize) -> Self {
        Self {
            max_samples_per_group,
            randomization_enabled: true,
        }
    }

    /// Create round-robin groups from samples
    pub fn create_round_robin_groups(
        &self,
        samples: &[SampleInfo],
        group_name: &str,
    ) -> Result<Vec<RoundRobinGroup>> {
        if samples.is_empty() {
            return Ok(Vec::new());
        }

        let mut groups = Vec::new();

        for (group_index, chunk) in samples.chunks(self.max_samples_per_group).enumerate() {
            let group_id = format!("{}_{}", group_name, group_index);
            let sample_paths = chunk.iter().map(|s| s.path.clone()).collect();

            let group = RoundRobinGroup {
                group_id,
                sample_paths,
                current_position: 0,
                randomization: if self.randomization_enabled { 0.3 } else { 0.0 }, // 30% randomization
            };

            groups.push(group);
        }

        Ok(groups)
    }
}

/// Advanced sampler format exporter
pub struct AdvancedSamplerExporter {
    /// Enable velocity layer export
    pub velocity_layers: bool,

    /// Enable round-robin export
    pub round_robin: bool,

    /// Enable release sample export
    pub release_samples: bool,

    /// Enable articulation switching
    pub articulations: bool,
}

impl Default for AdvancedSamplerExporter {
    fn default() -> Self {
        Self {
            velocity_layers: true,
            round_robin: true,
            release_samples: false, // Requires additional samples
            articulations: false,   // Requires configuration
        }
    }
}

impl AdvancedSamplerExporter {
    /// Export to DecentSampler format with advanced features
    pub fn export_decent_sampler(
        &self,
        instrument: &AdvancedInstrument,
        output_path: &Path,
    ) -> Result<()> {
        let mut dspreset = DecentSamplerPreset::new(&instrument.name);

        // Add articulations
        for (articulation_name, articulation) in &instrument.articulations {
            let mut group = SampleGroup::new(articulation_name);

            // Add velocity layers
            if self.velocity_layers {
                for layer in &articulation.velocity_layers {
                    let mut sample = Sample::new(&layer.sample_path);
                    sample.velocity_range = Some((layer.velocity_min, layer.velocity_max));
                    sample.gain_db = layer.gain_offset;

                    if let Some(ref loop_points) = layer.loop_points {
                        sample.loop_start = Some(loop_points.start_sample);
                        sample.loop_end = Some(loop_points.end_sample);
                        sample.loop_enabled = true;
                    }

                    group.samples.push(sample);
                }
            }

            // Add round-robin support
            if self.round_robin {
                for rr_group in &articulation.round_robin_groups {
                    group.round_robin_groups.push(rr_group.clone());
                }
            }

            dspreset.groups.push(group);
        }

        // Set global parameters
        dspreset.master_volume = instrument.global_settings.master_volume_db;
        dspreset.master_tune = instrument.global_settings.master_tune_cents;

        // Write DecentSampler XML
        self.write_decent_sampler_xml(&dspreset, output_path)?;

        Ok(())
    }

    /// Export to SFZ format with advanced features
    pub fn export_sfz(&self, instrument: &AdvancedInstrument, output_path: &Path) -> Result<()> {
        let mut sfz_content = String::new();

        // Global settings
        sfz_content.push_str("// Advanced SFZ instrument\n");
        sfz_content.push_str("// Generated by BatcherBird\n\n");

        // Control section
        sfz_content.push_str("<control>\n");
        sfz_content.push_str(&format!(
            "default_path={}\n",
            output_path.parent().unwrap_or(Path::new(".")).display()
        ));
        sfz_content.push('\n');

        // Global section
        sfz_content.push_str("<global>\n");
        sfz_content.push_str(&format!(
            "volume={:.1}\n",
            instrument.global_settings.master_volume_db
        ));
        sfz_content.push_str(&format!(
            "tune={:.0}\n",
            instrument.global_settings.master_tune_cents
        ));

        if let Some(polyphony) = instrument.global_settings.polyphony_limit {
            sfz_content.push_str(&format!("polyphony={}\n", polyphony));
        }

        if let Some(ref envelope) = instrument.global_settings.global_envelope {
            sfz_content.push_str(&format!("ampeg_attack={:.3}\n", envelope.attack_time));
            sfz_content.push_str(&format!("ampeg_decay={:.3}\n", envelope.decay_time));
            sfz_content.push_str(&format!(
                "ampeg_sustain={:.1}\n",
                envelope.sustain_level * 100.0
            ));
            sfz_content.push_str(&format!("ampeg_release={:.3}\n", envelope.release_time));
        }

        sfz_content.push('\n');

        // Add articulations
        for (articulation_name, articulation) in &instrument.articulations {
            sfz_content.push_str(&format!("// Articulation: {}\n", articulation_name));

            // Group section for this articulation
            sfz_content.push_str("<group>\n");

            // Add articulation trigger
            match &articulation.trigger {
                ArticulationTrigger::MidiCC {
                    controller,
                    value_range,
                } => {
                    sfz_content.push_str(&format!(
                        "locc{}={} hicc{}={}\n",
                        controller, value_range.0, controller, value_range.1
                    ));
                }
                ArticulationTrigger::Keyswitch { note, .. } => {
                    sfz_content.push_str(&format!("sw_last={}\n", note));
                }
                ArticulationTrigger::VelocityRange {
                    min_velocity,
                    max_velocity,
                } => {
                    sfz_content
                        .push_str(&format!("lovel={} hivel={}\n", min_velocity, max_velocity));
                }
                _ => {} // Other triggers not yet implemented
            }

            sfz_content.push('\n');

            // Add velocity layers
            for layer in &articulation.velocity_layers {
                sfz_content.push_str("<region>\n");
                sfz_content.push_str(&format!("sample={}\n", layer.sample_path.display()));
                sfz_content.push_str(&format!(
                    "lovel={} hivel={}\n",
                    layer.velocity_min, layer.velocity_max
                ));

                if layer.gain_offset != 0.0 {
                    sfz_content.push_str(&format!("volume={:.1}\n", layer.gain_offset));
                }

                if let Some(ref loop_points) = layer.loop_points {
                    sfz_content.push_str("loop_mode=loop_continuous\n");
                    sfz_content.push_str(&format!("loop_start={}\n", loop_points.start_sample));
                    sfz_content.push_str(&format!("loop_end={}\n", loop_points.end_sample));
                }

                sfz_content.push('\n');
            }
        }

        // Write SFZ file
        std::fs::write(output_path, sfz_content)
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to write SFZ file: {}", e)))?;

        Ok(())
    }

    fn write_decent_sampler_xml(
        &self,
        preset: &DecentSamplerPreset,
        output_path: &Path,
    ) -> Result<()> {
        // Simplified XML generation - in production would use proper XML library
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<DecentSampler>\n");
        xml.push_str("  <groups>\n");

        for group in &preset.groups {
            xml.push_str(&format!("    <group name=\"{}\">\n", group.name));

            for sample in &group.samples {
                xml.push_str("      <sample ");
                xml.push_str(&format!("path=\"{}\" ", sample.path.display()));

                if let Some((min_vel, max_vel)) = sample.velocity_range {
                    xml.push_str(&format!("loVel=\"{}\" hiVel=\"{}\" ", min_vel, max_vel));
                }

                if sample.gain_db != 0.0 {
                    xml.push_str(&format!("volume=\"{:.1}\" ", sample.gain_db));
                }

                if sample.loop_enabled {
                    if let (Some(start), Some(end)) = (sample.loop_start, sample.loop_end) {
                        xml.push_str(&format!("loopStart=\"{}\" loopEnd=\"{}\" ", start, end));
                    }
                }

                xml.push_str("/>\n");
            }

            xml.push_str("    </group>\n");
        }

        xml.push_str("  </groups>\n");
        xml.push_str("</DecentSampler>\n");

        std::fs::write(output_path, xml).map_err(|e| {
            BatcherbirdError::Audio(format!("Failed to write DecentSampler XML: {}", e))
        })?;

        Ok(())
    }
}

/// DecentSampler preset structure
#[derive(Debug)]
struct DecentSamplerPreset {
    #[allow(dead_code)] // Used in planned XML serialization output
    name: String,
    groups: Vec<SampleGroup>,
    master_volume: f32,
    master_tune: f32,
}

impl DecentSamplerPreset {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            groups: Vec::new(),
            master_volume: 0.0,
            master_tune: 0.0,
        }
    }
}

/// Sample group for DecentSampler
#[derive(Debug)]
struct SampleGroup {
    name: String,
    samples: Vec<Sample>,
    round_robin_groups: Vec<RoundRobinGroup>,
}

impl SampleGroup {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            samples: Vec::new(),
            round_robin_groups: Vec::new(),
        }
    }
}

/// Individual sample for DecentSampler
#[derive(Debug)]
struct Sample {
    path: PathBuf,
    velocity_range: Option<(u8, u8)>,
    gain_db: f32,
    loop_enabled: bool,
    loop_start: Option<u32>,
    loop_end: Option<u32>,
}

impl Sample {
    fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            velocity_range: None,
            gain_db: 0.0,
            loop_enabled: false,
            loop_start: None,
            loop_end: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_velocity_layer_generation() {
        let engine = VelocityLayerEngine::new(4);

        let samples = vec![
            SampleInfo::new(PathBuf::from("sample1.wav")),
            SampleInfo::new(PathBuf::from("sample2.wav")),
            SampleInfo::new(PathBuf::from("sample3.wav")),
            SampleInfo::new(PathBuf::from("sample4.wav")),
        ];

        let layers = engine.generate_velocity_layers(&samples).unwrap();

        assert_eq!(layers.len(), 4);

        // Check velocity ranges don't overlap inappropriately
        assert_eq!(layers[0].velocity_min, 0);
        assert_eq!(layers[0].velocity_max, 31);
        assert_eq!(layers[1].velocity_min, 31);
        assert_eq!(layers[1].velocity_max, 63);
        assert_eq!(layers[3].velocity_max, 127);

        // Check crossfade ranges
        for layer in &layers {
            assert!(layer.crossfade_range > 0);
        }
    }

    #[test]
    fn test_round_robin_groups() {
        let processor = RoundRobinProcessor::new(3);

        let samples = vec![
            SampleInfo::new(PathBuf::from("rr1.wav")),
            SampleInfo::new(PathBuf::from("rr2.wav")),
            SampleInfo::new(PathBuf::from("rr3.wav")),
            SampleInfo::new(PathBuf::from("rr4.wav")),
            SampleInfo::new(PathBuf::from("rr5.wav")),
        ];

        let groups = processor
            .create_round_robin_groups(&samples, "test")
            .unwrap();

        assert_eq!(groups.len(), 2); // 5 samples / 3 max = 2 groups
        assert_eq!(groups[0].sample_paths.len(), 3);
        assert_eq!(groups[1].sample_paths.len(), 2);
        assert_eq!(groups[0].group_id, "test_0");
        assert_eq!(groups[1].group_id, "test_1");
    }

    #[test]
    fn test_advanced_instrument_creation() {
        let mut instrument = AdvancedInstrument {
            name: "Test Synth".to_string(),
            category: "Synthesizer".to_string(),
            note_range: (36, 96), // C2 to C7
            default_articulation: "sustain".to_string(),
            articulations: HashMap::new(),
            global_settings: InstrumentSettings {
                master_volume_db: 0.0,
                master_tune_cents: 0.0,
                global_envelope: None,
                global_filter: None,
                polyphony_limit: Some(16),
                voice_allocation: VoiceAllocation::Oldest,
            },
            metadata: ProfessionalMetadata::new(),
        };

        // Add a basic articulation
        let articulation = Articulation {
            name: "sustain".to_string(),
            trigger: ArticulationTrigger::VelocityRange {
                min_velocity: 1,
                max_velocity: 127,
            },
            velocity_layers: Vec::new(),
            round_robin_groups: Vec::new(),
            release_samples: Vec::new(),
        };

        instrument
            .articulations
            .insert("sustain".to_string(), articulation);

        assert_eq!(instrument.articulations.len(), 1);
        assert!(instrument.articulations.contains_key("sustain"));
    }

    #[test]
    fn test_articulation_triggers() {
        let cc_trigger = ArticulationTrigger::MidiCC {
            controller: 64,
            value_range: (64, 127),
        };

        let keyswitch_trigger = ArticulationTrigger::Keyswitch {
            note: 36,
            channel: None,
        };

        let _velocity_trigger = ArticulationTrigger::VelocityRange {
            min_velocity: 100,
            max_velocity: 127,
        };

        // Just verify they can be created and serialized
        match cc_trigger {
            ArticulationTrigger::MidiCC {
                controller,
                value_range,
            } => {
                assert_eq!(controller, 64);
                assert_eq!(value_range, (64, 127));
            }
            _ => panic!("Wrong trigger type"),
        }

        match keyswitch_trigger {
            ArticulationTrigger::Keyswitch { note, channel } => {
                assert_eq!(note, 36);
                assert_eq!(channel, None);
            }
            _ => panic!("Wrong trigger type"),
        }
    }

    #[test]
    fn test_sample_info() {
        let mut sample = SampleInfo::new(PathBuf::from("test.wav"));

        sample
            .metadata
            .insert("gain_offset".to_string(), "-2.5".to_string());
        sample.peak_level_db = Some(-6.0);
        sample.rms_level_db = Some(-18.0);

        assert_eq!(sample.path, PathBuf::from("test.wav"));
        assert_eq!(
            sample.metadata.get("gain_offset"),
            Some(&"-2.5".to_string())
        );
        assert_eq!(sample.peak_level_db, Some(-6.0));
    }

    #[test]
    fn test_loop_points() {
        let loop_points = LoopPoints {
            start_sample: 1000,
            end_sample: 5000,
            loop_type: 0,
            crossfade_samples: 100,
            tune_cents: 0.0,
        };

        assert_eq!(loop_points.start_sample, 1000);
        assert_eq!(loop_points.end_sample, 5000);
        assert_eq!(loop_points.loop_type, 0);
        assert_eq!(loop_points.crossfade_samples, 100);
    }
}
