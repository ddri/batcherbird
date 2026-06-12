use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

/// Complete session configuration following professional audio standards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub project_name: String,
    pub project_directory: PathBuf,
    pub audio: AudioSessionConfig,
    pub midi: MidiSessionConfig,
    pub recording: RecordingSessionConfig,
    pub export: ExportSessionConfig,
    pub created_at: SystemTime,
}

/// Audio configuration with professional defaults and validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSessionConfig {
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    pub sample_rate: u32,        // 44100, 48000, 88200, 96000, etc.
    pub bit_depth: u16,          // 16, 24, 32
    pub buffer_size: u32,        // 128, 256, 512, 1024, 2048
    pub input_channels: Vec<u8>, // Selected input channels
    pub monitoring_enabled: bool,
    pub playthrough_enabled: bool,
}

/// MIDI configuration with professional defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiSessionConfig {
    pub output_device: Option<String>,
    pub channel: u8, // 0-15 (MIDI channels 1-16)
    pub velocity_curve: VelocityCurve,
    pub program_change_delay_ms: u32,
}

/// Recording parameters following professional standards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSessionConfig {
    pub note_duration_ms: u64, // 100-30000ms
    pub release_time_ms: u64,  // 0-10000ms
    pub pre_delay_ms: u64,     // 0-1000ms
    pub post_delay_ms: u64,    // 0-1000ms
    pub auto_detect_silence: bool,
    pub detection_threshold_db: f32, // -60.0 to -6.0
}

/// Export configuration with professional file organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSessionConfig {
    pub output_directory: PathBuf,
    pub naming_pattern: String,
    pub sample_format: AudioFormat,
    pub normalize: bool,
    pub fade_in_ms: f32,
    pub fade_out_ms: f32,
    pub creator_name: Option<String>,
    pub project_description: Option<String>,
}

/// Velocity curve options for MIDI response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VelocityCurve {
    Linear,
    Exponential,
    Logarithmic,
    Custom(Vec<f32>),
}

/// Audio format options following professional standards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioFormat {
    Wav16Bit,
    Wav24Bit,
    Wav32BitFloat,
    DecentSampler,
    SFZ,
    All,
}

/// Export target determines recommended formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportTarget {
    LivePerformance,
    StudioProduction,
    Distribution,
    Archival,
}

/// Professional project structure following DAW conventions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStructure {
    pub project_root: PathBuf,
    pub samples_dir: PathBuf,
    pub exports_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub sessions_dir: PathBuf,
}

/// Session state tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionState {
    Uninitialized,
    Initializing,
    Ready,
    Recording,
    Error(String),
}

/// Validation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// Validation report containing all issues found
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
    pub is_valid: bool,
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            is_valid: true,
        }
    }

    pub fn add_error(&mut self, field: &str, message: &str) {
        self.errors.push(ValidationError {
            field: field.to_string(),
            message: message.to_string(),
            severity: ValidationSeverity::Error,
        });
        self.is_valid = false;
    }

    pub fn add_warning(&mut self, field: &str, message: &str) {
        self.warnings.push(ValidationError {
            field: field.to_string(),
            message: message.to_string(),
            severity: ValidationSeverity::Warning,
        });
    }
}

/// Professional defaults based on industry standards
impl Default for AudioSessionConfig {
    fn default() -> Self {
        Self {
            input_device: None,
            output_device: None,
            sample_rate: 44100,      // Music industry standard
            bit_depth: 16,           // Music industry standard
            buffer_size: 512,        // Low latency balance
            input_channels: vec![0], // First channel default
            monitoring_enabled: true,
            playthrough_enabled: false,
        }
    }
}

impl Default for MidiSessionConfig {
    fn default() -> Self {
        Self {
            output_device: None,
            channel: 0, // MIDI channel 1 (0-indexed)
            velocity_curve: VelocityCurve::Linear,
            program_change_delay_ms: 50,
        }
    }
}

impl Default for RecordingSessionConfig {
    fn default() -> Self {
        Self {
            note_duration_ms: 2500, // 2.5s captures full decay
            release_time_ms: 1000,  // 1s professional standard
            pre_delay_ms: 100,      // Eliminates MIDI latency
            post_delay_ms: 100,     // Clean buffer flush
            auto_detect_silence: true,
            detection_threshold_db: -35.0, // Professional threshold
        }
    }
}

impl Default for ExportSessionConfig {
    fn default() -> Self {
        Self {
            output_directory: PathBuf::new(),
            naming_pattern: "{project_name}_{note_name}_{note}_{velocity}.wav".to_string(),
            sample_format: AudioFormat::Wav16Bit,
            normalize: false,
            fade_in_ms: 0.0,
            fade_out_ms: 10.0, // Professional fade-out
            creator_name: None,
            project_description: None,
        }
    }
}

/// Get professional audio defaults based on device capabilities
pub fn get_professional_audio_defaults(_device_name: Option<&str>) -> AudioSessionConfig {
    // TODO: Query actual device capabilities when device manager is available
    // For now, return conservative professional defaults
    AudioSessionConfig {
        sample_rate: 44100, // Music industry standard
        bit_depth: 16,      // Music industry standard
        buffer_size: 512,   // Good latency/stability balance
        ..Default::default()
    }
}

/// Create professional project structure
pub fn create_project_structure(
    project_name: &str,
    base_directory: Option<PathBuf>,
) -> Result<ProjectStructure, std::io::Error> {
    // Use Documents/BatcherBird Projects as default location
    let base = base_directory.unwrap_or_else(|| {
        dirs::document_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("BatcherBird Projects")
    });

    let project_root = base.join(sanitize_filename(project_name));

    let structure = ProjectStructure {
        samples_dir: project_root.join("Samples"),
        exports_dir: project_root.join("Exports"),
        templates_dir: project_root.join("Templates"),
        sessions_dir: project_root.join("Sessions"),
        project_root: project_root.clone(),
    };

    // Create all directories
    std::fs::create_dir_all(&structure.project_root)?;
    std::fs::create_dir_all(&structure.samples_dir)?;
    std::fs::create_dir_all(&structure.exports_dir)?;
    std::fs::create_dir_all(&structure.templates_dir)?;
    std::fs::create_dir_all(&structure.sessions_dir)?;

    Ok(structure)
}

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

/// Get export format recommendations based on intended use
pub fn get_export_format_recommendations(target: ExportTarget) -> Vec<AudioFormat> {
    match target {
        ExportTarget::LivePerformance => vec![
            AudioFormat::DecentSampler, // Free, widely supported
            AudioFormat::Wav24Bit,      // Universal compatibility
        ],
        ExportTarget::StudioProduction => vec![
            AudioFormat::Wav32BitFloat, // Maximum quality
            AudioFormat::DecentSampler, // Instant playback
            AudioFormat::SFZ,           // Professional standard
        ],
        ExportTarget::Distribution => vec![
            AudioFormat::DecentSampler, // User-friendly
            AudioFormat::SFZ,           // Professional option
            AudioFormat::Wav24Bit,      // Source files
        ],
        ExportTarget::Archival => vec![
            AudioFormat::Wav32BitFloat, // Future-proof
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_configs() {
        let audio_config = AudioSessionConfig::default();
        assert_eq!(audio_config.sample_rate, 44100);
        assert_eq!(audio_config.bit_depth, 16);
        assert_eq!(audio_config.buffer_size, 512);

        let midi_config = MidiSessionConfig::default();
        assert_eq!(midi_config.channel, 0);

        let recording_config = RecordingSessionConfig::default();
        assert_eq!(recording_config.note_duration_ms, 2500);
        assert_eq!(recording_config.release_time_ms, 1000);
    }

    #[test]
    fn test_filename_sanitization() {
        assert_eq!(sanitize_filename("My Project"), "My Project");
        assert_eq!(
            sanitize_filename("Bad/Name\\With:*?\"<>|"),
            "Bad_Name_With_______"
        );
        assert_eq!(sanitize_filename(" \t Spaces \n "), "_ Spaces _");
    }

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new();
        assert!(report.is_valid);
        assert_eq!(report.errors.len(), 0);

        report.add_error("test_field", "Test error");
        assert!(!report.is_valid);
        assert_eq!(report.errors.len(), 1);

        report.add_warning("test_field", "Test warning");
        assert_eq!(report.warnings.len(), 1);
        assert!(!report.is_valid); // Still invalid due to error
    }
}
