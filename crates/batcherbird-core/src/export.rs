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
    /// Whether to embed standard RIFF metadata (smpl, bext, LIST-INFO) into exported WAV files
    pub embed_metadata: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioFormat {
    Wav16Bit,
    Wav24Bit,
    Wav32BitFloat,
    DecentSampler,        // Generates .dspreset XML file with WAV samples
    SFZ,                  // Generates .sfz file with WAV samples
    DecentSamplerAndSfz,  // Generates both .dspreset and .sfz files sharing WAV samples
    All,                  // Complete package: DecentSampler, SFZ, and 24-bit WAV samples
}

impl AudioFormat {
    /// Canonical directory name used for structured folder exports.
    pub fn directory_name(&self) -> &'static str {
        match self {
            AudioFormat::Wav16Bit => "WAV_16Bit",
            AudioFormat::Wav24Bit => "WAV_24Bit",
            AudioFormat::Wav32BitFloat => "WAV_32BitFloat",
            AudioFormat::DecentSampler => "DecentSampler",
            AudioFormat::SFZ => "SFZ",
            AudioFormat::DecentSamplerAndSfz => "DecentSampler_SFZ",
            AudioFormat::All => "All_Formats",
        }
    }
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
            embed_metadata: true,
        }
    }
}

/// Metadata parsed from standard RIFF chunks (`smpl`, `bext`, `LIST-INFO`) in a WAV file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WavMetadata {
    pub sample_period_ns: Option<u32>,
    pub midi_unity_note: Option<u8>,
    pub loop_points: Option<(u32, u32)>,
    pub bext_description: Option<String>,
    pub bext_originator: Option<String>,
    pub bext_origination_date: Option<String>,
    pub bext_origination_time: Option<String>,
    pub info_title: Option<String>,
    pub info_artist: Option<String>,
    pub info_date: Option<String>,
    pub info_software: Option<String>,
    pub info_comment: Option<String>,
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
            AudioFormat::DecentSampler
            | AudioFormat::SFZ
            | AudioFormat::DecentSamplerAndSfz
            | AudioFormat::All => {
                // For sampler presets, we write 24-bit WAV files here.
                // Preset files (.dspreset and/or .sfz) will be generated via export_samples()
                let wav_config = ExportConfig {
                    sample_format: AudioFormat::Wav24Bit, // 24-bit standard for sampler compatibility
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

        // Generate .dspreset XML file for DecentSampler format (or combined formats)
        if matches!(
            self.config.sample_format,
            AudioFormat::DecentSampler | AudioFormat::DecentSamplerAndSfz | AudioFormat::All
        ) {
            let dspreset_path = self.generate_dspreset_file(samples, &exported_files)?;
            exported_files.push(dspreset_path);
        }

        // Generate .sfz file for SFZ format (or combined formats)
        if matches!(
            self.config.sample_format,
            AudioFormat::SFZ | AudioFormat::DecentSamplerAndSfz | AudioFormat::All
        ) {
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
            AudioFormat::DecentSampler
            | AudioFormat::SFZ
            | AudioFormat::DecentSamplerAndSfz
            | AudioFormat::All => {
                return Err(BatcherbirdError::Export(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Sampler preset formats should be handled via export_sample/export_samples, not in direct write_wav_file",
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
            AudioFormat::DecentSampler
            | AudioFormat::SFZ
            | AudioFormat::DecentSamplerAndSfz
            | AudioFormat::All => {
                return Err(BatcherbirdError::Export(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Sampler preset formats should not reach write_wav_file sample writing",
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

        // If metadata embedding is enabled, inject bext, LIST-INFO, and smpl chunks
        if self.config.embed_metadata {
            let mut chunks = Vec::new();

            // 1. Loop detection for smpl chunk (if detection is active)
            let loop_points = if self.config.apply_detection {
                let detector_cfg = crate::loop_detection::LoopDetectionConfig::default();
                let detector = crate::loop_detection::LoopDetector::new(detector_cfg);
                let res = detector.detect_loop_points(audio_data, sample.sample_rate);
                res.best_candidate.map(|c| (c.start_sample as u32, c.end_sample as u32))
            } else {
                None
            };

            // 2. smpl chunk (MIDI root note & loop points)
            chunks.push(Self::build_smpl_chunk(
                sample.sample_rate,
                sample.note,
                loop_points,
            ));

            // 3. bext chunk (Broadcast Wave Format v1)
            let now = chrono::Utc::now();
            let desc = self
                .config
                .instrument_description
                .as_deref()
                .or(Some("Synthesizer Sample"));
            chunks.push(Self::build_bext_chunk(
                self.config.creator_name.as_deref(),
                desc,
                &now,
            ));

            // 4. LIST-INFO chunk
            let note_name = Self::note_to_name(sample.note);
            let title = format!("{} (Note {})", note_name, sample.note);
            let date_str = now.format("%Y-%m-%d").to_string();
            let comment = format!(
                "Velocity: {}, Sample Rate: {} Hz",
                sample.velocity, sample.sample_rate
            );
            chunks.push(Self::build_info_chunk(
                &title,
                self.config.creator_name.as_deref(),
                &date_str,
                "Batcherbird",
                Some(&comment),
            ));

            Self::append_riff_chunks(filepath, &chunks)?;
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
                        let detector_cfg = crate::loop_detection::LoopDetectionConfig::default();
                        let crossfade_sec = detector_cfg.crossfade_ms / 1000.0;
                        let detector = crate::loop_detection::LoopDetector::new(detector_cfg);
                        let res = detector.detect_loop_points(&sample.audio_data, sample.sample_rate);
                        if let Some(cand) = res.best_candidate {
                            sample_tag.push_str(&format!(
                                " loopEnabled=\"true\" loopStart=\"{}\" loopEnd=\"{}\" loopCrossfade=\"{:.3}\"",
                                cand.start_sample, cand.end_sample, crossfade_sec
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
                        let detector_cfg = crate::loop_detection::LoopDetectionConfig::default();
                        let crossfade_sec = detector_cfg.crossfade_ms / 1000.0;
                        let detector = crate::loop_detection::LoopDetector::new(detector_cfg);
                        let res = detector.detect_loop_points(&sample.audio_data, sample.sample_rate);
                        if let Some(cand) = res.best_candidate {
                            sfz.push_str("loop_mode=loop_continuous\n");
                            sfz.push_str(&format!("loop_start={}\n", cand.start_sample));
                            sfz.push_str(&format!("loop_end={}\n", cand.end_sample));
                            sfz.push_str(&format!("loop_crossfade={:.3}\n", crossfade_sec));
                        }
                    }

                    sfz.push('\n');
                }
            }
        }

        Ok(sfz)
    }

    /// Build a standard RIFF `smpl` chunk.
    ///
    /// Encodes sample period in nanoseconds, MIDI unity note (root key), and
    /// forward sustain loop points (when provided).
    pub fn build_smpl_chunk(
        sample_rate: u32,
        midi_note: u8,
        loop_points: Option<(u32, u32)>,
    ) -> Vec<u8> {
        let has_loop = loop_points.is_some();
        let payload_size: u32 = if has_loop { 36 + 24 } else { 36 };
        let mut chunk = Vec::with_capacity(8 + payload_size as usize);

        // Header: FourCC + payload size
        chunk.extend_from_slice(b"smpl");
        chunk.extend_from_slice(&payload_size.to_le_bytes());

        // Payload
        let sample_period_ns = if sample_rate > 0 {
            (1_000_000_000.0 / sample_rate as f64).round() as u32
        } else {
            0
        };

        chunk.extend_from_slice(&0u32.to_le_bytes()); // manufacturer: 0
        chunk.extend_from_slice(&0u32.to_le_bytes()); // product: 0
        chunk.extend_from_slice(&sample_period_ns.to_le_bytes());
        chunk.extend_from_slice(&(midi_note as u32).to_le_bytes());
        chunk.extend_from_slice(&0u32.to_le_bytes()); // midi_pitch_fraction: 0
        chunk.extend_from_slice(&0u32.to_le_bytes()); // smpte_format: 0
        chunk.extend_from_slice(&0u32.to_le_bytes()); // smpte_offset: 0
        chunk.extend_from_slice(&(if has_loop { 1u32 } else { 0u32 }).to_le_bytes()); // num_sample_loops
        chunk.extend_from_slice(&0u32.to_le_bytes()); // sampler_data: 0

        if let Some((start, end)) = loop_points {
            chunk.extend_from_slice(&0u32.to_le_bytes()); // cue_point_id: 0
            chunk.extend_from_slice(&0u32.to_le_bytes()); // type: 0 = forward loop
            chunk.extend_from_slice(&start.to_le_bytes()); // start sample index
            chunk.extend_from_slice(&end.to_le_bytes()); // end sample index
            chunk.extend_from_slice(&0u32.to_le_bytes()); // fraction: 0
            chunk.extend_from_slice(&0u32.to_le_bytes()); // play_count: 0 = infinite sustain loop
        }

        chunk
    }

    /// Build a standard Broadcast Wave Format (BWF v1) `bext` chunk (EBU Tech 3285, 602 bytes payload).
    pub fn build_bext_chunk(
        originator: Option<&str>,
        description: Option<&str>,
        timestamp: &chrono::DateTime<chrono::Utc>,
    ) -> Vec<u8> {
        let payload_size: u32 = 602;
        let mut chunk = Vec::with_capacity(8 + payload_size as usize);

        chunk.extend_from_slice(b"bext");
        chunk.extend_from_slice(&payload_size.to_le_bytes());

        // 1. Description: [u8; 256] ASCII (null-padded)
        let mut desc_buf = [0u8; 256];
        if let Some(desc) = description {
            let bytes = desc.as_bytes();
            let len = bytes.len().min(255);
            desc_buf[..len].copy_from_slice(&bytes[..len]);
        }
        chunk.extend_from_slice(&desc_buf);

        // 2. Originator: [u8; 32] ASCII (null-padded)
        let mut orig_buf = [0u8; 32];
        let orig_str = originator.unwrap_or("Batcherbird");
        let orig_bytes = orig_str.as_bytes();
        let orig_len = orig_bytes.len().min(31);
        orig_buf[..orig_len].copy_from_slice(&orig_bytes[..orig_len]);
        chunk.extend_from_slice(&orig_buf);

        // 3. Originator Reference: [u8; 32] ASCII (null-padded)
        let mut ref_buf = [0u8; 32];
        let ref_str = format!("BB-{}", timestamp.format("%Y%m%d%H%M%S"));
        let ref_bytes = ref_str.as_bytes();
        let ref_len = ref_bytes.len().min(31);
        ref_buf[..ref_len].copy_from_slice(&ref_bytes[..ref_len]);
        chunk.extend_from_slice(&ref_buf);

        // 4. Origination Date: [u8; 10] ASCII (YYYY-MM-DD)
        let mut date_buf = [0u8; 10];
        let date_str = timestamp.format("%Y-%m-%d").to_string();
        let date_bytes = date_str.as_bytes();
        let date_len = date_bytes.len().min(10);
        date_buf[..date_len].copy_from_slice(&date_bytes[..date_len]);
        chunk.extend_from_slice(&date_buf);

        // 5. Origination Time: [u8; 8] ASCII (HH:MM:SS)
        let mut time_buf = [0u8; 8];
        let time_str = timestamp.format("%H:%M:%S").to_string();
        let time_bytes = time_str.as_bytes();
        let time_len = time_bytes.len().min(8);
        time_buf[..time_len].copy_from_slice(&time_bytes[..time_len]);
        chunk.extend_from_slice(&time_buf);

        // 6. Time Reference: u64 (low u32, high u32) = 0
        chunk.extend_from_slice(&0u32.to_le_bytes()); // low
        chunk.extend_from_slice(&0u32.to_le_bytes()); // high

        // 7. Version: u16 = 1 (BWF v1)
        chunk.extend_from_slice(&1u16.to_le_bytes());

        // 8. SMPTE UMID: [u8; 64] = 0
        chunk.extend_from_slice(&[0u8; 64]);

        // 9. Loudness values: 5 x u16 = 0
        chunk.extend_from_slice(&0u16.to_le_bytes()); // loudness_value
        chunk.extend_from_slice(&0u16.to_le_bytes()); // loudness_range
        chunk.extend_from_slice(&0u16.to_le_bytes()); // max_true_peak
        chunk.extend_from_slice(&0u16.to_le_bytes()); // max_momentary_loudness
        chunk.extend_from_slice(&0u16.to_le_bytes()); // max_short_term_loudness

        // 10. Reserved: [u8; 180] = 0
        chunk.extend_from_slice(&[0u8; 180]);

        chunk
    }

    /// Helper to push a null-terminated INFO subchunk with 2-byte RIFF alignment.
    fn push_info_subchunk(buf: &mut Vec<u8>, id: &[u8; 4], text: &str) {
        buf.extend_from_slice(id);
        let mut bytes = text.as_bytes().to_vec();
        bytes.push(0); // null terminator
        let len = bytes.len() as u32;
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(&bytes);
        if !len.is_multiple_of(2) {
            buf.push(0); // RIFF 2-byte alignment padding byte
        }
    }

    /// Build a standard RIFF `LIST` chunk containing `INFO` subchunks.
    pub fn build_info_chunk(
        title: &str,
        artist: Option<&str>,
        date: &str,
        software: &str,
        comment: Option<&str>,
    ) -> Vec<u8> {
        let mut info_body = Vec::new();
        info_body.extend_from_slice(b"INFO");

        Self::push_info_subchunk(&mut info_body, b"INAM", title);
        if let Some(art) = artist {
            Self::push_info_subchunk(&mut info_body, b"IART", art);
        }
        Self::push_info_subchunk(&mut info_body, b"ICRD", date);
        Self::push_info_subchunk(&mut info_body, b"ISFT", software);
        if let Some(cmt) = comment {
            Self::push_info_subchunk(&mut info_body, b"ICMT", cmt);
        }

        let mut chunk = Vec::new();
        chunk.extend_from_slice(b"LIST");
        let list_len = info_body.len() as u32;
        chunk.extend_from_slice(&list_len.to_le_bytes());
        chunk.extend_from_slice(&info_body);
        if !list_len.is_multiple_of(2) {
            chunk.push(0); // RIFF pad byte
        }

        chunk
    }

    /// Append serialized RIFF chunks to a finalized WAV file and update the RIFF size header at offset 4.
    pub fn append_riff_chunks(filepath: &Path, chunks: &[Vec<u8>]) -> Result<()> {
        if chunks.is_empty() {
            return Ok(());
        }

        use std::io::{Read, Seek, SeekFrom, Write};

        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(filepath)
            .map_err(BatcherbirdError::Export)?;

        // Validate RIFF / WAVE header
        let mut header = [0u8; 12];
        file.read_exact(&mut header)
            .map_err(BatcherbirdError::Export)?;

        if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
            return Err(BatcherbirdError::Export(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Target file is not a valid RIFF/WAVE file",
            )));
        }

        // Seek to end
        let original_len = file
            .seek(SeekFrom::End(0))
            .map_err(BatcherbirdError::Export)?;

        // Ensure 2-byte chunk alignment before writing new chunks
        if !original_len.is_multiple_of(2) {
            file.write_all(&[0u8])
                .map_err(BatcherbirdError::Export)?;
        }

        for chunk in chunks {
            file.write_all(chunk)
                .map_err(BatcherbirdError::Export)?;
        }

        // Get final length and update RIFF size at offset 4
        let final_len = file
            .seek(SeekFrom::End(0))
            .map_err(BatcherbirdError::Export)?;

        if final_len < 8 {
            return Err(BatcherbirdError::Export(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "File too small after writing chunks",
            )));
        }

        let riff_size = (final_len - 8) as u32;
        file.seek(SeekFrom::Start(4))
            .map_err(BatcherbirdError::Export)?;
        file.write_all(&riff_size.to_le_bytes())
            .map_err(BatcherbirdError::Export)?;

        file.flush().map_err(BatcherbirdError::Export)?;
        file.sync_all().map_err(BatcherbirdError::Export)?;

        Ok(())
    }

    pub fn get_export_info(&self) -> String {
        format!(
            "Export Configuration:\n  Directory: {}\n  Format: {:?}\n  Normalize: {}\n  Fade out: {}ms\n  Embed RIFF Metadata: {}",
            self.config.output_directory.display(),
            self.config.sample_format,
            self.config.normalize,
            self.config.fade_out_ms,
            self.config.embed_metadata
        )
    }
}

/// Configuration for exporting samples across multiple formats simultaneously.
#[derive(Debug, Clone)]
pub struct BatchExportConfig {
    pub output_directory: PathBuf,
    pub naming_pattern: String,
    pub formats: Vec<AudioFormat>,
    /// If true, creates dedicated subdirectories for each format (e.g. `DecentSampler/`, `SFZ/`, `WAV_24Bit/`).
    /// If false, exports files into the shared output directory (reusing WAV files between presets).
    pub organize_subdirectories: bool,
    pub normalize: bool,
    pub fade_in_ms: f32,
    pub fade_out_ms: f32,
    pub apply_detection: bool,
    pub detection_config: DetectionConfig,
    pub creator_name: Option<String>,
    pub instrument_description: Option<String>,
    pub embed_metadata: bool,
}

impl Default for BatchExportConfig {
    fn default() -> Self {
        Self {
            output_directory: PathBuf::from("./samples"),
            naming_pattern: "{note_name}_{note}_{velocity}.wav".to_string(),
            formats: vec![
                AudioFormat::Wav24Bit,
                AudioFormat::DecentSampler,
                AudioFormat::SFZ,
            ],
            organize_subdirectories: false,
            normalize: false,
            fade_in_ms: 0.0,
            fade_out_ms: 10.0,
            apply_detection: true,
            detection_config: DetectionConfig::default(),
            creator_name: None,
            instrument_description: None,
            embed_metadata: true,
        }
    }
}

/// Results of a batch multi-format export operation.
#[derive(Debug, Clone, Default)]
pub struct BatchExportResult {
    /// Exported file paths grouped by format
    pub files_by_format: Vec<(AudioFormat, Vec<PathBuf>)>,
    /// All unique generated file paths
    pub all_files: Vec<PathBuf>,
}

/// Multi-format batch exporter for generating multiple sampler formats and audio depths simultaneously.
pub struct BatchExporter {
    config: BatchExportConfig,
}

impl BatchExporter {
    pub fn new(config: BatchExportConfig) -> Result<Self> {
        if !config.output_directory.exists() {
            fs::create_dir_all(&config.output_directory).map_err(BatcherbirdError::Export)?;
        }
        Ok(Self { config })
    }

    pub fn export_samples(&self, samples: &[Sample]) -> Result<BatchExportResult> {
        if samples.is_empty() {
            return Ok(BatchExportResult::default());
        }

        let mut files_by_format = Vec::new();
        let mut all_files = Vec::new();

        if self.config.organize_subdirectories {
            for format in &self.config.formats {
                let sub_dir = self.config.output_directory.join(format.directory_name());
                if !sub_dir.exists() {
                    fs::create_dir_all(&sub_dir).map_err(BatcherbirdError::Export)?;
                }

                let sub_config = ExportConfig {
                    output_directory: sub_dir,
                    naming_pattern: self.config.naming_pattern.clone(),
                    sample_format: format.clone(),
                    normalize: self.config.normalize,
                    fade_in_ms: self.config.fade_in_ms,
                    fade_out_ms: self.config.fade_out_ms,
                    apply_detection: self.config.apply_detection,
                    detection_config: self.config.detection_config.clone(),
                    creator_name: self.config.creator_name.clone(),
                    instrument_description: self.config.instrument_description.clone(),
                    embed_metadata: self.config.embed_metadata,
                };

                let exporter = SampleExporter::new(sub_config)?;
                let files = exporter.export_samples(samples)?;
                for f in &files {
                    if !all_files.contains(f) {
                        all_files.push(f.clone());
                    }
                }
                files_by_format.push((format.clone(), files));
            }
        } else {
            let has_ds = self.config.formats.contains(&AudioFormat::DecentSampler)
                || self.config.formats.contains(&AudioFormat::DecentSamplerAndSfz)
                || self.config.formats.contains(&AudioFormat::All);
            let has_sfz = self.config.formats.contains(&AudioFormat::SFZ)
                || self.config.formats.contains(&AudioFormat::DecentSamplerAndSfz)
                || self.config.formats.contains(&AudioFormat::All);
            let has_wav24 = self.config.formats.contains(&AudioFormat::Wav24Bit);

            if has_ds && has_sfz {
                let shared_format = AudioFormat::DecentSamplerAndSfz;
                let shared_config = ExportConfig {
                    output_directory: self.config.output_directory.clone(),
                    naming_pattern: self.config.naming_pattern.clone(),
                    sample_format: shared_format.clone(),
                    normalize: self.config.normalize,
                    fade_in_ms: self.config.fade_in_ms,
                    fade_out_ms: self.config.fade_out_ms,
                    apply_detection: self.config.apply_detection,
                    detection_config: self.config.detection_config.clone(),
                    creator_name: self.config.creator_name.clone(),
                    instrument_description: self.config.instrument_description.clone(),
                    embed_metadata: self.config.embed_metadata,
                };

                let exporter = SampleExporter::new(shared_config)?;
                let files = exporter.export_samples(samples)?;
                for f in &files {
                    if !all_files.contains(f) {
                        all_files.push(f.clone());
                    }
                }
                files_by_format.push((shared_format, files));

                for format in &self.config.formats {
                    if *format == AudioFormat::DecentSampler
                        || *format == AudioFormat::SFZ
                        || *format == AudioFormat::DecentSamplerAndSfz
                        || *format == AudioFormat::All
                        || (*format == AudioFormat::Wav24Bit && has_wav24)
                    {
                        continue;
                    }

                    let sub_config = ExportConfig {
                        output_directory: self.config.output_directory.clone(),
                        naming_pattern: self.config.naming_pattern.clone(),
                        sample_format: format.clone(),
                        normalize: self.config.normalize,
                        fade_in_ms: self.config.fade_in_ms,
                        fade_out_ms: self.config.fade_out_ms,
                        apply_detection: self.config.apply_detection,
                        detection_config: self.config.detection_config.clone(),
                        creator_name: self.config.creator_name.clone(),
                        instrument_description: self.config.instrument_description.clone(),
                        embed_metadata: self.config.embed_metadata,
                    };

                    let exporter = SampleExporter::new(sub_config)?;
                    let files = exporter.export_samples(samples)?;
                    for f in &files {
                        if !all_files.contains(f) {
                            all_files.push(f.clone());
                        }
                    }
                    files_by_format.push((format.clone(), files));
                }
            } else {
                for format in &self.config.formats {
                    let sub_config = ExportConfig {
                        output_directory: self.config.output_directory.clone(),
                        naming_pattern: self.config.naming_pattern.clone(),
                        sample_format: format.clone(),
                        normalize: self.config.normalize,
                        fade_in_ms: self.config.fade_in_ms,
                        fade_out_ms: self.config.fade_out_ms,
                        apply_detection: self.config.apply_detection,
                        detection_config: self.config.detection_config.clone(),
                        creator_name: self.config.creator_name.clone(),
                        instrument_description: self.config.instrument_description.clone(),
                        embed_metadata: self.config.embed_metadata,
                    };

                    let exporter = SampleExporter::new(sub_config)?;
                    let files = exporter.export_samples(samples)?;
                    for f in &files {
                        if !all_files.contains(f) {
                            all_files.push(f.clone());
                        }
                    }
                    files_by_format.push((format.clone(), files));
                }
            }
        }

        Ok(BatchExportResult {
            files_by_format,
            all_files,
        })
    }
}

/// Helper to extract a null-terminated string from a byte slice.
fn parse_null_terminated_str(bytes: &[u8]) -> Option<String> {
    let nul_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let s = String::from_utf8_lossy(&bytes[..nul_pos]).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Read and parse metadata from standard RIFF chunks (`smpl`, `bext`, `LIST-INFO`) in a WAV file.
pub fn read_wav_metadata<P: AsRef<Path>>(path: P) -> Result<WavMetadata> {
    use std::io::{Read, Seek, SeekFrom};

    let mut file = std::fs::File::open(path.as_ref()).map_err(BatcherbirdError::Export)?;
    let file_len = file
        .seek(SeekFrom::End(0))
        .map_err(BatcherbirdError::Export)?;

    if file_len < 12 {
        return Err(BatcherbirdError::Export(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "WAV file is too small to contain a RIFF header",
        )));
    }

    file.seek(SeekFrom::Start(0))
        .map_err(BatcherbirdError::Export)?;
    let mut header = [0u8; 12];
    file.read_exact(&mut header)
        .map_err(BatcherbirdError::Export)?;

    if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
        return Err(BatcherbirdError::Export(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "File is not a valid RIFF/WAVE file",
        )));
    }

    let mut metadata = WavMetadata::default();
    let mut current_pos = 12u64;

    while current_pos + 8 <= file_len {
        file.seek(SeekFrom::Start(current_pos))
            .map_err(BatcherbirdError::Export)?;

        let mut chunk_header = [0u8; 8];
        if file.read_exact(&mut chunk_header).is_err() {
            break;
        }

        let chunk_id = &chunk_header[0..4];
        let chunk_size = u32::from_le_bytes(chunk_header[4..8].try_into().unwrap()) as u64;

        let data_pos = current_pos + 8;
        if data_pos + chunk_size > file_len {
            break;
        }

        let mut data = vec![0u8; chunk_size as usize];
        file.read_exact(&mut data)
            .map_err(BatcherbirdError::Export)?;

        match chunk_id {
            b"smpl" => {
                if data.len() >= 36 {
                    let period = u32::from_le_bytes(data[8..12].try_into().unwrap());
                    let note = u32::from_le_bytes(data[12..16].try_into().unwrap()) as u8;
                    let num_loops = u32::from_le_bytes(data[28..32].try_into().unwrap());

                    metadata.sample_period_ns = Some(period);
                    metadata.midi_unity_note = Some(note);

                    if num_loops >= 1 && data.len() >= 60 {
                        let loop_start = u32::from_le_bytes(data[44..48].try_into().unwrap());
                        let loop_end = u32::from_le_bytes(data[48..52].try_into().unwrap());
                        metadata.loop_points = Some((loop_start, loop_end));
                    }
                }
            }
            b"bext" => {
                if data.len() >= 338 {
                    metadata.bext_description = parse_null_terminated_str(&data[0..256]);
                    metadata.bext_originator = parse_null_terminated_str(&data[256..288]);
                    metadata.bext_origination_date = parse_null_terminated_str(&data[320..330]);
                    metadata.bext_origination_time = parse_null_terminated_str(&data[330..338]);
                }
            }
            b"LIST" => {
                if data.len() >= 4 && &data[0..4] == b"INFO" {
                    let mut pos = 4usize;
                    while pos + 8 <= data.len() {
                        let sub_id = &data[pos..pos + 4];
                        let sub_len = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap())
                            as usize;
                        let val_start = pos + 8;
                        let val_end = val_start + sub_len;
                        if val_end > data.len() {
                            break;
                        }

                        let text_opt = parse_null_terminated_str(&data[val_start..val_end]);
                        if let Some(text) = text_opt {
                            match sub_id {
                                b"INAM" => metadata.info_title = Some(text),
                                b"IART" => metadata.info_artist = Some(text),
                                b"ICRD" => metadata.info_date = Some(text),
                                b"ISFT" => metadata.info_software = Some(text),
                                b"ICMT" => metadata.info_comment = Some(text),
                                _ => {}
                            }
                        }

                        let pad = if !sub_len.is_multiple_of(2) { 1 } else { 0 };
                        pos = val_end + pad;
                    }
                }
            }
            _ => {}
        }

        let pad = if !chunk_size.is_multiple_of(2) { 1 } else { 0 };
        current_pos = data_pos + chunk_size + pad;
    }

    Ok(metadata)
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

    #[test]
    fn test_smpl_chunk_structure_without_loop() {
        let chunk = SampleExporter::build_smpl_chunk(48000, 60, None);
        assert_eq!(&chunk[0..4], b"smpl");
        let size = u32::from_le_bytes(chunk[4..8].try_into().unwrap());
        assert_eq!(size, 36);
        assert_eq!(chunk.len(), 44);

        // sample_period: 1_000_000_000 / 48000 = 20833
        let sample_period = u32::from_le_bytes(chunk[16..20].try_into().unwrap());
        assert_eq!(sample_period, 20833);

        // midi_unity_note = 60
        let note = u32::from_le_bytes(chunk[20..24].try_into().unwrap());
        assert_eq!(note, 60);

        // num_sample_loops = 0
        let num_loops = u32::from_le_bytes(chunk[36..40].try_into().unwrap());
        assert_eq!(num_loops, 0);
    }

    #[test]
    fn test_smpl_chunk_structure_with_loop() {
        let chunk = SampleExporter::build_smpl_chunk(44100, 69, Some((1000, 5000)));
        assert_eq!(&chunk[0..4], b"smpl");
        let size = u32::from_le_bytes(chunk[4..8].try_into().unwrap());
        assert_eq!(size, 60);
        assert_eq!(chunk.len(), 68);

        // num_sample_loops = 1
        let num_loops = u32::from_le_bytes(chunk[36..40].try_into().unwrap());
        assert_eq!(num_loops, 1);

        // loop type = 0 (forward)
        let loop_type = u32::from_le_bytes(chunk[48..52].try_into().unwrap());
        assert_eq!(loop_type, 0);

        // start = 1000, end = 5000
        let start = u32::from_le_bytes(chunk[52..56].try_into().unwrap());
        let end = u32::from_le_bytes(chunk[56..60].try_into().unwrap());
        assert_eq!(start, 1000);
        assert_eq!(end, 5000);
    }

    #[test]
    fn test_bext_chunk_structure() {
        let ts = chrono::DateTime::parse_from_rfc3339("2026-09-21T15:30:45Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let chunk = SampleExporter::build_bext_chunk(
            Some("My Originator"),
            Some("Bass Lead"),
            &ts,
        );

        assert_eq!(&chunk[0..4], b"bext");
        let size = u32::from_le_bytes(chunk[4..8].try_into().unwrap());
        assert_eq!(size, 602);
        assert_eq!(chunk.len(), 610);

        let desc = parse_null_terminated_str(&chunk[8..264]).unwrap();
        assert_eq!(desc, "Bass Lead");

        let orig = parse_null_terminated_str(&chunk[264..296]).unwrap();
        assert_eq!(orig, "My Originator");

        let date = parse_null_terminated_str(&chunk[328..338]).unwrap();
        assert_eq!(date, "2026-09-21");

        let time = parse_null_terminated_str(&chunk[338..346]).unwrap();
        assert_eq!(time, "15:30:45");
    }

    #[test]
    fn test_info_chunk_structure() {
        let chunk = SampleExporter::build_info_chunk(
            "C4 Piano",
            Some("Test Artist"),
            "2026-09-21",
            "Batcherbird",
            Some("Vel 127"),
        );

        assert_eq!(&chunk[0..4], b"LIST");
        assert_eq!(&chunk[8..12], b"INFO");

        // Length must be 2-byte aligned
        assert!(chunk.len().is_multiple_of(2));
    }

    #[test]
    fn test_wav_export_with_and_without_metadata() {
        let temp_dir = std::env::temp_dir().join("batcherbird_test_export_meta_flag");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let sample = Sample {
            note: 60,
            velocity: 99,
            audio_data: vec![0.1, -0.2, 0.3, -0.4, 0.5, 0.0, -0.1],
            sample_rate: 44100,
            channels: 1,
            recorded_at: std::time::SystemTime::now(),
            midi_timing: std::time::Duration::from_millis(50),
            audio_timing: std::time::Duration::from_millis(500),
        };

        // 1. Export with embed_metadata = true
        let config_with = ExportConfig {
            output_directory: temp_dir.clone(),
            naming_pattern: "with_meta.wav".to_string(),
            sample_format: AudioFormat::Wav16Bit,
            normalize: false,
            fade_in_ms: 0.0,
            fade_out_ms: 0.0,
            apply_detection: false,
            creator_name: Some("Audio Maker".to_string()),
            instrument_description: Some("Preset 1".to_string()),
            embed_metadata: true,
            ..Default::default()
        };
        let exp_with = SampleExporter::new(config_with).unwrap();
        let path_with = exp_with.export_sample(&sample).unwrap();

        let meta_with = read_wav_metadata(&path_with).unwrap();
        assert_eq!(meta_with.midi_unity_note, Some(60));
        assert_eq!(meta_with.bext_originator.as_deref(), Some("Audio Maker"));
        assert_eq!(meta_with.info_software.as_deref(), Some("Batcherbird"));

        // Verify hound reads audio data cleanly
        let mut reader = hound::WavReader::open(&path_with).unwrap();
        let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
        assert_eq!(samples.len(), 7);

        // 2. Export with embed_metadata = false
        let config_without = ExportConfig {
            output_directory: temp_dir.clone(),
            naming_pattern: "without_meta.wav".to_string(),
            sample_format: AudioFormat::Wav16Bit,
            normalize: false,
            fade_in_ms: 0.0,
            fade_out_ms: 0.0,
            apply_detection: false,
            creator_name: Some("Audio Maker".to_string()),
            instrument_description: Some("Preset 1".to_string()),
            embed_metadata: false,
            ..Default::default()
        };
        let exp_without = SampleExporter::new(config_without).unwrap();
        let path_without = exp_without.export_sample(&sample).unwrap();

        let meta_without = read_wav_metadata(&path_without).unwrap();
        assert_eq!(meta_without.midi_unity_note, None);
        assert_eq!(meta_without.bext_originator, None);
        assert_eq!(meta_without.info_software, None);

        // Verify audio data is identical between both
        let mut reader_no_meta = hound::WavReader::open(&path_without).unwrap();
        let samples_no_meta: Vec<i16> = reader_no_meta.samples::<i16>().map(|s| s.unwrap()).collect();
        assert_eq!(samples, samples_no_meta);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_audio_format_directory_name() {
        assert_eq!(AudioFormat::Wav16Bit.directory_name(), "WAV_16Bit");
        assert_eq!(AudioFormat::Wav24Bit.directory_name(), "WAV_24Bit");
        assert_eq!(AudioFormat::Wav32BitFloat.directory_name(), "WAV_32BitFloat");
        assert_eq!(AudioFormat::DecentSampler.directory_name(), "DecentSampler");
        assert_eq!(AudioFormat::SFZ.directory_name(), "SFZ");
        assert_eq!(AudioFormat::DecentSamplerAndSfz.directory_name(), "DecentSampler_SFZ");
        assert_eq!(AudioFormat::All.directory_name(), "All_Formats");
    }

    #[test]
    fn test_simultaneous_decent_sampler_and_sfz_export() {
        let temp_dir = std::env::temp_dir().join("batcherbird_test_simul_ds_sfz");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let samples = vec![
            Sample {
                note: 60,
                velocity: 64,
                audio_data: vec![0.1, -0.2, 0.3, 0.0],
                sample_rate: 44100,
                channels: 1,
                recorded_at: std::time::SystemTime::now(),
                midi_timing: std::time::Duration::from_millis(50),
                audio_timing: std::time::Duration::from_millis(500),
            },
            Sample {
                note: 64,
                velocity: 127,
                audio_data: vec![0.2, -0.4, 0.6, 0.0],
                sample_rate: 44100,
                channels: 1,
                recorded_at: std::time::SystemTime::now(),
                midi_timing: std::time::Duration::from_millis(50),
                audio_timing: std::time::Duration::from_millis(500),
            },
        ];

        let config = ExportConfig {
            output_directory: temp_dir.clone(),
            naming_pattern: "Simul_{note_name}_{note}_{velocity}.wav".to_string(),
            sample_format: AudioFormat::DecentSamplerAndSfz,
            creator_name: Some("Patch Master".to_string()),
            instrument_description: Some("Multi Synth".to_string()),
            ..Default::default()
        };

        let exporter = SampleExporter::new(config).unwrap();
        let files = exporter.export_samples(&samples).unwrap();

        // Should produce 2 WAV files + 1 .dspreset + 1 .sfz = 4 files
        assert_eq!(files.len(), 4);

        let dspreset_file = files.iter().find(|p| p.extension().is_some_and(|e| e == "dspreset"));
        let sfz_file = files.iter().find(|p| p.extension().is_some_and(|e| e == "sfz"));
        let wav_files: Vec<_> = files.iter().filter(|p| p.extension().is_some_and(|e| e == "wav")).collect();

        assert!(dspreset_file.is_some());
        assert!(sfz_file.is_some());
        assert_eq!(wav_files.len(), 2);

        // Verify both presets exist and have valid content referencing the WAVs
        let ds_content = std::fs::read_to_string(dspreset_file.unwrap()).unwrap();
        assert!(ds_content.contains("<DecentSampler"));
        assert!(ds_content.contains("Simul_C4_60_vel064.wav"));
        assert!(ds_content.contains("Simul_E4_64_vel127.wav"));

        let sfz_content = std::fs::read_to_string(sfz_file.unwrap()).unwrap();
        assert!(sfz_content.contains("Generated by Batcherbird"));
        assert!(sfz_content.contains("<region>"));
        assert!(sfz_content.contains("Simul_C4_60_vel064.wav"));
        assert!(sfz_content.contains("Simul_E4_64_vel127.wav"));

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_batch_exporter_shared_and_subdirectories() {
        let sample = Sample {
            note: 60,
            velocity: 100,
            audio_data: vec![0.1, -0.2, 0.3, 0.0],
            sample_rate: 44100,
            channels: 1,
            recorded_at: std::time::SystemTime::now(),
            midi_timing: std::time::Duration::from_millis(50),
            audio_timing: std::time::Duration::from_millis(500),
        };

        // 1. Test shared mode
        let shared_dir = std::env::temp_dir().join("batcherbird_test_batch_shared");
        std::fs::create_dir_all(&shared_dir).unwrap();

        let batch_config_shared = BatchExportConfig {
            output_directory: shared_dir.clone(),
            naming_pattern: "Shared_{note_name}_{note}_{velocity}.wav".to_string(),
            formats: vec![AudioFormat::DecentSampler, AudioFormat::SFZ],
            organize_subdirectories: false,
            ..Default::default()
        };

        let batch_exp = BatchExporter::new(batch_config_shared).unwrap();
        let result = batch_exp
            .export_samples(std::slice::from_ref(&sample))
            .unwrap();

        // In shared mode with DS + SFZ, 1 WAV + 1 dspreset + 1 sfz = 3 files
        assert_eq!(result.all_files.len(), 3);
        assert!(shared_dir.join("Shared.dspreset").exists());
        assert!(shared_dir.join("Shared.sfz").exists());
        std::fs::remove_dir_all(&shared_dir).ok();

        // 2. Test organized subdirectories mode
        let sub_dir = std::env::temp_dir().join("batcherbird_test_batch_subdirs");
        std::fs::create_dir_all(&sub_dir).unwrap();

        let batch_config_subdirs = BatchExportConfig {
            output_directory: sub_dir.clone(),
            naming_pattern: "Sub_{note_name}_{note}_{velocity}.wav".to_string(),
            formats: vec![AudioFormat::Wav24Bit, AudioFormat::DecentSampler, AudioFormat::SFZ],
            organize_subdirectories: true,
            ..Default::default()
        };

        let batch_exp_sub = BatchExporter::new(batch_config_subdirs).unwrap();
        let result_sub = batch_exp_sub.export_samples(&[sample]).unwrap();

        assert_eq!(result_sub.files_by_format.len(), 3);
        assert!(sub_dir.join("WAV_24Bit").exists());
        assert!(sub_dir.join("DecentSampler").exists());
        assert!(sub_dir.join("SFZ").exists());

        std::fs::remove_dir_all(&sub_dir).ok();
    }
}
