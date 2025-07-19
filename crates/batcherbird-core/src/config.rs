use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiConfig {
    pub device_name: String,
    pub channel: u8,
    pub program_change_delay_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub device_name: String,
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub channels: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    pub note_range: NoteRange,
    pub velocities: Vec<u8>,
    pub note_duration_ms: u32,
    pub release_time_ms: u32,
    pub pre_delay_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteRange {
    pub start: u8,
    pub end: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub midi: MidiConfig,
    pub audio: AudioConfig,
    pub sampling: SamplingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            midi: MidiConfig {
                device_name: String::new(),
                channel: 1,
                program_change_delay_ms: 50,
            },
            audio: AudioConfig {
                device_name: String::new(),
                sample_rate: 48000,
                bit_depth: 24,
                channels: 2,
            },
            sampling: SamplingConfig {
                note_range: NoteRange { start: 36, end: 84 }, // C2 to C6
                velocities: vec![64, 96, 127],
                note_duration_ms: 2000,
                release_time_ms: 1000,
                pre_delay_ms: 100,
            },
        }
    }
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}