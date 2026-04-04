use batcherbird_core::export::AudioFormat;
use batcherbird_core::sampler::VizChunk;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum AppEvent {
    // Devices
    RefreshDevices,
    SelectMidiDevice(usize),
    SelectAudioInput(usize),

    // Config
    SetStartNote(u8),
    SetEndNote(u8),
    SetVelocityLayers(u8),
    SetDuration(u32),
    SetExportFormat(AudioFormat),
    SetOutputDirectory(PathBuf),
    SelectOutputDirectory,

    // Recording lifecycle
    Arm,
    Disarm,
    StartRecording,
    CancelRecording,

    // Internal events (from background threads)
    RecordingProgress {
        note: u8,
        velocity: u8,
        layer: u8,
        completed: u32,
        total: u32,
    },
    RecordingComplete,
    RecordingError(String),

    // Playback
    PlaySample(usize),
    StopPlayback,

    // Export
    ExportAll,

    // Visualization data
    PushVizChunk(VizChunk),

    // Timer tick
    Tick,
}
