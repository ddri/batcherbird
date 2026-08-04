//! Batcherbird Core Library
//!
//! Core library for hardware synthesizer sampling automation.

pub mod audio;
pub mod audio_diagnostics;
pub mod detection;
pub mod error;
pub mod export;
pub mod lock_free_recording;
pub mod loop_detection;
pub mod midi;
pub mod preview_player;
pub mod professional_meters;
pub mod sampler;

pub use audio_diagnostics::{
    AudioDiagnostics, AudioPerformanceReport, CallbackTimer, LockTimer, PerformanceStatus,
};
pub use error::{BatcherbirdError, Result};
pub use lock_free_recording::{LockFreeRecorder, LockFreeRecordingConfig, RealtimeMeterData};
pub use preview_player::PreviewPlayer;
pub use professional_meters::{
    GainRecommendation, GainStagingAnalysis, GainStagingAssistant, GainStagingStatus,
    HeadroomStatus, ProfessionalMeterEngine, ProfessionalMeterReadings,
};
pub use sampler::{AudioLevels, LevelMeterState};
