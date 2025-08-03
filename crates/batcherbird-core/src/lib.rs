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
pub mod playback;
pub mod professional_meters;

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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}