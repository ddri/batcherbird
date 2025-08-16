# Batcherbird Implementation PRD

## Project Overview
**Goal**: Build a professional auto-sampling tool for hardware synthesizers using Rust + Tauri
**Target**: Rival commercial tools like SampleRobot with open-source flexibility
**Platform**: macOS first (user's primary platform), with cross-platform potential

## Architecture Constraints (MUST FOLLOW)

### Core Principle
**Audio operations use BLOCKING Tauri commands with dedicated threads - NOT async commands**

### Implementation Pattern
```rust
// ✅ CORRECT PATTERN - Always use this
#[tauri::command]  // NO async keyword
fn audio_operation() -> Result<String, String> {
    // Spawn dedicated thread for audio
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        // Audio processing here
    });
    
    // Block until complete
    let result = rx.recv()?;
    Ok(result)
}
```

## Current Implementation Status

### ✅ Completed (Working)
1. **Core Audio Engine** - Professional sampling implementation
2. **MIDI Management** - Device enumeration and connection
3. **Audio Device Detection** - Input/output device listing
4. **WAV Export** - Professional 32-bit float export
5. **GUI Framework** - Tauri app with device selection
6. **Preferences** - localStorage for device settings

### 🔄 Needs Architecture Fix (Priority 1)
1. **Record Sample Command** - Currently has thread safety issues
   - **Issue**: Trying to use async patterns with CPAL streams
   - **Fix**: Use blocking command with dedicated thread pattern
   - **Status**: In progress

### 📋 Next Features (Priority 2)
1. **Range Sampling Interface** - C2-C6 batch processing
2. **Sample Detection** - Automatic trimming and normalization
3. **Real-time Monitoring** - Audio level meters during recording

## Detailed Implementation Plan

### Phase 1: Fix Core Recording (In Progress)
**Goal**: Make single note recording work reliably

**Tasks**:
1. ✅ Research proper Tauri + CPAL architecture patterns
2. ✅ Document definitive architecture in `TAURI_AUDIO_ARCHITECTURE.md`
3. 🔄 Fix `record_sample` command using blocking pattern
4. 🔄 Test recording with real hardware (DW6000 + MiniFuse)
5. 🔄 Verify WAV export quality and file naming

**Acceptance Criteria**:
- [ ] Record Sample button works without errors
- [ ] Creates valid WAV files with audio content
- [ ] No thread safety compilation errors
- [ ] Professional audio quality (32-bit float, proper timing)

### Phase 2: Range Sampling Interface
**Goal**: Enable batch sampling across note ranges

**Implementation**:
```rust
#[tauri::command]  // Blocking
fn record_range(start_note: u8, end_note: u8, velocity: u8, duration: u32) -> Result<String, String> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let engine = SamplingEngine::new(config)?;
        let samples = engine.sample_range_blocking(start_note, end_note)?;
        tx.send(samples).unwrap();
    });
    
    let samples = rx.recv()?;
    Ok(format!("Recorded {} samples from {} to {}", 
        samples.len(), note_name(start_note), note_name(end_note)))
}
```

**UI Components**:
- Start/End note selectors (C2-C6 default range)
- Progress bar with current note display
- Velocity and duration controls
- Pause/Resume functionality

### Phase 3: Sample Processing
**Goal**: Professional sample preparation

**Features**:
- Automatic silence detection and trimming
- Optional normalization with preserve dynamics option
- Fade in/out options
- Sample validation and quality checks

### Phase 4: Advanced Features
**Goal**: Professional workflow enhancements

**Features**:
- Real-time audio level meters
- Sample preview playback
- Batch export to multiple formats
- Session save/restore
- MIDI velocity curves

## Technical Requirements

### Core Audio Engine (`batcherbird-core`)
**Must implement blocking methods only**:
```rust
impl SamplingEngine {
    // ✅ Correct: Blocking methods
    pub fn sample_single_note_blocking(&self, conn: &mut MidiConnection, note: u8) -> Result<Sample>;
    pub fn sample_range_blocking(&self, conn: &mut MidiConnection, start: u8, end: u8) -> Result<Vec<Sample>>;
    
    // ❌ Incorrect: No async methods
    // pub async fn sample_single_note(&self, ...) -> Result<Sample>; // DON'T DO THIS
}
```

### GUI Layer (`batcherbird-gui`)
**Must use blocking Tauri commands**:
```rust
// ✅ All audio commands follow this pattern
#[tauri::command]  // NO async
fn audio_command() -> Result<String, String> {
    // Delegate to core via dedicated thread + channels
}
```

### Frontend (HTML/JS)
**Can use normal async/await for Tauri invoke calls**:
```javascript
// ✅ This is fine - frontend doesn't know about backend threading
async function recordSample() {
    const result = await invoke('record_sample', params);
}
```

## Hardware Testing Setup
- **Synthesizer**: Korg DW6000
- **Audio Interface**: Arturia MiniFuse 
- **DAW Verification**: Ableton Live (confirms MIDI/audio routing works)
- **Test Process**: 
  1. Verify MIDI connection triggers synth
  2. Record audio samples
  3. Export to WAV
  4. Validate audio quality in DAW

## Quality Standards
- **Timing Precision**: Sub-millisecond MIDI timing
- **Audio Quality**: 32-bit float WAV, up to 192kHz sample rate
- **File Naming**: Professional pattern `Batcherbird_{note_name}_v{velocity}_rk{note}.wav`
- **Reliability**: Zero audio dropouts, robust error handling
- **Professional UX**: Clear feedback, progress indication, intuitive controls

## Success Metrics
1. **Functional**: Successfully records and exports high-quality samples
2. **Performance**: Professional-grade timing and audio quality
3. **Usability**: Intuitive workflow for musicians
4. **Reliability**: Stable operation with real hardware
5. **Extensibility**: Clear architecture for adding features

## Risk Mitigation
- **Thread Safety**: Follow documented architecture patterns strictly
- **Platform Issues**: Test thoroughly on target macOS system
- **Hardware Compatibility**: Verify with user's actual equipment
- **Performance**: Profile audio operations for real-time suitability

## Implementation Rules (CRITICAL)
1. **NO async Tauri commands for audio operations**
2. **Use blocking commands with dedicated threads**
3. **Communicate via channels (mpsc), not shared state**
4. **Keep CPAL streams in their creation thread**
5. **Follow `TAURI_AUDIO_ARCHITECTURE.md` patterns exactly**

This PRD is the definitive implementation guide. Any deviation from these patterns requires updating this document first.