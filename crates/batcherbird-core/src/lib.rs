//! Batcherbird Core Library
//! 
//! Core library for hardware synthesizer sampling automation.

pub mod error;
pub mod midi;
pub mod audio;
pub mod device;
pub mod session;
pub mod session_manager;
pub mod config;
pub mod sampler;
pub mod export;
pub mod detection;
pub mod intelligent_detection;
pub mod loop_detection;
pub mod advanced_loop_detection;
pub mod professional_metadata;
pub mod crossfading;
pub mod batch_processing;
pub mod advanced_sampler_formats;
pub mod quality_validation;
pub mod playback;
pub mod professional_meters;
pub mod audio_diagnostics;
pub mod lock_free_recording;

pub use error::{BatcherbirdError, Result};
pub use sampler::{AudioLevels, LevelMeterState};
pub use professional_meters::{
    ProfessionalMeterEngine, ProfessionalMeterReadings, GainStagingStatus,
    GainStagingAssistant, GainStagingAnalysis, GainRecommendation, HeadroomStatus
};
pub use intelligent_detection::{
    IntelligentSampleDetector, IntelligentDetectionConfig, IntelligentDetectionResult,
    DetectionAlgorithm, SynthesizerProfile, ProfessionalTrimmer, TrimmingConfig, TrimmingResult
};
pub use advanced_loop_detection::{
    AdvancedLoopDetector, AdvancedLoopConfig, AdvancedLoopResult, LoopCandidate, LoopQualityMetrics
};
pub use professional_metadata::{
    ProfessionalMetadata, MetadataEngine, SmplChunk, BroadcastWavChunk, QualityMetrics, SamplerCompatibility
};
pub use crossfading::{
    EqualPowerCrossfader, CrossfadeConfig, CrossfadeResult, CrossfadeQuality, CrossfadeCurve, PhaseAlignment
};
pub use batch_processing::{
    BatchProcessor, BatchConfig, BatchResult, BatchProgress, SampleData, BatchPerformanceMetrics,
    StreamingProcessor, StreamConfig, MemoryManager
};
pub use advanced_sampler_formats::{
    AdvancedInstrument, AdvancedSamplerExporter, VelocityLayer, VelocityLayerEngine, 
    RoundRobinGroup, RoundRobinProcessor, ReleaseSample, Articulation, ArticulationTrigger,
    LoopPoints, SampleInfo, InstrumentSettings, ADSREnvelope, FilterSettings
};
pub use quality_validation::{
    ProfessionalQualityValidator, QualityValidationConfig, QualityThresholds, ValidationResult,
    ValidationStatus, AudioQualityMetrics, QualityRecommendation, RecommendationSeverity
};
pub use audio_diagnostics::{
    AudioDiagnostics, AudioPerformanceReport, CallbackTimer, LockTimer, PerformanceStatus
};
pub use lock_free_recording::{
    LockFreeRecorder, LockFreeRecordingConfig, RecordingStats, RecordingPerformanceGrade
};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}