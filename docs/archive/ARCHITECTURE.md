# Batcherbird Architecture Documentation

*Comprehensive guide to the Batcherbird hardware sampling system architecture*

## Overview

Batcherbird is a professional hardware sampling tool built with Rust and Tauri, designed to batch-sample hardware synthesizers with professional audio quality. The system separates recording operations from export generation to ensure optimal user experience and reliable file creation.

## System Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Frontend      │    │   Tauri Bridge   │    │   Rust Core     │
│   (JavaScript)  │◄──►│   (Commands)     │◄──►│   (Audio/MIDI)  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
│                                                │
▼                                                ▼
┌─────────────────┐                            ┌─────────────────┐
│   UI Controls   │                            │  Hardware I/O   │
│   Progress      │                            │  MIDI Devices   │
│   File Dialogs  │                            │  Audio Devices  │
└─────────────────┘                            └─────────────────┘
```

## Core Components

### 1. Frontend Layer (`frontend/main.js`)

**Responsibilities:**
- User interface management and responsiveness
- Real-time progress tracking during recording sessions
- Async scheduling to prevent UI blocking
- File dialog integration and preferences management

**Key Functions:**
- `recordRange()` - Orchestrates range sampling with progress feedback
- `recordSample()` - Single note recording with UI updates
- Template system integration and preference persistence

### 2. Tauri Command Layer (`src-tauri/src/lib.rs`)

**Responsibilities:**
- Bridge between JavaScript frontend and Rust backend
- Command routing and parameter validation
- Thread management for audio operations
- Error handling and type conversion

**Key Commands:**
- `record_sample()` - Individual sample recording (blocking)
- `record_range()` - Range sampling with batch processing
- `generate_instrument_files()` - Post-recording export generation
- Device management and audio monitoring commands

### 3. Core Audio Engine (`batcherbird-core/`)

**Responsibilities:**
- Professional audio recording and processing
- MIDI device communication and timing
- Sample detection and trimming algorithms
- Export format generation (.wav, .dspreset, .sfz)

**Key Modules:**
- `sampler/` - Audio recording and MIDI integration
- `export/` - File format generation and batch processing
- `audio/` - Device management and level monitoring
- `midi/` - MIDI communication and device control

## Recording Architecture: Two-Phase Design

### Phase 1: Recording (User Experience Focused)

```rust
// Frontend loop with real-time progress
for note in range {
    await invoke('record_sample', {
        note: note,
        exportFormat: 'wav24bit'  // WAV only during recording
    });
    updateProgress(note);  // UI remains responsive
}
```

**Characteristics:**
- Individual `record_sample` calls for progress feedback
- Only WAV files generated during recording
- Working stop button (can abort between samples)
- Responsive UI with accurate progress tracking
- Professional audio quality maintained

### Phase 2: Export Generation (File Creation Focused)

```rust
// Post-recording instrument file generation
fn generate_instrument_files(directory, format) {
    let wav_files = scan_directory(&directory);
    let samples = parse_metadata_from_filenames(&wav_files);
    
    match format {
        "decentsampler" => generate_dspreset_file(&samples),
        "sfz" => generate_sfz_file(&samples),
        "all" => generate_both_formats(&samples),
    }
}
```

**Characteristics:**
- Scans completed WAV files from recording phase
- Parses note/velocity metadata from standardized filenames
- Generates single instrument file containing all samples
- Supports multiple export formats simultaneously
- Independent of recording success/failure

## Audio Threading Model

### Dedicated Audio Threads
```rust
#[tauri::command]  // Blocking by design
fn record_sample() -> Result<String, String> {
    let (tx, rx) = mpsc::channel();
    
    std::thread::spawn(move || {
        // Dedicated audio thread - stream lives here
        let engine = SamplingEngine::new(config);
        let result = engine.sample_single_note_blocking(&mut midi_conn, note);
        tx.send(result).unwrap();
    });
    
    // Block until audio operation completes
    rx.recv().map_err(|e| e.to_string())
}
```

**Benefits:**
- **Audio Isolation**: Dedicated threads with deterministic scheduling
- **Platform Respect**: No fighting macOS Core Audio constraints
- **Predictable Timing**: Professional audio requirements met
- **Simple Communication**: Channel-based message passing

### UI Responsiveness Pattern
```javascript
async function recordNotesWithResponsiveUI() {
    for (let note of notes) {
        await invoke('record_sample', { note });
        // Yield control back to UI thread
        await new Promise(resolve => setTimeout(resolve, 200));
    }
}
```

## Export System Architecture

### Format Support Matrix

| Format | Extension | Use Case | Compatibility |
|--------|-----------|----------|---------------|
| WAV 16-bit | .wav | Basic compatibility | Universal |
| WAV 24-bit | .wav | Professional quality | Most samplers |
| WAV 32-bit Float | .wav | Maximum fidelity | Pro audio tools |
| Decent Sampler | .dspreset | Ready-to-play instruments | Decent Sampler |
| SFZ | .sfz | Universal sampler format | Most samplers |

### Export Configuration
```rust
pub struct ExportConfig {
    pub output_directory: PathBuf,
    pub naming_pattern: String,        // "{note_name}_{note}_{velocity}.wav"
    pub sample_format: AudioFormat,
    pub normalize: bool,
    pub fade_in_ms: f32,
    pub fade_out_ms: f32,
    pub apply_detection: bool,         // Automatic sample trimming
    pub detection_config: DetectionConfig,
    pub creator_name: Option<String>,
    pub instrument_description: Option<String>,
}
```

### Filename Pattern System
```rust
// Standard Batcherbird format
"Roland-EM1017_C5_72_vel127.wav"
//  ^prefix    ^note ^vel ^velocity

// Parsing regex patterns
Pattern 1: r".*_([A-G][#b]?\d+)_(\d+)_vel(\d+)$"  // Current format
Pattern 2: r".*_([A-G][#b]?\d+)_v(\d+)_rk(\d+)$"  // Legacy support
```

## Sample Detection System

### RMS Window Analysis
```rust
fn apply_detection(&mut self, config: DetectionConfig) -> DetectionResult {
    // Calculate RMS energy over time windows
    let window_size_samples = (config.window_size_ms * self.sample_rate as f32 / 1000.0) as usize;
    let rms_values = calculate_rms_windows(&self.audio_data, window_size_samples);
    
    // Find signal boundaries using threshold
    let threshold_linear = db_to_linear(config.threshold_db);
    let boundaries = find_signal_boundaries(&rms_values, threshold_linear);
    
    // Apply detection with pre/post triggers
    trim_audio_to_boundaries(&mut self.audio_data, boundaries, config);
}
```

### Detection Presets
- **Vintage Synth**: Threshold -30dB, 10ms windows, optimized for analog warmth
- **Percussive**: Threshold -40dB, 5ms windows, fast attack detection  
- **Sustained/Pads**: Threshold -50dB, 20ms windows, preserves long decays

## MIDI Integration

### Device Management
```rust
pub struct MidiManager {
    input: MidiInput,
    output: MidiOutput,
}

impl MidiManager {
    pub fn list_output_devices(&mut self) -> Result<Vec<String>>;
    pub fn connect_output(&mut self, device_index: usize) -> Result<MidiOutputConnection>;
    pub async fn send_test_note(&mut self, channel: u8, note: u8, velocity: u8, duration: Duration);
    pub fn send_midi_panic(&mut self) -> Result<()>;  // Enhanced for vintage synths
}
```

### MIDI Timing and Panic
- **Professional Timing**: Sub-millisecond MIDI note timing precision
- **Enhanced Panic**: Special handling for vintage synthesizers (DW6000 tested)
- **Channel Management**: Per-channel and global panic modes

## Audio Monitoring System

### Real-Time Level Meters
```rust
pub struct AudioLevels {
    pub peak: f32,      // Linear peak amplitude
    pub rms: f32,       // RMS energy level  
    pub peak_db: f32,   // Peak in dB (-∞ to 0)
    pub rms_db: f32,    // RMS in dB (-∞ to 0)
}
```

### Monitoring Architecture
- **Integrated Approach**: Reuses existing SamplingEngine infrastructure
- **Professional Ballistics**: 30Hz update rate with peak hold
- **Color Zones**: Green (good), Yellow (loud), Red (clipping)
- **Resource Efficient**: No separate monitoring streams

## Template System

### Project Templates
```javascript
const TEMPLATES = {
    "vintage-analog": {
        startNote: 24,   // C2
        endNote: 84,     // C6  
        velocityLayers: [96, 127],
        duration: 2000,
        exportFormat: 'sfz'
    },
    // ... 4 more templates
};
```

### Template Categories
1. **Vintage Analog Synth** - Classic synthesizer sampling
2. **Electric Piano** - Velocity-sensitive keyboard instruments  
3. **Drum Machine** - Percussive sounds with minimal layers
4. **Bass Synthesizer** - Low-frequency focused sampling
5. **String Ensemble** - Sustained, layered textures

## Error Handling Philosophy

### Graceful Degradation
```rust
match record_and_export() {
    Ok(files) => show_success("All files created successfully"),
    Err(RecordingError) => show_error("Recording failed - check connections"),
    Err(ExportError) => show_warning("WAV files saved, export failed")
}
```

**Principles:**
- **Audio First**: WAV recording success independent of export
- **Clear Messaging**: Specific error context for troubleshooting
- **User Recovery**: Always provide actionable next steps
- **No Data Loss**: Critical audio data never lost due to export failures

## Performance Characteristics

### Resource Usage
- **CPU Usage**: 5-10% during recording (single-threaded audio)
- **Memory Usage**: 2-4MB baseline + sample buffer allocation
- **Thread Count**: 1-2 threads (main + audio when recording)
- **Latency**: 10-20ms MIDI-to-audio roundtrip

### Scaling Characteristics
- **Single Sample**: <1 second completion time
- **Range Recording**: ~4 seconds per note (configurable duration)
- **Export Generation**: <5 seconds for full range instrument files
- **Memory Scaling**: Linear with sample buffer size and count

## Platform Integration

### macOS Optimizations
- **Core Audio**: Native integration with optimal threading
- **Native Dialogs**: System file picker integration
- **Permission Handling**: Microphone access management
- **Device Detection**: Audio unit and MIDI device discovery

### Cross-Platform Considerations
- **CPAL Audio**: Cross-platform audio layer for universal hardware support
- **Tauri Framework**: Native performance with web-based UI flexibility
- **Rust Core**: Platform-agnostic audio processing algorithms

## Development Workflow

### Building and Testing
```bash
# Development mode with hot reload
cargo tauri dev

# Production build
cargo tauri build

# CLI testing
cargo run --bin batcherbird -- sample-note 60

# Core library testing  
cargo test -p batcherbird-core
```

### Code Organization
```
batcherbird/
├── crates/
│   ├── batcherbird-core/     # Audio processing library
│   ├── batcherbird-gui/      # Tauri application
│   └── batcherbird-cli/      # Command-line interface
├── export_examples/          # Sample output files
└── docs/                     # Additional documentation
```

## Future Architecture Considerations

### Extensibility Points
- **Plugin System**: Modular sample processing algorithms
- **Custom Formats**: User-defined export format specifications
- **Remote Sampling**: Network-based hardware sampling
- **Batch Processing**: Queue-based overnight sampling workflows

### Performance Targets
- **v0.2**: Intelligent auto-loop detection with real-time preview
- **v0.3**: Hardware integration profiles for popular synthesizers
- **v0.4**: VST host integration for software instrument sampling
- **v0.5**: Sub-10ms latency and real-time audio processing

This architecture provides a solid foundation for professional-grade sampling tools while maintaining the flexibility to expand into advanced features as the project grows.