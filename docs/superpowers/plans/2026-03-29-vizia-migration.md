# BatcherBird VIZIA Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Tauri + React frontend with a native VIZIA GUI, delivering a cleaner single-binary Rust application with the hybrid sidebar + stage layout.

**Architecture:** New `batcherbird-vizia` crate joins the workspace. It depends on `batcherbird-core` for all audio/MIDI/export logic. VIZIA provides the GUI with CSS theming and Skia rendering. Real-time meter data flows from the audio thread via rtrb ring buffers directly to the UI thread — no serialization boundary.

**Tech Stack:** Rust, VIZIA 0.3, rtrb (ring buffers), cpal (audio), midir (MIDI), rfd (file dialogs), crossbeam-channel (thread communication)

**Spec:** `docs/superpowers/specs/2026-03-29-vizia-migration-design.md`

**Scope:** This plan covers the Connect → Configure → Record flow (Tasks 1-15). The Review/Export phase (auto-loop detection, audio playback, export wiring, file-based waveform) is a follow-up plan — the core engine already has these features, they just need UI wiring.

---

## File Structure

```
crates/batcherbird-vizia/
├── Cargo.toml
├── src/
│   ├── main.rs              # App entry, window setup, timer registration
│   ├── app_data.rs           # AppData model (state), AppState enum
│   ├── app_event.rs          # AppEvent enum (all user actions + internal events)
│   ├── views/
│   │   ├── mod.rs            # Re-exports all views
│   │   ├── sidebar.rs        # Left sidebar: devices, sampling config, export
│   │   ├── stage.rs          # Main stage container, delegates to state-specific content
│   │   ├── meters.rs         # Peak/RMS horizontal meter bars (custom drawn)
│   │   ├── waveform.rs       # Waveform display (custom drawn via Canvas)
│   │   ├── keyboard.rs       # Piano keyboard range visualization (custom drawn)
│   │   ├── note_display.rs   # Current note name + velocity + layer info
│   │   └── progress.rs       # Recording progress bar
│   └── style/
│       └── theme.css         # Dark theme CSS
├── resources/
│   └── (empty for now)
└── tests/
    └── app_data_test.rs      # State machine + model tests
```

---

### Task 1: Scaffold the crate and get a window on screen

**Files:**
- Create: `crates/batcherbird-vizia/Cargo.toml`
- Create: `crates/batcherbird-vizia/src/main.rs`
- Modify: `Cargo.toml` (workspace members)

- [ ] **Step 1: Create the Cargo.toml**

```toml
[package]
name = "batcherbird-vizia"
version = "0.1.0"
edition = "2021"
description = "BatcherBird native GUI built with VIZIA"
license.workspace = true
repository.workspace = true
authors.workspace = true

[dependencies]
batcherbird-core = { path = "../batcherbird-core" }
vizia = "0.3"
rtrb = "0.3"
cpal = { workspace = true }
midir = { workspace = true }
serde = { workspace = true }
crossbeam-channel = "0.5"
rfd = "0.15"
```

- [ ] **Step 2: Create minimal main.rs**

```rust
use vizia::prelude::*;

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        Label::new(cx, "BatcherBird");
    })
    .title("BatcherBird")
    .inner_size((900, 550))
    .run()
}
```

- [ ] **Step 3: Add crate to workspace**

In the root `Cargo.toml`, add `"crates/batcherbird-vizia"` to the `members` array.

- [ ] **Step 4: Build and run**

Run: `cargo run -p batcherbird-vizia`
Expected: A window appears with "BatcherBird" label. No errors.

- [ ] **Step 5: Commit**

```bash
git add crates/batcherbird-vizia/ Cargo.toml
git commit -m "feat: scaffold batcherbird-vizia crate with empty window"
```

---

### Task 2: Define AppData model and AppEvent enum

**Files:**
- Create: `crates/batcherbird-vizia/src/app_data.rs`
- Create: `crates/batcherbird-vizia/src/app_event.rs`
- Create: `crates/batcherbird-vizia/tests/app_data_test.rs`
- Modify: `crates/batcherbird-vizia/src/main.rs`

- [ ] **Step 1: Write state machine tests**

Create `crates/batcherbird-vizia/tests/app_data_test.rs`:

```rust
use batcherbird_vizia::app_data::{AppData, AppState};
use batcherbird_vizia::app_event::AppEvent;

#[test]
fn initial_state_is_idle() {
    let data = AppData::default();
    assert!(matches!(data.app_state, AppState::Idle));
}

#[test]
fn default_sampling_config() {
    let data = AppData::default();
    assert_eq!(data.start_note, 36); // C2
    assert_eq!(data.end_note, 84);   // C5
    assert_eq!(data.velocity_layers, 1);
    assert_eq!(data.note_duration_ms, 2000);
}

#[test]
fn total_notes_calculation() {
    let mut data = AppData::default();
    data.start_note = 60; // C4
    data.end_note = 72;   // C5
    data.velocity_layers = 3;
    // 13 notes (C4 to C5 inclusive) * 3 layers = 39
    assert_eq!(data.total_samples(), 39);
}

#[test]
fn single_note_mode() {
    let mut data = AppData::default();
    data.start_note = 60;
    data.end_note = 60;
    data.velocity_layers = 1;
    assert_eq!(data.total_samples(), 1);
}

#[test]
fn note_to_name_conversion() {
    assert_eq!(AppData::note_name(60), "C4");
    assert_eq!(AppData::note_name(69), "A4");
    assert_eq!(AppData::note_name(36), "C2");
    assert_eq!(AppData::note_name(84), "C6");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p batcherbird-vizia`
Expected: Compilation errors — modules don't exist yet.

- [ ] **Step 3: Create app_event.rs**

```rust
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
    SelectOutputDirectory, // opens native dialog

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

    // Timer tick (60fps meter + waveform updates)
    Tick,
}
```

- [ ] **Step 4: Create app_data.rs**

```rust
use std::path::PathBuf;
use vizia::prelude::*;
use batcherbird_core::export::AudioFormat;
use batcherbird_core::sampler::VizChunk;
use crate::app_event::AppEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Idle,
    Armed,
    Recording,
    Review,
}

#[derive(Lens)]
pub struct AppData {
    // Devices
    pub midi_devices: Vec<String>,
    pub audio_input_devices: Vec<String>,
    pub selected_midi_device: Option<usize>,
    pub selected_audio_input: Option<usize>,
    pub midi_connected: bool,
    pub audio_connected: bool,

    // Sampling config
    pub start_note: u8,
    pub end_note: u8,
    pub velocity_layers: u8,
    pub note_duration_ms: u32,

    // Export config
    pub export_format: AudioFormat,
    pub output_directory: PathBuf,

    // App state
    pub app_state: AppState,

    // Real-time meters
    pub meter_left: f32,
    pub meter_right: f32,
    pub meter_left_db: f32,
    pub meter_right_db: f32,
    pub is_clipping: bool,

    // Recording progress
    pub current_note: u8,
    pub current_velocity: u8,
    pub current_layer: u8,
    pub total_layers: u8,
    pub notes_completed: u32,
    pub notes_total: u32,

    // Waveform data
    pub viz_chunks: Vec<VizChunk>,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            midi_devices: Vec::new(),
            audio_input_devices: Vec::new(),
            selected_midi_device: None,
            selected_audio_input: None,
            midi_connected: false,
            audio_connected: false,

            start_note: 36,       // C2
            end_note: 84,         // C6
            velocity_layers: 1,
            note_duration_ms: 2000,

            export_format: AudioFormat::Wav24Bit,
            output_directory: dirs::document_dir()
                .unwrap_or_else(|| PathBuf::from(".")),

            app_state: AppState::Idle,

            meter_left: 0.0,
            meter_right: 0.0,
            meter_left_db: -60.0,
            meter_right_db: -60.0,
            is_clipping: false,

            current_note: 0,
            current_velocity: 0,
            current_layer: 0,
            total_layers: 0,
            notes_completed: 0,
            notes_total: 0,

            viz_chunks: Vec::new(),
        }
    }
}

impl AppData {
    pub fn total_samples(&self) -> u32 {
        let num_notes = (self.end_note as u32)
            .saturating_sub(self.start_note as u32) + 1;
        num_notes * self.velocity_layers as u32
    }

    pub fn note_name(note: u8) -> String {
        let names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let octave = (note as i8 / 12) - 2;
        let index = (note % 12) as usize;
        format!("{}{}", names[index], octave)
    }

    pub fn estimated_duration_secs(&self) -> f32 {
        let total = self.total_samples() as f32;
        let per_note_secs = self.note_duration_ms as f32 / 1000.0 + 1.5; // note + release + delay
        total * per_note_secs
    }
}

impl Model for AppData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event: &AppEvent, _| match app_event {
            AppEvent::SetStartNote(n) => self.start_note = *n,
            AppEvent::SetEndNote(n) => self.end_note = *n,
            AppEvent::SetVelocityLayers(n) => self.velocity_layers = *n,
            AppEvent::SetDuration(ms) => self.note_duration_ms = *ms,
            AppEvent::SetExportFormat(fmt) => self.export_format = fmt.clone(),
            AppEvent::SetOutputDirectory(path) => self.output_directory = path.clone(),

            AppEvent::Arm => {
                if self.app_state == AppState::Idle {
                    self.app_state = AppState::Armed;
                }
            }
            AppEvent::Disarm => {
                if self.app_state == AppState::Armed {
                    self.app_state = AppState::Idle;
                }
            }
            AppEvent::StartRecording => {
                if self.app_state == AppState::Armed {
                    self.app_state = AppState::Recording;
                    self.notes_total = self.total_samples();
                    self.notes_completed = 0;
                    self.viz_chunks.clear();
                }
            }
            AppEvent::CancelRecording => {
                self.app_state = AppState::Idle;
            }
            AppEvent::RecordingProgress { note, velocity, layer, completed, total } => {
                self.current_note = *note;
                self.current_velocity = *velocity;
                self.current_layer = *layer;
                self.notes_completed = *completed;
                self.notes_total = *total;
            }
            AppEvent::RecordingComplete => {
                self.app_state = AppState::Review;
            }
            AppEvent::RecordingError(_msg) => {
                self.app_state = AppState::Idle;
            }

            // Device and other events handled in later tasks
            _ => {}
        });
    }
}
```

- [ ] **Step 5: Update main.rs to expose modules as a library**

Add `crates/batcherbird-vizia/src/lib.rs`:

```rust
pub mod app_data;
pub mod app_event;
```

Update `Cargo.toml` to add:

```toml
[[bin]]
name = "batcherbird-vizia"
path = "src/main.rs"

[lib]
name = "batcherbird_vizia"
path = "src/lib.rs"

[dependencies]
dirs = "5.0"
```

Update `main.rs`:

```rust
use vizia::prelude::*;
use batcherbird_vizia::app_data::AppData;

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        AppData::default().build(cx);
        Label::new(cx, "BatcherBird");
    })
    .title("BatcherBird")
    .inner_size((900, 550))
    .run()
}
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p batcherbird-vizia`
Expected: All 5 tests pass.

- [ ] **Step 7: Build and run the app**

Run: `cargo run -p batcherbird-vizia`
Expected: Window appears, no errors.

- [ ] **Step 8: Commit**

```bash
git add crates/batcherbird-vizia/
git commit -m "feat: add AppData model and AppEvent enum with state machine"
```

---

### Task 3: Create the CSS theme

**Files:**
- Create: `crates/batcherbird-vizia/src/style/theme.css`
- Modify: `crates/batcherbird-vizia/src/main.rs`

- [ ] **Step 1: Create theme.css**

```css
/* BatcherBird Dark Theme */

* {
    font-family: system-ui, -apple-system, sans-serif;
    color: #e0e0e0;
}

:root {
    background-color: #111118;
}

.sidebar {
    background-color: #0e0e15;
    width: 200px;
    child-space: 0px;
}

.sidebar-section {
    child-left: 14px;
    child-right: 14px;
    child-top: 12px;
    child-bottom: 4px;
}

.sidebar-label {
    font-size: 10;
    color: #555555;
    child-bottom: 8px;
}

.device-row {
    child-bottom: 8px;
}

.device-type {
    font-size: 10;
    color: #666666;
}

.device-name {
    font-size: 12;
    color: #cccccc;
}

.signal-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background-color: #28c840;
}

.signal-dot.disconnected {
    background-color: #444444;
}

.field-box {
    background-color: #161620;
    border-color: #252530;
    border-width: 1px;
    border-radius: 4px;
    child-space: 6px;
    col-between: 4px;
}

.field-label {
    font-size: 9;
    color: #555555;
}

.field-value {
    font-size: 14;
    color: #dddddd;
}

.stage {
    child-left: 16px;
    child-right: 16px;
    child-top: 16px;
    child-bottom: 16px;
    row-between: 12px;
}

.meter-label {
    font-size: 10;
    color: #555555;
    width: 12px;
}

.meter-db {
    font-size: 10;
    color: #666666;
    width: 45px;
}

.note-name {
    font-size: 32;
    color: #ffffff;
}

.note-detail {
    font-size: 13;
    color: #666666;
}

.rec-indicator {
    background-color: #2a1015;
    border-color: #3a1520;
    border-width: 1px;
    border-radius: 6px;
    child-space: 6px;
}

.rec-text {
    font-size: 12;
    color: #e53935;
}

.btn-arm {
    border-color: #e5393555;
    border-width: 2px;
    border-radius: 8px;
    color: #e53935;
    font-size: 16;
    child-space: 12px;
    child-left: 48px;
    child-right: 48px;
    background-color: #1a1a28;
}

.btn-arm:hover {
    background-color: #251520;
}

.btn-record {
    background-color: #e53935;
    color: #ffffff;
    border-radius: 6px;
    font-size: 13;
    child-space: 8px;
    child-left: 20px;
    child-right: 20px;
}

.btn-export {
    background-color: #1a1a28;
    border-color: #333333;
    border-width: 1px;
    border-radius: 5px;
    color: #666666;
    font-size: 12;
    child-space: 8px;
}

.btn-export.ready {
    background-color: #4a9eff;
    border-color: #4a9eff;
    color: #ffffff;
}

.progress-text {
    font-size: 12;
    color: #888888;
}

.idle-text {
    font-size: 13;
    color: #444444;
}

.idle-subtext {
    font-size: 11;
    color: #333333;
}

.divider {
    height: 1px;
    background-color: #1e1e28;
}
```

- [ ] **Step 2: Load the stylesheet in main.rs**

```rust
use vizia::prelude::*;
use batcherbird_vizia::app_data::AppData;

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        cx.add_stylesheet(include_style!("src/style/theme.css"))
            .expect("Failed to load theme");

        AppData::default().build(cx);

        HStack::new(cx, |cx| {
            // Sidebar placeholder
            VStack::new(cx, |cx| {
                Label::new(cx, "DEVICES").class("sidebar-label");
            })
            .class("sidebar");

            // Stage placeholder
            VStack::new(cx, |cx| {
                Label::new(cx, "BatcherBird").class("note-name");
            })
            .class("stage");
        });
    })
    .title("BatcherBird")
    .inner_size((900, 550))
    .run()
}
```

- [ ] **Step 3: Build and run**

Run: `cargo run -p batcherbird-vizia`
Expected: Window with dark background, sidebar on left (dark), stage on right. "DEVICES" label in gray, "BatcherBird" in large white text.

- [ ] **Step 4: Commit**

```bash
git add crates/batcherbird-vizia/src/style/
git commit -m "feat: add dark theme CSS for BatcherBird VIZIA app"
```

---

### Task 4: Build the sidebar view

**Files:**
- Create: `crates/batcherbird-vizia/src/views/mod.rs`
- Create: `crates/batcherbird-vizia/src/views/sidebar.rs`
- Modify: `crates/batcherbird-vizia/src/lib.rs`
- Modify: `crates/batcherbird-vizia/src/main.rs`

- [ ] **Step 1: Create views/mod.rs**

```rust
mod sidebar;
pub use sidebar::*;
```

- [ ] **Step 2: Create sidebar.rs**

```rust
use vizia::prelude::*;
use crate::app_data::AppData;
use crate::app_event::AppEvent;

pub fn sidebar(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // === DEVICES ===
        VStack::new(cx, |cx| {
            Label::new(cx, "DEVICES").class("sidebar-label");

            // MIDI Out
            VStack::new(cx, |cx| {
                Label::new(cx, "MIDI Out").class("device-type");
                Dropdown::new(
                    cx,
                    move |cx|
                        Label::new(cx, AppData::midi_devices.map(|devs| {
                            if devs.is_empty() { "No MIDI devices".to_string() }
                            else { devs[0].clone() }
                        })).class("device-name"),
                    move |cx| {
                        List::new(cx, AppData::midi_devices, |cx, index, item| {
                            Label::new(cx, item)
                                .class("device-name")
                                .on_press(move |cx| {
                                    cx.emit(AppEvent::SelectMidiDevice(index));
                                    cx.emit(PopupEvent::Close);
                                });
                        });
                    },
                );
            })
            .class("device-row");

            // Audio In
            VStack::new(cx, |cx| {
                Label::new(cx, "Audio In").class("device-type");
                Dropdown::new(
                    cx,
                    move |cx|
                        Label::new(cx, AppData::audio_input_devices.map(|devs| {
                            if devs.is_empty() { "No audio devices".to_string() }
                            else { devs[0].clone() }
                        })).class("device-name"),
                    move |cx| {
                        List::new(cx, AppData::audio_input_devices, |cx, index, item| {
                            Label::new(cx, item)
                                .class("device-name")
                                .on_press(move |cx| {
                                    cx.emit(AppEvent::SelectAudioInput(index));
                                    cx.emit(PopupEvent::Close);
                                });
                        });
                    },
                );
            })
            .class("device-row");
        })
        .class("sidebar-section");

        Element::new(cx).class("divider");

        // === SAMPLING ===
        VStack::new(cx, |cx| {
            Label::new(cx, "SAMPLING").class("sidebar-label");

            HStack::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    Label::new(cx, "START").class("field-label");
                    Label::new(cx, AppData::start_note.map(|n| AppData::note_name(*n)))
                        .class("field-value");
                })
                .class("field-box");

                VStack::new(cx, |cx| {
                    Label::new(cx, "END").class("field-label");
                    Label::new(cx, AppData::end_note.map(|n| AppData::note_name(*n)))
                        .class("field-value");
                })
                .class("field-box");
            })
            .col_between(Pixels(8.0));

            HStack::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    Label::new(cx, "LAYERS").class("field-label");
                    Label::new(cx, AppData::velocity_layers.map(|n| n.to_string()))
                        .class("field-value");
                })
                .class("field-box");

                VStack::new(cx, |cx| {
                    Label::new(cx, "DURATION").class("field-label");
                    Label::new(cx, AppData::note_duration_ms.map(|ms| {
                        format!("{:.1}s", *ms as f32 / 1000.0)
                    }))
                    .class("field-value");
                })
                .class("field-box");
            })
            .col_between(Pixels(8.0));
        })
        .class("sidebar-section");

        Element::new(cx).class("divider");

        // === EXPORT ===
        VStack::new(cx, |cx| {
            Label::new(cx, "EXPORT").class("sidebar-label");

            VStack::new(cx, |cx| {
                Label::new(cx, "FORMAT").class("field-label");
                Label::new(cx, AppData::export_format.map(|fmt| format!("{:?}", fmt)))
                    .class("field-value");
            })
            .class("field-box");

            VStack::new(cx, |cx| {
                Label::new(cx, "OUTPUT").class("field-label");
                Label::new(cx, AppData::output_directory.map(|p| {
                    p.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| p.to_string_lossy().to_string())
                }))
                .class("field-value")
                .on_press(|cx| cx.emit(AppEvent::SelectOutputDirectory));
            })
            .class("field-box");
        })
        .class("sidebar-section");

        // Export button at bottom
        Button::new(cx, |cx| Label::new(cx, "Export All"))
            .class("btn-export")
            .on_press(|cx| cx.emit(AppEvent::ExportAll));
    })
    .class("sidebar");
}
```

- [ ] **Step 3: Update lib.rs**

```rust
pub mod app_data;
pub mod app_event;
pub mod views;
```

- [ ] **Step 4: Update main.rs to use sidebar**

```rust
use vizia::prelude::*;
use batcherbird_vizia::app_data::AppData;
use batcherbird_vizia::views::sidebar;

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        cx.add_stylesheet(include_style!("src/style/theme.css"))
            .expect("Failed to load theme");

        AppData::default().build(cx);

        HStack::new(cx, |cx| {
            sidebar(cx);

            // Stage placeholder
            VStack::new(cx, |cx| {
                Label::new(cx, "Stage area").class("idle-text");
            })
            .class("stage");
        });
    })
    .title("BatcherBird")
    .inner_size((900, 550))
    .run()
}
```

- [ ] **Step 5: Build and run**

Run: `cargo run -p batcherbird-vizia`
Expected: Sidebar visible on left with DEVICES, SAMPLING, EXPORT sections. Default values shown (C2, C6, 1 layer, 2.0s). Stage area on right.

Note: Dropdowns may need VIZIA API adjustments — if `Dropdown` isn't the exact API, use `PickList` or `ComboBox` as available. Check VIZIA docs and adjust. The important thing is the layout renders.

- [ ] **Step 6: Commit**

```bash
git add crates/batcherbird-vizia/src/views/
git commit -m "feat: add sidebar view with device, sampling, and export panels"
```

---

### Task 5: Build custom meter view

**Files:**
- Create: `crates/batcherbird-vizia/src/views/meters.rs`
- Modify: `crates/batcherbird-vizia/src/views/mod.rs`

- [ ] **Step 1: Create meters.rs with custom drawing**

```rust
use vizia::prelude::*;
use vizia::vg;
use crate::app_data::AppData;

pub struct MeterBar {
    level_lens: Box<dyn Lens<Target = f32>>,
    db_lens: Box<dyn Lens<Target = f32>>,
    label: String,
}

pub fn meters(cx: &mut Context) {
    HStack::new(cx, |cx| {
        VStack::new(cx, |cx| {
            meter_row(cx, "L", AppData::meter_left, AppData::meter_left_db);
            meter_row(cx, "R", AppData::meter_right, AppData::meter_right_db);
        })
        .row_between(Pixels(3.0));
    });
}

fn meter_row(cx: &mut Context, label: &str, level: impl Lens<Target = f32> + Copy, db: impl Lens<Target = f32> + Copy) {
    HStack::new(cx, |cx| {
        Label::new(cx, label).class("meter-label");

        MeterBarView::new(cx, level)
            .height(Pixels(5.0));

        Label::new(cx, db.map(|v| format!("{:.1} dB", v)))
            .class("meter-db");
    })
    .col_between(Pixels(6.0))
    .height(Auto);
}

struct MeterBarView<L: Lens<Target = f32>> {
    level: L,
}

impl<L: Lens<Target = f32>> MeterBarView<L> {
    fn new(cx: &mut Context, level: L) -> Handle<'_, Self> {
        Self { level }
            .build(cx, |_| {})
            .bind(level, |mut handle, _| handle.needs_redraw())
    }
}

impl<L: Lens<Target = f32>> View for MeterBarView<L> {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let bounds = cx.bounds();
        let level = self.level.get(cx).clamp(0.0, 1.0);

        // Background
        let bg_rect: vg::Rect = bounds.into();
        let bg_path = vg::Path::rect(bg_rect, None);
        let mut bg_paint = vg::Paint::default();
        bg_paint.set_color(vg::Color::from_rgb(26, 26, 37)); // #1a1a25
        canvas.draw_path(&bg_path, &bg_paint);

        // Filled portion
        if level > 0.001 {
            let fill_width = bounds.w * level;
            let fill_rect = vg::Rect::from_xywh(
                bounds.x, bounds.y, fill_width, bounds.h,
            );
            let fill_path = vg::Path::rect(fill_rect, None);

            // Green -> Yellow -> Red gradient based on level
            let color = if level < 0.6 {
                vg::Color::from_rgb(40, 200, 64) // #28c840
            } else if level < 0.8 {
                vg::Color::from_rgb(254, 188, 46) // #febc2e
            } else {
                vg::Color::from_rgb(229, 57, 53) // #e53935
            };

            let mut fill_paint = vg::Paint::default();
            fill_paint.set_color(color);
            canvas.draw_path(&fill_path, &fill_paint);
        }
    }
}
```

- [ ] **Step 2: Update views/mod.rs**

```rust
mod sidebar;
mod meters;
pub use sidebar::*;
pub use meters::*;
```

- [ ] **Step 3: Build and verify**

Run: `cargo run -p batcherbird-vizia`
Expected: Compiles. Meters aren't wired into the layout yet but the view code compiles.

- [ ] **Step 4: Commit**

```bash
git add crates/batcherbird-vizia/src/views/meters.rs
git commit -m "feat: add custom-drawn meter bar view with level coloring"
```

---

### Task 6: Build custom waveform view

**Files:**
- Create: `crates/batcherbird-vizia/src/views/waveform.rs`
- Modify: `crates/batcherbird-vizia/src/views/mod.rs`

- [ ] **Step 1: Create waveform.rs**

```rust
use vizia::prelude::*;
use vizia::vg;
use batcherbird_core::sampler::VizChunk;
use crate::app_data::AppData;

pub struct WaveformView;

impl WaveformView {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        Self.build(cx, |_| {})
            .bind(AppData::viz_chunks, |mut handle, _| handle.needs_redraw())
    }
}

impl View for WaveformView {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let bounds = cx.bounds();
        let chunks = AppData::viz_chunks.get(cx);

        // Background
        let bg_rect: vg::Rect = bounds.into();
        let bg_path = vg::Path::rect(bg_rect, None);
        let mut bg_paint = vg::Paint::default();
        bg_paint.set_color(vg::Color::from_rgb(12, 12, 18)); // #0c0c12
        canvas.draw_path(&bg_path, &bg_paint);

        // Center line
        let center_y = bounds.y + bounds.h / 2.0;
        let mut center_path = vg::Path::new();
        center_path.move_to(vg::Point::new(bounds.x, center_y));
        center_path.line_to(vg::Point::new(bounds.x + bounds.w, center_y));
        let mut center_paint = vg::Paint::default();
        center_paint.set_color(vg::Color::from_rgb(30, 30, 40)); // #1e1e28
        center_paint.set_style(vg::PaintStyle::Stroke);
        center_paint.set_stroke_width(1.0);
        canvas.draw_path(&center_path, &center_paint);

        if chunks.is_empty() {
            return;
        }

        // Draw waveform from viz chunks
        let num_chunks = chunks.len();
        let x_step = bounds.w / num_chunks as f32;
        let half_h = bounds.h / 2.0;

        // Fill path (translucent)
        let mut fill_path = vg::Path::new();
        fill_path.move_to(vg::Point::new(bounds.x, center_y));

        // Top half (positive peaks)
        for (i, chunk) in chunks.iter().enumerate() {
            let x = bounds.x + i as f32 * x_step;
            let y = center_y - chunk.peak.clamp(0.0, 1.0) * half_h;
            fill_path.line_to(vg::Point::new(x, y));
        }

        // Bottom half (mirror)
        for (i, chunk) in chunks.iter().enumerate().rev() {
            let x = bounds.x + i as f32 * x_step;
            let y = center_y + chunk.peak.clamp(0.0, 1.0) * half_h;
            fill_path.line_to(vg::Point::new(x, y));
        }

        fill_path.close();

        let mut fill_paint = vg::Paint::default();
        fill_paint.set_color(vg::Color::from_argb(30, 74, 158, 255)); // #4a9eff at 12% opacity
        canvas.draw_path(&fill_path, &fill_paint);

        // Stroke path (top edge)
        let mut stroke_path = vg::Path::new();
        for (i, chunk) in chunks.iter().enumerate() {
            let x = bounds.x + i as f32 * x_step;
            let y = center_y - chunk.peak.clamp(0.0, 1.0) * half_h;
            if i == 0 {
                stroke_path.move_to(vg::Point::new(x, y));
            } else {
                stroke_path.line_to(vg::Point::new(x, y));
            }
        }

        let mut stroke_paint = vg::Paint::default();
        stroke_paint.set_color(vg::Color::from_argb(180, 74, 158, 255)); // #4a9eff at 70%
        stroke_paint.set_style(vg::PaintStyle::Stroke);
        stroke_paint.set_stroke_width(1.5);
        stroke_paint.set_anti_alias(true);
        canvas.draw_path(&stroke_path, &stroke_paint);

        // Bottom stroke (mirror)
        let mut bottom_path = vg::Path::new();
        for (i, chunk) in chunks.iter().enumerate() {
            let x = bounds.x + i as f32 * x_step;
            let y = center_y + chunk.peak.clamp(0.0, 1.0) * half_h;
            if i == 0 {
                bottom_path.move_to(vg::Point::new(x, y));
            } else {
                bottom_path.line_to(vg::Point::new(x, y));
            }
        }
        canvas.draw_path(&bottom_path, &stroke_paint);
    }
}
```

- [ ] **Step 2: Update views/mod.rs**

```rust
mod sidebar;
mod meters;
mod waveform;
pub use sidebar::*;
pub use meters::*;
pub use waveform::*;
```

- [ ] **Step 3: Build**

Run: `cargo build -p batcherbird-vizia`
Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add crates/batcherbird-vizia/src/views/waveform.rs
git commit -m "feat: add custom-drawn waveform view using Skia canvas"
```

---

### Task 7: Build keyboard, note display, and progress views

**Files:**
- Create: `crates/batcherbird-vizia/src/views/keyboard.rs`
- Create: `crates/batcherbird-vizia/src/views/note_display.rs`
- Create: `crates/batcherbird-vizia/src/views/progress.rs`
- Modify: `crates/batcherbird-vizia/src/views/mod.rs`

- [ ] **Step 1: Create keyboard.rs**

```rust
use vizia::prelude::*;
use vizia::vg;
use crate::app_data::AppData;

pub struct KeyboardView;

impl KeyboardView {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        Self.build(cx, |_| {})
            .bind(AppData::start_note, |mut handle, _| handle.needs_redraw())
            .bind(AppData::end_note, |mut handle, _| handle.needs_redraw())
            .bind(AppData::current_note, |mut handle, _| handle.needs_redraw())
            .height(Pixels(24.0))
    }
}

impl View for KeyboardView {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let bounds = cx.bounds();
        let start = *AppData::start_note.get(cx);
        let end = *AppData::end_note.get(cx);
        let current = *AppData::current_note.get(cx);

        // Draw 88 keys (A0=21 to C8=108), highlight range and current note
        let total_white_keys = 52; // white keys on a standard piano
        let white_key_w = bounds.w / total_white_keys as f32;
        let black_key_w = white_key_w * 0.6;
        let black_key_h = bounds.h * 0.6;

        // Map MIDI note to whether it's a black key
        fn is_black(note: u8) -> bool {
            matches!(note % 12, 1 | 3 | 6 | 8 | 10)
        }

        // Draw white keys first
        let mut white_x = bounds.x;
        for note in 21..=108u8 {
            if is_black(note) { continue; }

            let in_range = note >= start && note <= end;
            let is_current = note == current && current > 0;

            let rect = vg::Rect::from_xywh(white_x, bounds.y, white_key_w - 1.0, bounds.h);
            let path = vg::Path::rect(rect, None);
            let mut paint = vg::Paint::default();

            paint.set_color(if is_current {
                vg::Color::from_rgb(74, 158, 255) // #4a9eff
            } else if in_range {
                vg::Color::from_rgb(136, 187, 238) // #88bbee
            } else {
                vg::Color::from_rgb(187, 187, 187) // #bbbbbb
            });

            canvas.draw_path(&path, &paint);
            white_x += white_key_w;
        }

        // Draw black keys on top
        white_x = bounds.x;
        for note in 21..=108u8 {
            if is_black(note) {
                let in_range = note >= start && note <= end;
                let is_current = note == current && current > 0;

                // Position black key relative to preceding white key
                let rect = vg::Rect::from_xywh(
                    white_x - black_key_w / 2.0,
                    bounds.y,
                    black_key_w,
                    black_key_h,
                );
                let path = vg::Path::rect(rect, None);
                let mut paint = vg::Paint::default();

                paint.set_color(if is_current {
                    vg::Color::from_rgb(74, 158, 255)
                } else if in_range {
                    vg::Color::from_rgb(60, 80, 120)
                } else {
                    vg::Color::from_rgb(42, 42, 53) // #2a2a35
                });

                canvas.draw_path(&path, &paint);
            } else {
                white_x += white_key_w;
            }
        }
    }
}
```

- [ ] **Step 2: Create note_display.rs**

```rust
use vizia::prelude::*;
use crate::app_data::{AppData, AppState};

pub fn note_display(cx: &mut Context) {
    HStack::new(cx, |cx| {
        Label::new(cx, AppData::current_note.map(|n| AppData::note_name(*n)))
            .class("note-name");
        Label::new(cx, AppData::current_velocity.map(|v| format!("vel {}", v)))
            .class("note-detail");
        Label::new(cx, "·").class("note-detail");
        Label::new(cx, AppData::current_layer.zip(AppData::total_layers).map(|(cur, total)| {
            format!("layer {} / {}", cur, total)
        }))
        .class("note-detail");
    })
    .col_between(Pixels(10.0))
    .display(AppData::app_state.map(|s| {
        if *s == AppState::Recording { Display::Flex } else { Display::None }
    }));
}
```

- [ ] **Step 3: Create progress.rs**

```rust
use vizia::prelude::*;
use vizia::vg;
use crate::app_data::{AppData, AppState};

pub fn progress_bar(cx: &mut Context) {
    HStack::new(cx, |cx| {
        ProgressBarView::new(cx);

        Label::new(cx, AppData::notes_completed.zip(AppData::notes_total).map(|(done, total)| {
            format!("{} / {}", done, total)
        }))
        .class("progress-text");

        Label::new(cx, AppData::notes_completed.zip(AppData::notes_total).map(|(done, total)| {
            if *total > 0 {
                format!("{}%", (*done as f32 / *total as f32 * 100.0) as u32)
            } else {
                String::new()
            }
        }))
        .class("progress-text");
    })
    .col_between(Pixels(10.0))
    .display(AppData::app_state.map(|s| {
        if *s == AppState::Recording { Display::Flex } else { Display::None }
    }));
}

struct ProgressBarView;

impl ProgressBarView {
    fn new(cx: &mut Context) -> Handle<'_, Self> {
        Self.build(cx, |_| {})
            .bind(AppData::notes_completed, |mut handle, _| handle.needs_redraw())
            .height(Pixels(3.0))
    }
}

impl View for ProgressBarView {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let bounds = cx.bounds();
        let completed = *AppData::notes_completed.get(cx) as f32;
        let total = *AppData::notes_total.get(cx) as f32;
        let progress = if total > 0.0 { completed / total } else { 0.0 };

        // Background
        let bg_rect: vg::Rect = bounds.into();
        let bg_path = vg::Path::rect(bg_rect, None);
        let mut bg_paint = vg::Paint::default();
        bg_paint.set_color(vg::Color::from_rgb(26, 26, 37));
        canvas.draw_path(&bg_path, &bg_paint);

        // Fill
        if progress > 0.0 {
            let fill_rect = vg::Rect::from_xywh(
                bounds.x, bounds.y, bounds.w * progress, bounds.h,
            );
            let fill_path = vg::Path::rect(fill_rect, None);
            let mut fill_paint = vg::Paint::default();
            fill_paint.set_color(vg::Color::from_rgb(74, 158, 255)); // #4a9eff
            canvas.draw_path(&fill_path, &fill_paint);
        }
    }
}
```

- [ ] **Step 4: Update views/mod.rs**

```rust
mod sidebar;
mod meters;
mod waveform;
mod keyboard;
mod note_display;
mod progress;
pub use sidebar::*;
pub use meters::*;
pub use waveform::*;
pub use keyboard::*;
pub use note_display::*;
pub use progress::*;
```

- [ ] **Step 5: Build**

Run: `cargo build -p batcherbird-vizia`
Expected: Compiles without errors.

- [ ] **Step 6: Commit**

```bash
git add crates/batcherbird-vizia/src/views/keyboard.rs crates/batcherbird-vizia/src/views/note_display.rs crates/batcherbird-vizia/src/views/progress.rs
git commit -m "feat: add keyboard, note display, and progress bar views"
```

---

### Task 8: Build the stage view and assemble the full layout

**Files:**
- Create: `crates/batcherbird-vizia/src/views/stage.rs`
- Modify: `crates/batcherbird-vizia/src/views/mod.rs`
- Modify: `crates/batcherbird-vizia/src/main.rs`

- [ ] **Step 1: Create stage.rs**

```rust
use vizia::prelude::*;
use crate::app_data::{AppData, AppState};
use crate::app_event::AppEvent;
use crate::views::{meters, WaveformView, KeyboardView, note_display, progress_bar};

pub fn stage(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // Meters (always visible)
        HStack::new(cx, |cx| {
            meters(cx);

            // Recording indicator (only when recording)
            HStack::new(cx, |cx| {
                Element::new(cx)
                    .size(Pixels(10.0))
                    .border_radius(Percentage(50.0))
                    .background_color(Color::from("#e53935"));
                Label::new(cx, "REC").class("rec-text");
            })
            .class("rec-indicator")
            .display(AppData::app_state.map(|s| {
                if *s == AppState::Recording { Display::Flex } else { Display::None }
            }));
        })
        .col_between(Pixels(12.0));

        // Note display (recording only)
        note_display(cx);

        // Keyboard
        KeyboardView::new(cx);

        // Waveform
        WaveformView::new(cx);

        // Progress bar (recording only)
        progress_bar(cx);

        // Idle state content
        VStack::new(cx, |cx| {
            Label::new(cx, "Ready to record").class("idle-text");
            Label::new(cx, AppData::start_note
                .zip(AppData::end_note)
                .zip(AppData::velocity_layers)
                .map(|((start, end), layers)| {
                    let notes = (*end as u32).saturating_sub(*start as u32) + 1;
                    let total = notes * *layers as u32;
                    format!("{} notes · {} velocity layers · {} total samples",
                        notes, layers, total)
                }))
                .class("idle-subtext");
        })
        .alignment(Alignment::Center)
        .display(AppData::app_state.map(|s| {
            if *s == AppState::Idle { Display::Flex } else { Display::None }
        }));

        // ARM button (idle state)
        Button::new(cx, |cx| Label::new(cx, "ARM"))
            .class("btn-arm")
            .on_press(|cx| cx.emit(AppEvent::Arm))
            .display(AppData::app_state.map(|s| {
                if *s == AppState::Idle { Display::Flex } else { Display::None }
            }));

        // RECORD button (armed state)
        Button::new(cx, |cx| Label::new(cx, "RECORD"))
            .class("btn-record")
            .on_press(|cx| cx.emit(AppEvent::StartRecording))
            .display(AppData::app_state.map(|s| {
                if *s == AppState::Armed { Display::Flex } else { Display::None }
            }));
    })
    .class("stage");
}
```

- [ ] **Step 2: Update views/mod.rs**

```rust
mod sidebar;
mod meters;
mod waveform;
mod keyboard;
mod note_display;
mod progress;
mod stage;
pub use sidebar::*;
pub use meters::*;
pub use waveform::*;
pub use keyboard::*;
pub use note_display::*;
pub use progress::*;
pub use stage::*;
```

- [ ] **Step 3: Update main.rs with full layout**

```rust
use vizia::prelude::*;
use batcherbird_vizia::app_data::AppData;
use batcherbird_vizia::views::{sidebar, stage};

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        cx.add_stylesheet(include_style!("src/style/theme.css"))
            .expect("Failed to load theme");

        AppData::default().build(cx);

        HStack::new(cx, |cx| {
            sidebar(cx);
            stage(cx);
        });
    })
    .title("BatcherBird")
    .inner_size((900, 550))
    .run()
}
```

- [ ] **Step 4: Build and run**

Run: `cargo run -p batcherbird-vizia`
Expected: Full layout visible — sidebar on left with config panels, stage on right with meters, keyboard, empty waveform, "Ready to record" text, and ARM button. Clicking ARM should switch to armed state (RECORD button appears).

- [ ] **Step 5: Commit**

```bash
git add crates/batcherbird-vizia/src/views/stage.rs crates/batcherbird-vizia/src/main.rs
git commit -m "feat: assemble full sidebar + stage layout"
```

---

### Task 9: Wire up device enumeration

**Files:**
- Modify: `crates/batcherbird-vizia/src/app_data.rs`
- Modify: `crates/batcherbird-vizia/src/main.rs`

- [ ] **Step 1: Add device enumeration to AppData event handler**

In `app_data.rs`, add to the `Model::event` match:

```rust
AppEvent::RefreshDevices => {
    // MIDI devices
    if let Ok(mut manager) = batcherbird_core::midi::MidiManager::new() {
        if let Ok(devices) = manager.list_output_devices() {
            self.midi_devices = devices;
        }
    }
    // Audio input devices
    if let Ok(manager) = batcherbird_core::audio::AudioManager::new() {
        if let Ok(devices) = manager.list_input_devices() {
            self.audio_input_devices = devices;
        }
    }
}
AppEvent::SelectMidiDevice(idx) => {
    self.selected_midi_device = Some(*idx);
    // Connection will be established when Arm is pressed
}
AppEvent::SelectAudioInput(idx) => {
    self.selected_audio_input = Some(*idx);
}
```

- [ ] **Step 2: Emit RefreshDevices on startup in main.rs**

After `AppData::default().build(cx);`, add:

```rust
cx.emit(AppEvent::RefreshDevices);
```

Add the import:

```rust
use batcherbird_vizia::app_event::AppEvent;
```

- [ ] **Step 3: Build and run**

Run: `cargo run -p batcherbird-vizia`
Expected: Sidebar shows actual MIDI and audio devices from the system. If no devices are connected, shows "No MIDI devices" / "No audio devices".

- [ ] **Step 4: Commit**

```bash
git add crates/batcherbird-vizia/src/app_data.rs crates/batcherbird-vizia/src/main.rs
git commit -m "feat: wire up MIDI and audio device enumeration"
```

---

### Task 10: Wire up real-time meter streaming via timer

**Files:**
- Modify: `crates/batcherbird-vizia/src/app_data.rs`
- Modify: `crates/batcherbird-vizia/src/main.rs`

This is the task that replaces the Tauri channel workaround with direct ring buffer reading.

- [ ] **Step 1: Add meter consumer and timer to AppData**

Add fields to AppData (these won't be lensed — they're engine handles):

```rust
// At the top of app_data.rs, add:
use std::sync::{Arc, Mutex};
use rtrb::Consumer;
use batcherbird_core::lock_free_recording::RealtimeMeterData;

// Add to AppData struct (after viz_chunks):
    // Engine handles (not derived by Lens — accessed via event handler only)
    #[lens(ignore)]
    pub meter_consumer: Option<Arc<Mutex<Consumer<RealtimeMeterData>>>>,
```

- [ ] **Step 2: Handle Tick event — poll ring buffer**

In the `Model::event` match, add:

```rust
AppEvent::Tick => {
    if let Some(consumer) = &self.meter_consumer {
        if let Ok(mut consumer) = consumer.lock() {
            // Drain all available meter data, keep latest
            let mut latest: Option<RealtimeMeterData> = None;
            while let Ok(data) = consumer.pop() {
                latest = Some(data);
            }
            if let Some(data) = latest {
                self.meter_left = data.peak_left;
                self.meter_right = data.peak_right;
                self.meter_left_db = if data.peak_left > 0.0 {
                    20.0 * data.peak_left.log10()
                } else {
                    -60.0
                };
                self.meter_right_db = if data.peak_right > 0.0 {
                    20.0 * data.peak_right.log10()
                } else {
                    -60.0
                };
                self.is_clipping = data.is_clipping;
            }
        }
    }
}
```

- [ ] **Step 3: Register 60fps timer in main.rs**

After building AppData, add:

```rust
use std::time::Duration;
use batcherbird_vizia::app_event::AppEvent;

// Inside Application::new closure, after AppData::default().build(cx):
cx.add_timer(
    Duration::from_millis(16), // ~60fps
    None,                       // run forever
    |cx, action| {
        if let TimerAction::Tick(_) = action {
            cx.emit(AppEvent::Tick);
        }
    },
);
```

- [ ] **Step 4: Build and run**

Run: `cargo run -p batcherbird-vizia`
Expected: App runs. Meters show -60 dB (no audio input connected to consumer yet). No crashes from the timer.

- [ ] **Step 5: Commit**

```bash
git add crates/batcherbird-vizia/src/app_data.rs crates/batcherbird-vizia/src/main.rs
git commit -m "feat: add 60fps timer for meter polling from ring buffer"
```

---

### Task 11: Wire up ARM → monitoring with live meters

**Files:**
- Modify: `crates/batcherbird-vizia/src/app_data.rs`

This connects ARM to the actual audio monitoring pipeline so meters show live input.

- [ ] **Step 1: Add engine fields to AppData**

```rust
use batcherbird_core::sampler::{SamplingEngine, SamplingConfig};
use cpal::Stream;

// Add to AppData struct:
    #[lens(ignore)]
    pub sampling_engine: Option<SamplingEngine>,
    #[lens(ignore)]
    pub monitoring_stream: Option<Stream>,
```

- [ ] **Step 2: Handle Arm event — start monitoring**

Replace the existing `AppEvent::Arm` handler with:

```rust
AppEvent::Arm => {
    if self.app_state == AppState::Idle {
        // Create sampling engine
        let config = SamplingConfig {
            note_duration_ms: self.note_duration_ms as u64,
            release_time_ms: 1000,
            pre_delay_ms: 100,
            post_delay_ms: 100,
            midi_channel: 0,
            velocity: 100,
        };

        match SamplingEngine::new(config) {
            Ok(engine) => {
                // Start monitoring stream
                match engine.start_monitoring_stream() {
                    Ok(stream) => {
                        self.monitoring_stream = Some(stream);
                        self.sampling_engine = Some(engine);
                        self.app_state = AppState::Armed;
                    }
                    Err(e) => {
                        eprintln!("Failed to start monitoring: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to create sampling engine: {}", e);
            }
        }
    }
}
```

- [ ] **Step 3: Handle Disarm — stop monitoring**

Replace the existing `AppEvent::Disarm` handler:

```rust
AppEvent::Disarm => {
    if self.app_state == AppState::Armed {
        self.monitoring_stream = None;
        self.sampling_engine = None;
        self.meter_consumer = None;
        self.app_state = AppState::Idle;
    }
}
```

- [ ] **Step 4: Build and run with audio interface**

Run: `cargo run -p batcherbird-vizia`
Expected: Click ARM → meters start showing live audio input levels. Click back (or add a disarm button) to stop.

Note: The monitoring stream from SamplingEngine may need its level meter state to be wired to the ring buffer consumer. If `SamplingEngine::start_monitoring_stream()` doesn't expose a meter consumer, you'll need to read levels via `engine.get_audio_levels()` in the Tick handler instead:

```rust
// Alternative in Tick handler if no ring buffer consumer:
if let Some(engine) = &self.sampling_engine {
    let levels = engine.get_audio_levels();
    self.meter_left = levels.peak_linear;
    self.meter_right = levels.peak_linear; // mono for now
    self.meter_left_db = levels.peak_db;
    self.meter_right_db = levels.peak_db;
}
```

- [ ] **Step 5: Commit**

```bash
git add crates/batcherbird-vizia/src/app_data.rs
git commit -m "feat: wire ARM to audio monitoring with live meters"
```

---

### Task 12: Wire up recording with live waveform

**Files:**
- Modify: `crates/batcherbird-vizia/src/app_data.rs`

- [ ] **Step 1: Handle StartRecording — spawn recording thread**

Replace the `AppEvent::StartRecording` handler:

```rust
AppEvent::StartRecording => {
    if self.app_state == AppState::Armed {
        self.app_state = AppState::Recording;
        self.notes_total = self.total_samples();
        self.notes_completed = 0;
        self.viz_chunks.clear();

        // Stop monitoring stream before recording
        self.monitoring_stream = None;

        let config = SamplingConfig {
            note_duration_ms: self.note_duration_ms as u64,
            release_time_ms: 1000,
            pre_delay_ms: 100,
            post_delay_ms: 100,
            midi_channel: 0,
            velocity: 100,
        };

        let start_note = self.start_note;
        let end_note = self.end_note;
        let velocity_layers = self.velocity_layers;
        let proxy = cx.get_proxy();

        std::thread::spawn(move || {
            let mut proxy = proxy;
            match SamplingEngine::new(config) {
                Ok(engine) => {
                    // For single note
                    if start_note == end_note && velocity_layers == 1 {
                        match engine.sample_single_note_blocking(start_note, 100) {
                            Ok(_sample) => {
                                let _ = proxy.emit(AppEvent::RecordingComplete);
                            }
                            Err(e) => {
                                let _ = proxy.emit(AppEvent::RecordingError(e.to_string()));
                            }
                        }
                    } else {
                        // Range recording
                        match engine.sample_note_range_blocking(start_note, end_note, 100) {
                            Ok(_samples) => {
                                let _ = proxy.emit(AppEvent::RecordingComplete);
                            }
                            Err(e) => {
                                let _ = proxy.emit(AppEvent::RecordingError(e.to_string()));
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = proxy.emit(AppEvent::RecordingError(e.to_string()));
                }
            }
        });
    }
}
```

Note: `cx.get_proxy()` returns a `ContextProxy` for sending events from background threads. This replaces Tauri's `AppHandle` entirely.

- [ ] **Step 2: Build and run**

Run: `cargo run -p batcherbird-vizia`
Expected: ARM → RECORD triggers recording. After recording completes, state transitions to Review. If MIDI device isn't connected, you'll get an error (expected — MIDI connection wiring comes next).

- [ ] **Step 3: Commit**

```bash
git add crates/batcherbird-vizia/src/app_data.rs
git commit -m "feat: wire recording to background thread with ContextProxy"
```

---

### Task 13: Wire up native file dialog for output directory

**Files:**
- Modify: `crates/batcherbird-vizia/src/app_data.rs`

- [ ] **Step 1: Handle SelectOutputDirectory event**

In the `Model::event` match:

```rust
AppEvent::SelectOutputDirectory => {
    let current_dir = self.output_directory.clone();
    let proxy = cx.get_proxy();

    std::thread::spawn(move || {
        let mut proxy = proxy;
        if let Some(path) = rfd::FileDialog::new()
            .set_directory(&current_dir)
            .pick_folder()
        {
            let _ = proxy.emit(AppEvent::SetOutputDirectory(path));
        }
    });
}
```

- [ ] **Step 2: Build and run**

Run: `cargo run -p batcherbird-vizia`
Expected: Clicking the output directory field in the sidebar opens a native macOS folder picker. Selected folder updates the sidebar display.

- [ ] **Step 3: Commit**

```bash
git add crates/batcherbird-vizia/src/app_data.rs
git commit -m "feat: add native file dialog for output directory selection"
```

---

### Task 14: End-to-end smoke test

**Files:**
- No new files — manual verification

- [ ] **Step 1: Run the full app**

Run: `cargo run -p batcherbird-vizia`

- [ ] **Step 2: Verify the idle state**

Expected:
- Sidebar shows real MIDI and audio devices
- Sampling shows C2–C6, 1 layer, 2.0s
- Export shows Wav24Bit and a valid output directory
- Stage shows meters (flat), keyboard, empty waveform, "Ready to record", ARM button

- [ ] **Step 3: Verify ARM state**

Click ARM.
Expected:
- Meters show live audio input levels
- ARM button replaced with RECORD button
- Stage layout adapts

- [ ] **Step 4: Verify recording (if MIDI device available)**

Click RECORD.
Expected:
- State transitions to Recording
- Note display shows current note
- Progress bar advances
- On completion, state transitions to Review

- [ ] **Step 5: Fix any issues found during smoke test**

Address any layout, styling, or functional issues. These are expected — VIZIA's API may differ slightly from what's in the plan (e.g., exact Dropdown API, CSS property names, lens chaining). Adjust as needed.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "fix: address smoke test findings"
```

---

### Task 15: Update workspace and clean up

**Files:**
- Modify: `Cargo.toml` (workspace)
- Modify: `CLAUDE.md`

- [ ] **Step 1: Verify workspace builds cleanly**

Run: `cargo build --workspace`
Expected: All crates compile (core, cli, vizia).

- [ ] **Step 2: Run all tests**

Run: `cargo test --workspace`
Expected: All tests pass.

- [ ] **Step 3: Update CLAUDE.md**

Add to CLAUDE.md:

```markdown
## VIZIA GUI

- GUI: VIZIA (Rust) in `crates/batcherbird-vizia/`
- Run development: `cargo run -p batcherbird-vizia`
- CSS theme: `crates/batcherbird-vizia/src/style/theme.css` (hot-reload supported)
- Old Tauri GUI in `crates/batcherbird-gui/` is deprecated — do not modify
```

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml CLAUDE.md
git commit -m "chore: update workspace config and docs for VIZIA migration"
```
