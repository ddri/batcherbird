//! Batcherbird Core Library
//!
//! Core library for hardware synthesizer sampling automation.

pub mod advanced_loop_detection;
pub mod advanced_sampler_formats;
pub mod audio;
pub mod audio_diagnostics;
pub mod batch_processing;
pub mod config;
pub mod crossfading;
pub mod detection;
pub mod device;
pub mod error;
pub mod export;
pub mod intelligent_detection;
pub mod lock_free_recording;
pub mod loop_detection;
pub mod midi;
pub mod playback;
pub mod professional_metadata;
pub mod professional_meters;
pub mod quality_validation;
pub mod sampler;
pub mod session;
pub mod session_manager;

pub use advanced_loop_detection::{
    AdvancedLoopConfig, AdvancedLoopDetector, AdvancedLoopResult, LoopCandidate, LoopQualityMetrics,
};
pub use advanced_sampler_formats::{
    ADSREnvelope, AdvancedInstrument, AdvancedSamplerExporter, Articulation, ArticulationTrigger,
    FilterSettings, InstrumentSettings, LoopPoints, ReleaseSample, RoundRobinGroup,
    RoundRobinProcessor, SampleInfo, VelocityLayer, VelocityLayerEngine,
};
pub use audio_diagnostics::{
    AudioDiagnostics, AudioPerformanceReport, CallbackTimer, LockTimer, PerformanceStatus,
};
pub use batch_processing::{
    BatchConfig, BatchPerformanceMetrics, BatchProcessor, BatchProgress, BatchResult,
    MemoryManager, SampleData, StreamConfig, StreamingProcessor,
};
pub use crossfading::{
    CrossfadeConfig, CrossfadeCurve, CrossfadeQuality, CrossfadeResult, EqualPowerCrossfader,
    PhaseAlignment,
};
pub use error::{BatcherbirdError, Result};
pub use intelligent_detection::{
    DetectionAlgorithm, IntelligentDetectionConfig, IntelligentDetectionResult,
    IntelligentSampleDetector, ProfessionalTrimmer, SynthesizerProfile, TrimmingConfig,
    TrimmingResult,
};
pub use lock_free_recording::{
    LockFreeRecorder, LockFreeRecordingConfig, RealtimeMeterData, RecordingPerformanceGrade,
    RecordingStats,
};
pub use professional_metadata::{
    BroadcastWavChunk, MetadataEngine, ProfessionalMetadata, QualityMetrics, SamplerCompatibility,
    SmplChunk,
};
pub use professional_meters::{
    GainRecommendation, GainStagingAnalysis, GainStagingAssistant, GainStagingStatus,
    HeadroomStatus, ProfessionalMeterEngine, ProfessionalMeterReadings,
};
pub use quality_validation::{
    AudioQualityMetrics, ProfessionalQualityValidator, QualityRecommendation, QualityThresholds,
    QualityValidationConfig, RecommendationSeverity, ValidationResult, ValidationStatus,
};
pub use sampler::{AudioLevels, LevelMeterState};
