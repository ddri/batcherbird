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
}

#[derive(Debug, Clone)]
pub enum AudioFormat {
    Wav16Bit,
    Wav24Bit,
    Wav32BitFloat,
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
        
        // Write WAV file
        self.write_wav_file(&filepath, &audio_data, sample)?;
        
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