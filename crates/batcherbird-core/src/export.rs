use crate::{Result, BatcherbirdError};
use crate::sampler::Sample;
use crate::detection::DetectionConfig;
use hound::{WavWriter, WavSpec, SampleFormat};
use std::path::{Path, PathBuf};
use std::fs;

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
    SFZ, // Generates .sfz file with WAV samples
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
            apply_detection: true,  // Enable detection by default
            detection_config: DetectionConfig::default(),
            creator_name: None,
            instrument_description: None,
        }
    }
}

pub struct SampleExporter {
    config: ExportConfig,
}

impl SampleExporter {
    pub fn new(config: ExportConfig) -> Result<Self> {
        // Create output directory if it doesn't exist
        if !config.output_directory.exists() {
            fs::create_dir_all(&config.output_directory)
                .map_err(|e| BatcherbirdError::Export(e))?;
        }
        
        Ok(Self { config })
    }

    pub fn export_sample(&self, sample: &Sample) -> Result<PathBuf> {
        let filename = self.generate_filename(sample);
        let filepath = self.config.output_directory.join(&filename);
        
        println!("💾 Exporting sample: {}", filename);
        
        // Clone sample for processing (detection may modify audio data)
        let mut sample_copy = sample.clone();
        
        // Apply sample detection if enabled
        if self.config.apply_detection {
            println!("🔍 Applying sample detection...");
            match sample_copy.apply_detection(self.config.detection_config.clone()) {
                Ok(detection_result) => {
                    if detection_result.success {
                        println!("   ✅ Detection successful, sample trimmed");
                    } else {
                        println!("   ⚠️ Detection failed: {}", 
                            detection_result.failure_reason.as_deref().unwrap_or("Unknown"));
                        println!("   📝 Exporting original sample without trimming");
                    }
                },
                Err(e) => {
                    println!("   ❌ Detection error: {}", e);
                    println!("   📝 Exporting original sample without trimming");
                }
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
            },
            AudioFormat::SFZ => {
                // For SFZ, we only write WAV files here
                // The .sfz file will be generated separately via export_samples()
                let wav_config = ExportConfig {
                    sample_format: AudioFormat::Wav24Bit, // Use 24-bit for good compatibility
                    ..self.config.clone()
                };
                let temp_exporter = SampleExporter { config: wav_config };
                temp_exporter.write_wav_file(&filepath, &audio_data, sample)?;
            },
            _ => {
                // Standard WAV export
                self.write_wav_file(&filepath, &audio_data, sample)?;
            }
        }
        
        println!("   ✅ Saved: {}", filepath.display());
        Ok(filepath)
    }

    pub fn export_samples(&self, samples: &[Sample]) -> Result<Vec<PathBuf>> {
        let mut exported_files = Vec::new();
        
        println!("💾 Exporting {} samples to: {}", samples.len(), self.config.output_directory.display());
        
        for (i, sample) in samples.iter().enumerate() {
            println!("   Exporting sample {} of {}...", i + 1, samples.len());
            let filepath = self.export_sample(sample)?;
            exported_files.push(filepath);
        }
        
        // Generate .dspreset XML file for DecentSampler format
        if matches!(self.config.sample_format, AudioFormat::DecentSampler) {
            println!("🎹 Generating Decent Sampler .dspreset file...");
            let dspreset_path = self.generate_dspreset_file(samples, &exported_files)?;
            exported_files.push(dspreset_path);
        }
        
        // Generate .sfz file for SFZ format
        if matches!(self.config.sample_format, AudioFormat::SFZ) {
            println!("🎼 Generating SFZ .sfz file...");
            let sfz_path = self.generate_sfz_file(samples, &exported_files)?;
            exported_files.push(sfz_path);
        }
        
        println!("✅ Exported {} samples successfully!", samples.len());
        Ok(exported_files)
    }

    fn generate_filename(&self, sample: &Sample) -> String {
        let note_name = Self::note_to_name(sample.note);
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        
        // Consistent "vel" prefix naming for all samples: C4_60_vel127.wav
        self.config.naming_pattern
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
            for i in 0..fade_in_samples.min(len) {
                let fade_factor = i as f32 / fade_in_samples as f32;
                audio_data[i] *= fade_factor;
            }
        }
        
        // Apply fade out
        if fade_out_samples > 0 && fade_out_samples < len {
            let fade_start = len.saturating_sub(fade_out_samples);
            for i in fade_start..len {
                let fade_factor = (len - i) as f32 / fade_out_samples as f32;
                audio_data[i] *= fade_factor;
            }
        }
        
        Ok(())
    }

    fn normalize_audio(&self, audio_data: &mut [f32]) -> Result<()> {
        // Find peak amplitude
        let peak = audio_data.iter()
            .map(|&sample| sample.abs())
            .fold(0.0f32, f32::max);
        
        if peak > 0.0 && peak < 1.0 {
            let gain = 0.95 / peak; // Normalize to 95% to avoid clipping
            for sample in audio_data.iter_mut() {
                *sample *= gain;
            }
            println!("   🔊 Normalized: +{:.1} dB gain", 20.0 * gain.log10());
        }
        
        Ok(())
    }

    fn write_wav_file(&self, filepath: &Path, audio_data: &[f32], sample: &Sample) -> Result<()> {
        println!("🔍 Writing WAV file: {} ({} samples)", filepath.display(), audio_data.len());
        
        // Validate audio data first
        if audio_data.is_empty() {
            return Err(BatcherbirdError::Export(std::io::Error::new(
                std::io::ErrorKind::InvalidData, 
                "Cannot export empty audio data"
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
                    "DecentSampler format should be handled separately, not in WAV writing"
                )));
            }
        };

        println!("🔍 WAV spec: {}Hz, {} channels, {} bits", spec.sample_rate, spec.channels, spec.bits_per_sample);

        // Create writer with explicit error handling
        let mut writer = match WavWriter::create(filepath, spec) {
            Ok(w) => {
                println!("✅ WAV writer created successfully");
                w
            },
            Err(e) => {
                println!("❌ Failed to create WAV writer: {}", e);
                return Err(BatcherbirdError::Export(std::io::Error::new(std::io::ErrorKind::Other, e)));
            }
        };

        // Write samples with progress tracking
        let total_samples = audio_data.len();
        match self.config.sample_format {
            AudioFormat::Wav16Bit => {
                for (i, &sample) in audio_data.iter().enumerate() {
                    let sample_i16 = (sample * i16::MAX as f32) as i16;
                    if let Err(e) = writer.write_sample(sample_i16) {
                        println!("❌ Failed to write sample {} of {}: {}", i, total_samples, e);
                        return Err(BatcherbirdError::Export(std::io::Error::new(std::io::ErrorKind::Other, e)));
                    }
                }
            }
            AudioFormat::Wav24Bit => {
                for (i, &sample) in audio_data.iter().enumerate() {
                    let sample_i32 = (sample * 8_388_607.0) as i32; // 24-bit max value
                    if let Err(e) = writer.write_sample(sample_i32) {
                        println!("❌ Failed to write sample {} of {}: {}", i, total_samples, e);
                        return Err(BatcherbirdError::Export(std::io::Error::new(std::io::ErrorKind::Other, e)));
                    }
                }
            }
            AudioFormat::Wav32BitFloat => {
                for (i, &sample) in audio_data.iter().enumerate() {
                    if let Err(e) = writer.write_sample(sample) {
                        println!("❌ Failed to write sample {} of {}: {}", i, total_samples, e);
                        return Err(BatcherbirdError::Export(std::io::Error::new(std::io::ErrorKind::Other, e)));
                    }
                }
            }
            AudioFormat::DecentSampler => {
                return Err(BatcherbirdError::Export(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "DecentSampler format should not reach write_wav_file - this is a logic error"
                )));
            },
            AudioFormat::SFZ => {
                return Err(BatcherbirdError::Export(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "SFZ format should not reach write_wav_file - this is a logic error"
                )));
            }
        }

        println!("✅ All {} samples written, finalizing...", total_samples);

        // Finalize with explicit error handling
        match writer.finalize() {
            Ok(_) => {
                println!("✅ WAV file finalized successfully");
            },
            Err(e) => {
                println!("❌ Failed to finalize WAV file: {}", e);
                return Err(BatcherbirdError::Export(std::io::Error::new(std::io::ErrorKind::Other, e)));
            }
        }

        // Explicitly sync file to disk to prevent corruption during rapid batch exports
        match std::fs::File::open(filepath) {
            Ok(file) => {
                if let Err(e) = file.sync_all() {
                    println!("⚠️ Warning: Failed to sync file to disk: {}", e);
                } else {
                    println!("✅ File synced to disk successfully");
                }
            },
            Err(e) => {
                println!("⚠️ Warning: Could not reopen file for sync: {}", e);
            }
        }

        // Verify file was created and has reasonable size
        match std::fs::metadata(filepath) {
            Ok(metadata) => {
                let file_size = metadata.len();
                println!("✅ File created: {} bytes", file_size);
                
                // Basic sanity check - WAV header is 44 bytes, so file should be larger
                if file_size < 100 {
                    println!("⚠️ Warning: File size suspiciously small: {} bytes", file_size);
                }
            },
            Err(e) => {
                println!("❌ Failed to verify file creation: {}", e);
                return Err(BatcherbirdError::Export(e));
            }
        }

        Ok(())
    }

    fn note_to_name(note: u8) -> String {
        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let octave = (note / 12).saturating_sub(1);
        let note_name = note_names[(note % 12) as usize];
        format!("{}{}", note_name, octave)
    }

    /// Generate a Decent Sampler .dspreset XML file
    fn generate_dspreset_file(&self, samples: &[Sample], wav_files: &[PathBuf]) -> Result<PathBuf> {
        use std::io::Write;
        
        // Create the .dspreset filename (use the sample name from config or default)
        let preset_name = self.config.naming_pattern
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
            preset_name 
        };
        
        let dspreset_filename = format!("{}.dspreset", preset_name);
        let dspreset_path = self.config.output_directory.join(&dspreset_filename);
        
        // Group samples by velocity for layering
        let mut velocity_groups = std::collections::HashMap::new();
        for (i, sample) in samples.iter().enumerate() {
            if i < wav_files.len() {
                velocity_groups.entry(sample.velocity)
                    .or_insert_with(Vec::new)
                    .push((sample, &wav_files[i]));
            }
        }
        
        // Generate XML content
        let xml_content = self.generate_dspreset_xml(&preset_name, &velocity_groups)?;
        
        // Write XML file
        let mut file = std::fs::File::create(&dspreset_path)
            .map_err(|e| BatcherbirdError::Export(e))?;
        
        file.write_all(xml_content.as_bytes())
            .map_err(|e| BatcherbirdError::Export(e))?;
            
        println!("   ✅ Generated Decent Sampler preset: {}", dspreset_filename);
        Ok(dspreset_path)
    }
    
    /// Generate the XML content for a Decent Sampler .dspreset file
    fn generate_dspreset_xml(&self, preset_name: &str, velocity_groups: &std::collections::HashMap<u8, Vec<(&Sample, &PathBuf)>>) -> Result<String> {
        let mut xml = String::new();
        
        // XML Declaration and root element  
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str(&format!("<!-- {} - Generated by Batcherbird -->\n", preset_name));
        
        // Add creator and description in comment if provided
        if let Some(ref creator) = self.config.creator_name {
            xml.push_str(&format!("<!-- Creator: {} -->\n", creator));
        }
        if let Some(ref description) = self.config.instrument_description {
            xml.push_str(&format!("<!-- Description: {} -->\n", description));
        }
        
        xml.push_str("<DecentSampler minVersion=\"1.0.0\">\n");
        
        // Add info section with metadata
        if self.config.creator_name.is_some() || self.config.instrument_description.is_some() {
            xml.push_str("  <info>\n");
            if let Some(ref creator) = self.config.creator_name {
                xml.push_str(&format!("    <author>{}</author>\n", creator));
            }
            if let Some(ref description) = self.config.instrument_description {
                xml.push_str(&format!("    <description>{}</description>\n", description));
            }
            xml.push_str("  </info>\n\n");
        }
        
        // UI Section - Basic controls
        xml.push_str("  <ui>\n");
        xml.push_str("    <tab name=\"main\">\n");
        xml.push_str("      <labeled-knob x=\"75\" y=\"75\" width=\"90\" height=\"105\" label=\"Attack\" type=\"float\" minValue=\"0.0\" maxValue=\"4.0\" value=\"0.05\" textColor=\"AA000000\">\n");
        xml.push_str("        <binding type=\"amp\" level=\"instrument\" position=\"0\" parameter=\"ENV_ATTACK\" />\n");
        xml.push_str("      </labeled-knob>\n");
        xml.push_str("      <labeled-knob x=\"175\" y=\"75\" width=\"90\" height=\"105\" label=\"Release\" type=\"float\" minValue=\"0.0\" maxValue=\"4.0\" value=\"1.0\" textColor=\"AA000000\">\n");
        xml.push_str("        <binding type=\"amp\" level=\"instrument\" position=\"0\" parameter=\"ENV_RELEASE\" />\n");
        xml.push_str("      </labeled-knob>\n");
        xml.push_str("      <labeled-knob x=\"275\" y=\"75\" width=\"90\" height=\"105\" label=\"Tone\" type=\"float\" minValue=\"0.0\" maxValue=\"1.0\" value=\"1.0\" textColor=\"AA000000\">\n");
        xml.push_str("        <binding type=\"effect\" level=\"instrument\" position=\"0\" parameter=\"FX_FILTER_FREQUENCY\" />\n");
        xml.push_str("      </labeled-knob>\n");
        xml.push_str("      <labeled-knob x=\"375\" y=\"75\" width=\"90\" height=\"105\" label=\"Reverb\" type=\"float\" minValue=\"0.0\" maxValue=\"1.0\" value=\"0.2\" textColor=\"AA000000\">\n");
        xml.push_str("        <binding type=\"effect\" level=\"instrument\" position=\"1\" parameter=\"FX_REVERB_WET_LEVEL\" />\n");
        xml.push_str("      </labeled-knob>\n");
        xml.push_str("    </tab>\n");
        xml.push_str("  </ui>\n\n");
        
        // Groups Section - Sample mappings organized by velocity
        xml.push_str("  <groups>\n");
        
        // Sort velocity groups for consistent output
        let mut sorted_velocities: Vec<_> = velocity_groups.keys().collect();
        sorted_velocities.sort();
        
        for (group_index, &velocity) in sorted_velocities.iter().enumerate() {
            if let Some(samples) = velocity_groups.get(velocity) {
                xml.push_str(&format!("    <group ampVelTrack=\"{:.1}\" volume=\"0.0\">\n", 
                    if sorted_velocities.len() > 1 { 1.0 } else { 0.5 })); // Enable velocity tracking if multiple layers
                
                // Add envelope settings
                xml.push_str("      <amplifier attack=\"0.05\" decay=\"0.0\" sustain=\"1.0\" release=\"1.0\" />\n");
                
                // Add samples for this velocity group
                for (sample, wav_file) in samples {
                    let filename = wav_file.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("sample.wav");
                        
                    // For velocity layers, adjust the velocity range
                    let (lo_vel, hi_vel) = if sorted_velocities.len() == 1 {
                        (0, 127) // Single velocity covers full range
                    } else {
                        // Distribute velocity ranges among layers
                        let vel_range = 127 / sorted_velocities.len() as i32;
                        let lo = (group_index as i32 * vel_range).max(0) as u8;
                        let hi = ((group_index + 1) as i32 * vel_range).min(127) as u8;
                        (lo, hi)
                    };
                    
                    xml.push_str(&format!(
                        "      <sample loNote=\"{}\" hiNote=\"{}\" rootNote=\"{}\" loVel=\"{}\" hiVel=\"{}\" path=\"{}\" />\n",
                        sample.note, sample.note, sample.note, lo_vel, hi_vel, filename
                    ));
                }
                
                xml.push_str("    </group>\n");
            }
        }
        
        xml.push_str("  </groups>\n\n");
        
        // Effects Section - Basic processing
        xml.push_str("  <effects>\n");
        xml.push_str("    <effect type=\"lowpass\" frequency=\"22000.0\" />\n");
        xml.push_str("    <effect type=\"reverb\" roomSize=\"0.8\" damping=\"0.2\" wetLevel=\"0.2\" dryLevel=\"1.0\" />\n");
        xml.push_str("  </effects>\n\n");
        
        // MIDI Section - Control mappings
        xml.push_str("  <midi>\n");
        xml.push_str("    <!-- MIDI CC bindings for knobs -->\n");
        xml.push_str("  </midi>\n");
        
        // Close root element
        xml.push_str("</DecentSampler>\n");
        
        Ok(xml)
    }
    
    /// Generate an SFZ .sfz file
    fn generate_sfz_file(&self, samples: &[Sample], wav_files: &[PathBuf]) -> Result<PathBuf> {
        use std::io::Write;
        
        // Create the .sfz filename (use the sample name from config or default)
        let preset_name = self.config.naming_pattern
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
            preset_name
        };
        
        let sfz_filename = format!("{}.sfz", preset_name);
        let sfz_path = self.config.output_directory.join(&sfz_filename);
        
        // Group samples by velocity for layering
        let mut velocity_groups = std::collections::HashMap::new();
        for (i, sample) in samples.iter().enumerate() {
            if i < wav_files.len() {
                velocity_groups.entry(sample.velocity)
                    .or_insert_with(Vec::new)
                    .push((sample, &wav_files[i]));
            }
        }
        
        // Generate SFZ content
        let sfz_content = self.generate_sfz_content(&preset_name, &velocity_groups)?;
        
        // Write SFZ file
        let mut file = std::fs::File::create(&sfz_path)
            .map_err(|e| BatcherbirdError::Export(e))?;
        
        file.write_all(sfz_content.as_bytes())
            .map_err(|e| BatcherbirdError::Export(e))?;
            
        println!("   ✅ Generated SFZ instrument: {}", sfz_filename);
        Ok(sfz_path)
    }
    
    /// Generate the SFZ content
    fn generate_sfz_content(&self, preset_name: &str, velocity_groups: &std::collections::HashMap<u8, Vec<(&Sample, &PathBuf)>>) -> Result<String> {
        let mut sfz = String::new();
        
        // SFZ Header with comments
        sfz.push_str(&format!("// {} - Generated by Batcherbird\n", preset_name));
        
        // Add creator and description in comments if provided
        if let Some(ref creator) = self.config.creator_name {
            sfz.push_str(&format!("// Creator: {}\n", creator));
        }
        if let Some(ref description) = self.config.instrument_description {
            sfz.push_str(&format!("// Description: {}\n", description));
        }
        
        sfz.push_str("\n");
        
        // Control section - path settings
        sfz.push_str("<control>\n");
        sfz.push_str("default_path=samples/\n");
        sfz.push_str("\n");
        
        // Global section - overall settings
        sfz.push_str("<global>\n");
        sfz.push_str("ampeg_release=0.5\n");
        sfz.push_str("\n");
        
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
                    sfz.push_str("\n");
                }
                
                // Add regions (samples) for this velocity group
                for (sample, wav_file) in samples {
                    let filename = wav_file.file_name()
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
                    
                    sfz.push_str("\n");
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