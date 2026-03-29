use std::path::PathBuf;
use batcherbird_core::export::AudioFormat;

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

    // Timer tick
    Tick,
}
