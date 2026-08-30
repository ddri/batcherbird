use crate::detection::DetectionConfig;
use crate::sampler::Sample;
use crate::{BatcherbirdError, Result};
use hound::{SampleFormat, WavSpec, WavWriter};
use std::fs;
use std::path::{Path, PathBuf};

/// Sanitize filename for cross-platform compatibility
pub(crate) fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

#[derive(Debug, Clone)]
pub struct ExportConfig {
    pub output_directory: PathBuf,
    pub naming_pattern: String,
    pub sample_format: AudioFormat,
    pub normalize: bool,
    pub fade_in_ms: f32,
    pub fade_out_ms: f32,
    pub apply_detection: bool,
    pub detection_config: DetectionConfig,
    // Decent Sampler metadata
    pub creator_name: Option<String>,
    pub instrument_description: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AudioFormat {
    Wav16Bit,
    Wav24Bit,
    Wav32BitFloat,
    DecentSampler, // Generates .dspreset XML file with WAV samples
    SFZ,           // Generates .sfz file with WAV samples
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            output_directory: PathBuf::from("./samples"),
            naming_pattern: "{note_name}_{note}_{velocity}.wav".to_string(),
            sample_format: AudioFormat::Wav24Bit,
            normalize: false,
            fade_in_ms: 0.0,
            fade_out_ms: 10.0,
            apply_detection: true, // Enable detection by default
            detection_config: DetectionConfig::default(),
            creator_name: None,
            instrument_description: None,
        }
    }
}

pub struct SampleExporter {
    config: ExportConfig,
}

/// Escape a string for safe interpolation into XML content
/// (element text, attribute values, and comments)
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Escape a string for safe interpolation into XML comment text.
///
/// Composes `escape_xml` (handles `&`, `<`, `>`, `"`, `'`) and then
/// collapses any `--` sequences (illegal inside XML comments per the spec)
/// into `- -`.
fn escape_xml_comment(s: &str) -> String {
    // `--` must not appear inside an XML comment body; collapse all runs.
    escape_xml(s).replace("--", "- -")
}

/// Flatten a string onto a single line for use in SFZ `//` comments
/// (embedded newlines would otherwise inject new directive lines)
fn single_line(s: &str) -> String {
    s.replace(['\r', '\n'], " ")
}

impl SampleExporter {
    pub fn new(config: ExportConfig) -> Result<Self> {
        // Create output directory if it doesn't exist
        if !config.output_directory.exists() {
            fs::create_dir_all(&config.output_directory).map_err(BatcherbirdError::Export)?;
        }

        Ok(Self { config })
    }

    pub fn export_sample(&self, sample: &Sample) -> Result<PathBuf> {
        let filename = self.generate_filename(sample);
        let filepath = self.config.output_directory.join(&filename);

        // Clone sample for processing (detection may modify audio data)
        let mut sample_copy = sample.clone();

        // Apply sample detection if enabled
        if self.config.apply_detection {
            // Attempt detection, but continue with export regardless of result
            if let Err(e) = sample_copy.apply_detection(self.config.detection_config.clone()) {
                tracing::warn!(
                    "Sample detection failed for note {} — exporting untrimmed audio: {}",
                    sample.note,
                    e
                );
            }
        }

        // Process audio data
        let mut audio_data = sample_copy.audio_data.clone();

        // Apply fades if configured
        if self.config.fade_in_ms > 0.0 || self.config.fade_out_ms > 0.0 {
            self.apply_fades(&mut audio_data, sample.sample_rate)?;
        }

        // Normalize if configured
        if self.config.normalize {
            self.normalize_audio(&mut audio_data)?;
        }

        // Handle different export formats
        match self.config.sample_format {
            AudioFormat::DecentSampler => {
                // For DecentSampler, we only write WAV files here
                // The .dspreset XML will be generated separately via export_samples()
                let wav_config = ExportConfig {
                    sample_format: AudioFormat::Wav24Bit, // Use 24-bit for DecentSampler compatibility
                    ..self.config.clone()
                };
                let temp_exporter = SampleExporter { config: wav_config };
                temp_exporter.write_wav_file(&filepath, &audio_data, sample)?;
            }
            AudioFormat::SFZ => {
                // For SFZ, we only write WAV files here
                // The .sfz file will be generated separately via export_samples()
                let wav_config = ExportConfig {
                    sample_format: AudioFormat::Wav24Bit, // Use 24-bit for good compatibility
                    ..self.config.clone()
                };
                let temp_exporter = SampleExporter { config: wav_config };
                temp_exporter.write_wav_file(&filepath, &audio_data, sample)?;
            }
            _ => {
                // Standard WAV export
                self.write_wav_file(&filepath, &audio_data, sample)?;
            }
        }

        Ok(filepath)
    }

    pub fn export_samples(&self, samples: &[Sample]) -> Result<Vec<PathBuf>> {
        let mut exported_files = Vec::new();

        for sample in samples.iter() {
            let filepath = self.export_sample(sample)?;
            exported_files.push(filepath);
        }

        // Generate .dspreset XML file for DecentSampler format
        if matches!(self.config.sample_format, AudioFormat::DecentSampler) {
            let dspreset_path = self.generate_dspreset_file(samples, &exported_files)?;
            exported_files.push(dspreset_path);
        }

        // Generate .sfz file for SFZ format
        if matches!(self.config.sample_format, AudioFormat::SFZ) {
            let sfz_path = self.generate_sfz_file(samples, &exported_files)?;
            exported_files.push(sfz_path);
        }

        Ok(exported_files)
    }

    fn generate_filename(&self, sample: &Sample) -> String {
        let note_name = Self::note_to_name(sample.note);
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");

        // Consistent "vel" prefix naming for all samples: C4_60_vel127.wav
        self.config
            .naming_pattern
            .replace("{note}", &sample.note.to_string())
            .replace("{note_name}", &note_name)
            .replace("{velocity}", &format!("vel{:03}", sample.velocity)) // vel064, vel127
            .replace("{timestamp}", &timestamp.to_string())
            .replace("{sample_rate}", &sample.sample_rate.to_string())
    }

    fn apply_fades(&self, audio_data: &mut [f32], sample_rate: u32) -> Result<()> {
        let fade_in_samples = ((self.config.fade_in_ms / 1000.0) * sample_rate as f32) as usize;
        let fade_out_samples = ((self.config.fade_out_ms / 1000.0) * sample_rate as f32) as usize;

        let len = audio_data.len();

        // Apply fade in
        if fade_in_samples > 0 && fade_in_samples < len {
            for (i, sample) in audio_data.iter_mut().enumerate().take(fade_in_samples.min(len)) {
                let fade_factor = i as f32 / fade_in_samples as f32;
                *sample *= fade_factor;
            }
        }

        // Apply fade out
        if fade_out_samples > 0 && fade_out_samples < len {
            let fade_start = len.saturating_sub(fade_out_samples);
            for (i, sample) in audio_data.iter_mut().enumerate().skip(fade_start).take(len - fade_start) {
                let fade_factor = (len - i) as f32 / fade_out_samples as f32;
                *sample *= fade_factor;
            }
        }

        Ok(())
    }

    fn normalize_audio(&self, audio_data: &mut [f32]) -> Result<()> {
        // Find peak amplitude
        let peak = audio_data
            .iter()
            .map(|&sample| sample.abs())
            .fold(0.0f32, f32::max);

        if peak > 0.0 && peak < 1.0 {
            let gain = 0.95 / peak; // Normalize to 95% to avoid clipping
            for sample in audio_data.iter_mut() {
                *sample *= gain;
            }
        }

        Ok(())
    }

    fn write_wav_file(&self, filepath: &Path, audio_data: &[f32], sample: &Sample) -> Result<()> {
        // Validate audio data first
        if audio_data.is_empty() {
            return Err(BatcherbirdError::Export(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Cannot export empty audio data",
            )));
        }

        let spec = match self.config.sample_format {
            AudioFormat::Wav16Bit => WavSpec {
                channels: sample.channels,
                sample_rate: sample.sample_rate,
                bits_per_sample: 16,
                sample_format: SampleFormat::Int,
            },
            AudioFormat::Wav24Bit => WavSpec {
                channels: sample.channels,
                sample_rate: sample.sample_rate,
                bits_per_sample: 24,
                sample_format: SampleFormat::Int,
            },
            AudioFormat::Wav32BitFloat => WavSpec {
                channels: sample.channels,
                sample_rate: sample.sample_rate,
                bits_per_sample: 32,
                sample_format: SampleFormat::Float,
            },
            AudioFormat::DecentSampler => {
                return Err(BatcherbirdError::Export(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "DecentSampler format should be handled separately, not in WAV writing",
                )));
            }
            AudioFormat::SFZ => {
                return Err(BatcherbirdError::Export(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "SFZ format should not reach write_wav_file - this is a logic error",
                )));
            }
        };

        // Create writer with explicit error handling
        let mut writer = WavWriter::create(filepath, spec)
            .map_err(|e| BatcherbirdError::Export(std::io::Error::other(e)))?;

        // Write samples
        match self.config.sample_format {
            AudioFormat::Wav16Bit => {
                for &sample in audio_data.iter() {
                    let sample_i16 = (sample * i16::MAX as f32) as i16;
                    writer
                        .write_sample(sample_i16)
                        .map_err(|e| BatcherbirdError::Export(std::io::Error::other(e)))?;
                }
            }
            AudioFormat::Wav24Bit => {
                for &sample in audio_data.iter() {
                    let sample_i32 = (sample * 8_388_607.0) as i32; // 24-bit max value
                    writer
                        .write_sample(sample_i32)
                        .map_err(|e| BatcherbirdError::Export(std::io::Error::other(e)))?;
                }
            }
            AudioFormat::Wav32BitFloat => {
                for &sample in audio_data.iter() {
                    writer
                        .write_sample(sample)
                        .map_err(|e| BatcherbirdError::Export(std::io::Error::other(e)))?;
                }
            }
            AudioFormat::DecentSampler => {
                return Err(BatcherbirdError::Export(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "DecentSampler format should not reach write_wav_file - this is a logic error",
                )));
            }
            AudioFormat::SFZ => {
                return Err(BatcherbirdError::Export(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "SFZ format should not reach write_wav_file - this is a logic error",
                )));
            }
        }

        // Finalize with explicit error handling
        writer
            .finalize()
            .map_err(|e| BatcherbirdError::Export(std::io::Error::other(e)))?;

        // Explicitly sync file to disk to prevent corruption during rapid batch exports
        if let Ok(file) = std::fs::File::open(filepath) {
            let _ = file.sync_all();
        }

        // Verify file was created
        std::fs::metadata(filepath).map_err(BatcherbirdError::Export)?;

        Ok(())
    }

    /// Convert a MIDI note number to its name (middle C = MIDI 60 = "C4")
    pub fn note_to_name(note: u8) -> String {
        let note_names = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        let octave = (note as i32 / 12) - 1;
        let note_name = note_names[(note % 12) as usize];
        format!("{}{}", note_name, octave)
    }

    /// Generate a Decent Sampler .dspreset XML file
    pub fn generate_dspreset_file(
        &self,
        samples: &[Sample],
        wav_files: &[PathBuf],
    ) -> Result<PathBuf> {
        use std::io::Write;

        // Create the .dspreset filename (use the sample name from config or default)
        let preset_name = self
            .config
            .naming_pattern
            .replace("{note}", "")
            .replace("{note_name}", "")
            .replace("{velocity}", "")
            .replace("_", "")
            .replace(".wav", "")
            .trim_matches('_')
            .to_string();

        let preset_name = if preset_name.is_empty() {
            "Batcherbird_Instrument".to_string()
        } else {
            sanitize_filename(&preset_name)
        };

        let dspreset_filename = format!("{}.dspreset", preset_name);
        let dspreset_path = self.config.output_directory.join(&dspreset_filename);

        // Group samples by velocity for layering
        let mut velocity_groups = std::collections::HashMap::new();
        for (i, sample) in samples.iter().enumerate() {
            if i < wav_files.len() {
                velocity_groups
                    .entry(sample.velocity)
                    .or_insert_with(Vec::new)
                    .push((sample, &wav_files[i]));
            }
        }

        // Generate XML content
        let xml_content = self.generate_dspreset_xml(&preset_name, &velocity_groups)?;

        // Write XML file
        let mut file = std::fs::File::create(&dspreset_path).map_err(BatcherbirdError::Export)?;

        file.write_all(xml_content.as_bytes())
            .map_err(BatcherbirdError::Export)?;

        Ok(dspreset_path)
    }

    /// Generate the XML content for a Decent Sampler .dspreset file
    fn generate_dspreset_xml(
        &self,
        preset_name: &str,
        velocity_groups: &std::collections::HashMap<u8, Vec<(&Sample, &PathBuf)>>,
    ) -> Result<String> {
        let mut xml = String::new();

        // XML Declaration and root element following official template
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str(&format!(
            "<!-- {} - Generated by Batcherbird -->\n",
            escape_xml_comment(preset_name)
        ));

        // Add creator and description in comment if provided
        if let Some(ref creator) = self.config.creator_name {
            xml.push_str(&format!(
                "<!-- Creator: {} -->\n",
                escape_xml_comment(creator)
            ));
        }
        if let Some(ref description) = self.config.instrument_description {
            xml.push_str(&format!(
                "<!-- Description: {} -->\n",
                escape_xml_comment(description)
            ));
        }

        xml.push_str("<DecentSampler>\n");

        // UI Section following official template structure
        xml.push_str("  <ui width=\"812\" height=\"375\">\n");
        xml.push_str("    <tab name=\"main\">\n");
        xml.push_str("      <labeled-knob x=\"50\" y=\"50\" label=\"Volume\" type=\"float\" minValue=\"0\" maxValue=\"1\" value=\"0.7\">\n");
        xml.push_str(
            "        <binding type=\"amp\" level=\"instrument\" parameter=\"VOLUME\" />\n",
        );
        xml.push_str("      </labeled-knob>\n");
        xml.push_str("    </tab>\n");
        xml.push_str("  </ui>\n");

        // Groups Section following official DecentSampler specification
        xml.push_str("  <groups>\n");

        let mut sorted_velocities: Vec<_> = velocity_groups.keys().collect();
        sorted_velocities.sort();

        for (group_index, &velocity) in sorted_velocities.iter().enumerate() {
            if let Some(samples) = velocity_groups.get(velocity) {
                let (lo_vel, hi_vel) = if sorted_velocities.len() == 1 {
                    (1, 127)
                } else {
                    let vel_range = 127.0 / sorted_velocities.len() as f32;
                    let lo = ((group_index as f32 * vel_range) as u8).max(1);
                    let hi = (((group_index + 1) as f32 * vel_range) as u8).min(127);
                    (lo, hi)
                };

                xml.push_str(&format!(
                    "    <group loVel=\"{}\" hiVel=\"{}\">\n",
                    lo_vel, hi_vel
                ));

                for (sample, wav_file) in samples {
                    let filename = wav_file
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("sample.wav");

                    let mut sample_tag = format!(
                        "      <sample path=\"{}\" loNote=\"{}\" hiNote=\"{}\" rootNote=\"{}\"",
                        escape_xml(filename),
                        sample.note,
                        sample.note,
                        sample.note
                    );

                    if self.config.apply_detection {
                        let detector = crate::loop_detection::LoopDetector::new(
                            crate::loop_detection::LoopDetectionConfig::default(),
                        );
                        let res = detector.detect_loop_points(&sample.audio_data, sample.sample_rate);
                        if let Some(cand) = res.best_candidate {
                            sample_tag.push_str(&format!(
                                " loopEnabled=\"true\" loopStart=\"{}\" loopEnd=\"{}\"",
                                cand.start_sample, cand.end_sample
                            ));
                        }
                    }

                    sample_tag.push_str(" />\n");
                    xml.push_str(&sample_tag);
                }

                xml.push_str("    </group>\n");
            }
        }

        xml.push_str("  </groups>\n");

        // Close root element
        xml.push_str("</DecentSampler>\n");

        Ok(xml)
    }

    /// Generate an SFZ .sfz file
    pub fn generate_sfz_file(&self, samples: &[Sample], wav_files: &[PathBuf]) -> Result<PathBuf> {
        use std::io::Write;

        // Create the .sfz filename (use the sample name from config or default)
        let preset_name = self
            .config
            .naming_pattern
            .replace("{note}", "")
            .replace("{note_name}", "")
            .replace("{velocity}", "")
            .replace("_", "")
            .replace(".wav", "")
            .trim_matches('_')
            .to_string();

        let preset_name = if preset_name.is_empty() {
            "Batcherbird_Instrument".to_string()
        } else {
            sanitize_filename(&preset_name)
        };

        let sfz_filename = format!("{}.sfz", preset_name);
        let sfz_path = self.config.output_directory.join(&sfz_filename);

        // Group samples by velocity for layering
        let mut velocity_groups = std::collections::HashMap::new();
        for (i, sample) in samples.iter().enumerate() {
            if i < wav_files.len() {
                velocity_groups
                    .entry(sample.velocity)
                    .or_insert_with(Vec::new)
                    .push((sample, &wav_files[i]));
            }
        }

        // Generate SFZ content
        let sfz_content = self.generate_sfz_content(&preset_name, &velocity_groups)?;

        // Write SFZ file
        let mut file = std::fs::File::create(&sfz_path).map_err(BatcherbirdError::Export)?;

        file.write_all(sfz_content.as_bytes())
            .map_err(BatcherbirdError::Export)?;

        Ok(sfz_path)
    }

    /// Generate the SFZ content
    fn generate_sfz_content(
        &self,
        preset_name: &str,
        velocity_groups: &std::collections::HashMap<u8, Vec<(&Sample, &PathBuf)>>,
    ) -> Result<String> {
        let mut sfz = String::new();

        // SFZ Header with comments (values flattened to one line so they
        // cannot inject SFZ directives)
        sfz.push_str(&format!(
            "// {} - Generated by Batcherbird\n",
            single_line(preset_name)
        ));

        // Add creator and description in comments if provided
        if let Some(ref creator) = self.config.creator_name {
            sfz.push_str(&format!("// Creator: {}\n", single_line(creator)));
        }
        if let Some(ref description) = self.config.instrument_description {
            sfz.push_str(&format!("// Description: {}\n", single_line(description)));
        }

        sfz.push('\n');

        // Control section - path settings
        sfz.push_str("<control>\n");
        sfz.push_str("default_path=samples/\n");
        sfz.push('\n');

        // Global section - overall settings
        sfz.push_str("<global>\n");
        sfz.push_str("ampeg_release=0.5\n");
        sfz.push('\n');

        // Sort velocity groups for consistent output
        let mut sorted_velocities: Vec<_> = velocity_groups.keys().collect();
        sorted_velocities.sort();

        // Generate regions for each velocity layer
        for (group_index, &velocity) in sorted_velocities.iter().enumerate() {
            if let Some(samples) = velocity_groups.get(velocity) {
                // Group header for this velocity layer
                if sorted_velocities.len() > 1 {
                    sfz.push_str("<group>\n");

                    // Calculate velocity range for this layer
                    let (lo_vel, hi_vel) = if sorted_velocities.len() == 1 {
                        (1, 127) // Single velocity covers full range
                    } else {
                        // Distribute velocity ranges among layers
                        let vel_range = 127.0 / sorted_velocities.len() as f32;
                        let lo = ((group_index as f32 * vel_range) as u8).max(1);
                        let hi = (((group_index + 1) as f32 * vel_range) as u8).min(127);
                        (lo, hi)
                    };

                    sfz.push_str(&format!("lovel={}\n", lo_vel));
                    sfz.push_str(&format!("hivel={}\n", hi_vel));
                    sfz.push('\n');
                }

                // Add regions (samples) for this velocity group
                for (sample, wav_file) in samples {
                    let filename = wav_file
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("sample.wav");

                    sfz.push_str("<region>\n");
                    sfz.push_str(&format!("sample={}\n", filename));
                    sfz.push_str(&format!("key={}\n", sample.note));

                    // Add velocity range for single-layer instruments
                    if sorted_velocities.len() == 1 {
                        sfz.push_str("lovel=1\n");
                        sfz.push_str("hivel=127\n");
                    }

                    if self.config.apply_detection {
                        let detector = crate::loop_detection::LoopDetector::new(
                            crate::loop_detection::LoopDetectionConfig::default(),
                        );
                        let res = detector.detect_loop_points(&sample.audio_data, sample.sample_rate);
                        if let Some(cand) = res.best_candidate {
                            sfz.push_str("loop_mode=loop_continuous\n");
                            sfz.push_str(&format!("loop_start={}\n", cand.start_sample));
                            sfz.push_str(&format!("loop_end={}\n", cand.end_sample));
                        }
                    }

                    sfz.push('\n');
                }
            }
        }

        Ok(sfz)
    }

    pub fn get_export_info(&self) -> String {
        format!(
            "Export Configuration:\n  Directory: {}\n  Format: {:?}\n  Normalize: {}\n  Fade out: {}ms",
            self.config.output_directory.display(),
            self.config.sample_format,
            self.config.normalize,
            self.config.fade_out_ms
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("My Project"), "My Project");
        assert_eq!(
            sanitize_filename("Bad/Name\\With:*?\"<>|"),
            "Bad_Name_With_______"
        );
        assert_eq!(sanitize_filename(" \t Spaces \n "), "_ Spaces _");
    }

    #[test]
    fn test_note_to_name() {
        assert_eq!(SampleExporter::note_to_name(0), "C-1");
        assert_eq!(SampleExporter::note_to_name(21), "A0");
        assert_eq!(SampleExporter::note_to_name(60), "C4");
        assert_eq!(SampleExporter::note_to_name(127), "G9");
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(
            escape_xml("a & b < c > d \" e ' f"),
            "a &amp; b &lt; c &gt; d &quot; e &apos; f"
        );
        // Ampersand must be escaped first (no double-escaping)
        assert_eq!(escape_xml("&lt;"), "&amp;lt;");
    }

    fn exporter_with_metadata(creator: &str, description: &str) -> SampleExporter {
        SampleExporter {
            config: ExportConfig {
                creator_name: Some(creator.to_string()),
                instrument_description: Some(description.to_string()),
                ..ExportConfig::default()
            },
        }
    }

    #[test]
    fn test_dspreset_xml_escapes_metadata() {
        let exporter = exporter_with_metadata("--> <evil/>", "desc");
        let groups = std::collections::HashMap::new();
        let xml = exporter.generate_dspreset_xml("Test", &groups).unwrap();

        assert!(!xml.contains("<evil/>"), "raw markup leaked into XML:\n{}", xml);
        assert!(xml.contains("&lt;evil/&gt;"), "metadata not escaped:\n{}", xml);
    }

    #[test]
    fn test_dspreset_xml_no_double_dash_in_comments() {
        // "--" is illegal inside XML comments; inputs that produce it after
        // escape_xml must be further sanitised.
        let exporter = exporter_with_metadata(
            "Dave -- Synth Pack",
            "Version 2 -- updated --> see notes",
        );
        let groups = std::collections::HashMap::new();
        let xml = exporter
            .generate_dspreset_xml("Preset -- Name", &groups)
            .unwrap();

        // Strip the opening "<!--" and closing "-->" delimiters before scanning.
        // We do this by splitting on comment boundaries and checking only the
        // interior text of each comment.
        for comment_body in xml.split("<!--").skip(1) {
            let interior = comment_body.split("-->").next().unwrap_or("");
            assert!(
                !interior.contains("--"),
                "illegal `--` found inside XML comment body: {:?}\nFull XML:\n{}",
                interior,
                xml
            );
        }
    }

    #[test]
    fn test_sfz_comment_stays_on_one_line() {
        let exporter =
            exporter_with_metadata("Creator", "line one\n<region> sample=evil.wav");
        let groups = std::collections::HashMap::new();
        let sfz = exporter.generate_sfz_content("Test", &groups).unwrap();

        assert!(
            !sfz.contains("\n<region> sample=evil.wav"),
            "newline in metadata injected an SFZ directive:\n{}",
            sfz
        );
        assert!(sfz.contains("// Description: line one <region> sample=evil.wav"));
    }
}
