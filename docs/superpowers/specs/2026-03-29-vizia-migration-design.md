# BatcherBird VIZIA Migration Design

## Overview

Replace the Tauri + React frontend with a native VIZIA (Rust) GUI. This is a fresh UI rethink, not a port — the layout, interaction model, and feature set are redesigned based on competitor research and user needs analysis.

The `batcherbird-core` crate is unchanged. The `batcherbird-gui` crate is replaced entirely with a new `batcherbird-vizia` crate.

## Motivation

- **Eliminate the serialization boundary**: Tauri requires JSON serialization across a webview bridge for every interaction. Real-time meter data (60fps) required a channel-based workaround to avoid crashes. With VIZIA, meter data flows directly from memory to the render loop.
- **Single language, single binary**: Remove the Node.js/React/TypeScript toolchain. One Rust binary, no webview runtime.
- **Cleaner architecture**: No 3-thread workaround for streaming data. The audio thread pushes to a ring buffer; the UI thread reads it directly.
- **Cross-platform potential**: VIZIA supports macOS, Windows, and Linux natively via Skia rendering.

## Core User Journey

Four phases, all on one screen:

1. **Connect** — Select MIDI output and audio input devices. Confirm signal is flowing. Should take 30 seconds.
2. **Configure** — Set note range, velocity layers, duration, export format. Smart defaults so this can be skipped for quick captures.
3. **Record** — Hit ARM, then record. Watch real-time meters and waveform. See progress through the batch. This is where trust is built.
4. **Export** — Review captures, apply auto-looping, export to DecentSampler/SFZ/WAV.

## Layout: Hybrid Sidebar + Clean Stage

A persistent left sidebar holds all configuration. The right "stage" area is dedicated to the recording experience. The layout does not change between states — only the stage content adapts.

### Sidebar (200px, always visible)

```
┌─────────────────────┐
│ DEVICES              │
│ MIDI Out: DW-6000  ● │
│ Audio In: Scarlett  ● │
│                      │
│ SAMPLING             │
│ ┌─────┐ ┌─────┐     │
│ │Start│ │ End │     │
│ │ C2  │ │ C5  │     │
│ └─────┘ └─────┘     │
│ ┌─────┐ ┌─────┐     │
│ │Layers│ │Dur  │     │
│ │  3  │ │2.0s │     │
│ └─────┘ └─────┘     │
│                      │
│ EXPORT               │
│ Format: DecentSampler│
│ Output: ~/Samples/   │
│                      │
│ [    Export All     ] │
└─────────────────────┘
```

Sections:
- **Devices**: MIDI output and audio input dropdowns with green signal dots showing connectivity
- **Sampling**: Start note, end note, velocity layers (1-4), note duration. Setting layers to 1 means single-velocity recording. Setting start = end means single-note recording. No mode switching needed.
- **Export**: Format picker (WAV / DecentSampler / SFZ), output directory chooser

### Stage (fills remaining width)

Content depends on app state:

**Idle state:**
- Quiet meters (showing noise floor)
- Keyboard visualization showing selected range
- Flat waveform area with summary text: "36 notes · 3 velocity layers · ~2 min total"
- ARM button (large, centered)

**Armed state:**
- Live meters (monitoring input)
- Keyboard visualization
- Flat waveform with "Monitoring — press Record when ready"
- RECORD button replaces ARM

**Recording state:**
- Active meters with dB readout
- Current note display: large note name (e.g., "E3"), velocity, layer number
- Keyboard with active note highlighted, range shown
- Live waveform filling in real-time
- Progress bar: "16 / 36 — 65%"
- Cancel button

**Review/Export state:**
- Completed waveform of last recorded sample with playback controls (play/stop/seek)
- Loop markers overlaid on waveform (if auto-loop detected)
- Summary: "Recorded 36 samples across 3 velocity layers"
- Export button becomes primary (blue, active) in sidebar

## State Machine

```
Idle → Armed → Recording → Review
 ↑                ↓          ↓
 ←── Cancel ──────┘          │
 ←────── Done ───────────────┘
```

States:
- `Idle`: No monitoring. Sidebar fully editable. Stage shows summary + ARM button.
- `Armed`: Input monitoring active. Sidebar editable. Stage shows live meters + RECORD button.
- `Recording`: Batch recording in progress. Sidebar locked (grayed out). Stage shows live waveform + progress.
- `Review`: Recording complete. Sidebar unlocked. Stage shows recorded samples + export controls.

## Architecture

### Crate Structure

```
batcherbird/
├── crates/
│   ├── batcherbird-core/      (unchanged — audio engine, MIDI, detection, export)
│   ├── batcherbird-cli/       (unchanged — diagnostic tool)
│   └── batcherbird-vizia/     (NEW — replaces batcherbird-gui entirely)
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs        (app entry, VIZIA Application setup)
│       │   ├── app_data.rs    (AppData model — all shared state)
│       │   ├── app_event.rs   (AppEvent enum — all user actions)
│       │   ├── views/
│       │   │   ├── sidebar.rs        (device, sampling, export panels)
│       │   │   ├── stage.rs          (main stage container, delegates to state views)
│       │   │   ├── meters.rs         (peak/RMS meter bars)
│       │   │   ├── waveform.rs       (custom-drawn waveform display)
│       │   │   ├── keyboard.rs       (piano keyboard range visualization)
│       │   │   ├── note_display.rs   (current note + velocity + layer)
│       │   │   └── progress.rs       (recording progress bar)
│       │   └── style/
│       │       └── theme.css         (VIZIA CSS stylesheet)
│       └── resources/
│           └── icons/
```

### Data Flow (replaces Tauri commands + events)

**Device enumeration:**
```
AppEvent::RefreshDevices
  → app_data calls batcherbird_core::MidiManager::list_devices()
  → app_data calls batcherbird_core::AudioManager::list_input_devices()
  → VIZIA lens updates sidebar dropdowns
```

**Real-time meters (replaces Tauri channel workaround):**
```
Audio callback (CPAL)
  → calculates peak/RMS
  → pushes to rtrb ring buffer (existing pattern)

VIZIA timer (16ms / 60fps)
  → pops from ring buffer
  → updates AppData meter fields
  → meters view redraws via lens binding
```

No serialization. No channels. No webview. The ring buffer is read directly by the UI thread via a VIZIA timer callback.

**Recording:**
```
AppEvent::StartRecording
  → spawns recording on background thread (tokio::spawn_blocking)
  → recording thread pushes VizChunks to ring buffer (existing pattern)
  → recording thread sends progress updates via crossbeam channel
  → VIZIA timer polls both ring buffer (waveform) and channel (progress)
  → on completion, sends AppEvent::RecordingComplete with results
```

**Waveform rendering:**
Custom VIZIA view using the Canvas drawing API. Draws waveform from VizChunk data during recording, switches to file-based waveform data after recording completes.

### AppData Model

```rust
pub struct AppData {
    // Devices
    pub midi_devices: Vec<String>,
    pub audio_input_devices: Vec<String>,
    pub selected_midi_device: Option<usize>,
    pub selected_audio_input: Option<usize>,
    pub midi_connected: bool,
    pub audio_connected: bool,

    // Sampling config
    pub start_note: u8,       // default: 36 (C2)
    pub end_note: u8,         // default: 84 (C5)
    pub velocity_layers: u8,  // 1-4, default: 1
    pub note_duration_ms: u32, // default: 2000

    // Export config
    pub export_format: ExportFormat,  // WAV | DecentSampler | SFZ
    pub output_directory: PathBuf,

    // State
    pub app_state: AppState,  // Idle | Armed | Recording | Review

    // Real-time data
    pub meter_left: f32,      // 0.0-1.0 linear
    pub meter_right: f32,
    pub meter_left_db: f32,
    pub meter_right_db: f32,

    // Recording progress
    pub current_note: u8,
    pub current_velocity: u8,
    pub current_layer: u8,
    pub total_layers: u8,
    pub notes_completed: u32,
    pub notes_total: u32,

    // Waveform
    pub viz_chunks: Vec<VizChunk>,     // live recording data
    pub waveform_data: Option<Vec<f32>>, // file-based after recording

    // Results
    pub recorded_samples: Vec<RecordedSample>,

    // Engine handles (not lensed)
    pub midi_manager: Option<MidiManager>,
    pub sampling_engine: Option<SamplingEngine>,
    pub meter_consumer: Option<rtrb::Consumer<RealtimeMeterData>>,
}
```

### AppEvent Enum

```rust
pub enum AppEvent {
    // Devices
    RefreshDevices,
    SelectMidiDevice(usize),
    SelectAudioInput(usize),
    TestMidiConnection,

    // Config
    SetStartNote(u8),
    SetEndNote(u8),
    SetVelocityLayers(u8),
    SetDuration(u32),
    SetExportFormat(ExportFormat),
    SelectOutputDirectory,

    // Recording
    Arm,
    Disarm,
    StartRecording,
    CancelRecording,
    RecordingProgress { note: u8, velocity: u8, layer: u8, completed: u32, total: u32 },
    RecordingComplete(Vec<RecordedSample>),

    // Playback
    PlaySample(usize),
    StopPlayback,

    // Export
    ExportAll,

    // Timer
    Tick, // 60fps timer for meter + waveform updates
}
```

## Features Included

- Device selection with signal confirmation (green dots)
- Single note recording (set start = end)
- Range recording (set start and end notes)
- Velocity layers (1-4)
- Real-time peak/RMS meters
- Live waveform during recording
- File-based waveform after recording
- Piano keyboard visualization with range + active note
- Progress tracking for batch operations
- Auto-loop detection
- Audio playback (preview recorded samples)
- Export to WAV, DecentSampler, SFZ
- Cancel recording mid-batch
- Native file dialogs (via rfd crate)

## Features Explicitly Cut

- Session initialization wizard (replaced by always-visible sidebar)
- Session recovery system (future enhancement)
- Professional meters (VU/PPM/LUFS) — keep simple peak/RMS only
- Quality validation dashboard
- Gain staging assistant
- Tab-based recording modes (unified into sidebar config fields)
- Tauri plugin ecosystem (dialog, log — replaced with native Rust equivalents)
- Web-based rendering entirely

## Dependencies (new crate)

```toml
[dependencies]
batcherbird-core = { path = "../batcherbird-core" }
vizia = "0.3"
rtrb = "0.3"             # lock-free ring buffer (already in workspace)
cpal = "0.16"            # audio device enumeration
midir = "0.10"           # MIDI device enumeration
crossbeam-channel = "0.5" # progress updates from recording thread
rfd = "0.15"             # native file dialogs
tokio = { version = "1.35", features = ["rt-multi-thread"] }
serde = { version = "1.0", features = ["derive"] }
```

## CSS Theming

VIZIA supports CSS with hot-reloading. The app uses a dark theme matching the mockup:

- Background: `#0e0e15` (sidebar), `#111118` (stage)
- Text: `#e0e0e0` (primary), `#888` (secondary), `#555` (tertiary)
- Accent: `#4a9eff` (active elements, waveform)
- Record: `#e53935` (recording indicator, ARM button border)
- Signal: `#28c840` (connected devices)
- Meters: gradient from `#28c840` → `#febc2e` → `#e53935`

Hot-reloading allows rapid design iteration without recompiling.

## What Happens to the Old GUI

The `crates/batcherbird-gui/` directory (Tauri + React) is removed from the project after the VIZIA crate is functional. The workspace `Cargo.toml` is updated to include `batcherbird-vizia` as a member.

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| VIZIA is a smaller ecosystem than React | The UI is simple enough (sidebar + waveform + meters) that we won't hit edge cases |
| Custom waveform rendering | VIZIA has Canvas drawing API; same approach as the current React canvas code |
| Native file dialogs | rfd crate is mature and cross-platform |
| VIZIA performance for 60fps meters | Timer-based polling of ring buffer is lightweight; Skia rendering handles this easily |
| Learning curve | VIZIA's declarative model is similar to React; CSS theming is familiar |
