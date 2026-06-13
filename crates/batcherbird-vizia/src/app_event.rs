use batcherbird_core::export::AudioFormat;
use batcherbird_core::sampler::VizChunk;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum AppEvent {
    // Devices
    RefreshDevices,
    SelectMidiDevice(usize),
    SelectAudioInput(usize),
    CycleNextMidiDevice,
    CycleNextAudioInput,

    // Config
    SetStartNote(u8),
    SetEndNote(u8),
    SetVelocityLayers(u8),
    SetDuration(u32),
    SetExportFormat(AudioFormat),
    SetOutputDirectory(PathBuf),
    SelectOutputDirectory,
    CycleExportFormat,
    CycleExportFormatBack,
    CyclePrevMidiDevice,
    CyclePrevAudioInput,
    SelectFormatByIndex(usize),

    // Increment/decrement events
    IncrementStartNote,
    DecrementStartNote,
    IncrementEndNote,
    DecrementEndNote,
    IncrementVelocityLayers,
    DecrementVelocityLayers,
    IncrementDuration,
    DecrementDuration,

    // Recording lifecycle
    Arm,
    Disarm,
    StartRecording,
    CancelRecording,

    // Internal events (from background threads)
    RecordingProgress {
        generation: u64,
        note: u8,
        velocity: u8,
        layer: u8,
        total_layers: u8,
        completed: u32,
        total: u32,
    },
    RecordingFinished {
        generation: u64,
    },
    RecordingError {
        generation: u64,
        message: String,
    },

    // Error handling
    DismissError,

    // Playback
    PlaySample(usize),
    StopPlayback,

    // Review preview controls
    PlayPreview,
    PausePreview,
    StopPreview,

    // Export
    ExportAll,
    ExportComplete {
        count: usize,
        directory: PathBuf,
    },
    ExportError(String),

    // Visualization data
    PushVizChunk(VizChunk),

    // Timer tick
    Tick,
}
