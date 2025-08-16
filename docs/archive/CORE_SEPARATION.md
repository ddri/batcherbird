# Batcherbird Architecture: Core Separation Principle

## Critical Design Principle

**Batcherbird follows a strict two-layer architecture where the Core Audio Engine and GUI are completely separate concerns.**

This separation is **essential** for professional audio software and must be maintained at all times.

## Layer Responsibilities

### 🎛️ **Core Audio Engine** (`batcherbird-core`)
**Role**: Professional audio and MIDI processing
**Location**: `/crates/batcherbird-core/`

**Responsibilities:**
- ✅ Real-time audio recording and processing
- ✅ MIDI sequencing and timing
- ✅ Sample detection and analysis  
- ✅ Professional audio formats and export
- ✅ Hardware device communication
- ✅ Audio quality validation
- ✅ Sample processing (trim, normalize, fade)

**Standards:**
- Sub-millisecond timing precision
- Lock-free real-time audio processing
- Professional audio formats (BWF, high bit-depth)
- Industry-standard buffer management
- Zero audio dropouts

### 🖥️ **Tauri GUI Layer** (`batcherbird-gui`)
**Role**: User interface and workflow orchestration
**Location**: `/crates/batcherbird-gui/`

**Responsibilities:**
- ✅ User interface components and interactions
- ✅ Device selection and configuration
- ✅ Progress tracking and visual feedback
- ✅ File management and organization
- ✅ Session persistence and settings
- ✅ Simple command orchestration

**Standards:**
- Responsive user interface
- Clear workflow guidance
- Intuitive controls and feedback
- Professional UI/UX patterns

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                 USER INTERFACE                          │
│                 (Tauri + HTML/JS)                       │
├─────────────────────────────────────────────────────────┤
│               GUI COORDINATION                          │
│               (Rust GUI Logic)                          │
│                                                         │
│  fn record_sample() {                                   │
│    core_engine.sample_single_note(note, velocity)      │
│  }                                                      │
├═════════════════════════════════════════════════════════┤
│                                                         │
│              CORE AUDIO ENGINE                          │
│              (batcherbird-core)                         │
│                                                         │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐      │
│  │    MIDI     │ │    Audio    │ │   Sample    │      │
│  │  Sequencer  │ │  Recorder   │ │  Processor  │      │
│  └─────────────┘ └─────────────┘ └─────────────┘      │
│                                                         │
├═════════════════════════════════════════════════════════┤
│                                                         │
│                 HARDWARE LAYER                          │
│              (Audio Interface + Synth)                  │
│                                                         │
│        MiniFuse ←→ DW6000 Synthesizer                   │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## Communication Protocol

### ✅ **Correct Pattern**
```rust
// GUI Layer - Simple command orchestration
#[tauri::command]
fn record_sample(note: u8, velocity: u8, duration: u32) -> Result<String, String> {
    // Get MIDI connection
    let connection = get_midi_connection()?;
    
    // Delegate to Core Audio Engine
    let sample = core_engine.sample_single_note(connection, note, velocity, duration)?;
    
    // Handle result in GUI context
    Ok(format!("Recorded: {}", sample.filename))
}
```

### ❌ **Anti-Pattern** 
```rust
// GUI Layer - DON'T implement audio processing here!
#[tauri::command] 
fn record_sample() -> Result<String, String> {
    // ❌ Building audio streams in GUI layer
    let stream = device.build_input_stream(...)?;
    
    // ❌ Audio processing in GUI layer  
    let ring_buffer = RingBuffer::new(...);
    
    // ❌ Real-time audio in GUI layer
    stream.play()?;
}
```

## Why This Separation Matters

### **Professional Audio Standards**
- **Deterministic Performance**: Core audio must be predictable and fast
- **Real-time Safety**: No GUI blocking audio processing
- **Hardware Abstraction**: Core handles device complexity
- **Testing**: Audio engine can be tested independently

### **Development Benefits**
- **Clear Responsibilities**: Each layer has a single purpose
- **Maintainability**: Changes don't affect the other layer
- **Reusability**: Core engine could power CLI, VST, or other interfaces
- **Debugging**: Isolate audio issues from GUI issues

### **Industry Examples**
This pattern is used by all professional audio software:
- **Pro Tools**: Core audio engine + GUI front-end
- **Ableton Live**: Core engine + Push/GUI interfaces  
- **SampleRobot**: Core sampling engine + multiple GUI options
- **Kontakt**: Core sampler + multiple interfaces

## Implementation Rules

### **Core Audio Engine Rules**
1. ✅ **No GUI dependencies** - never import Tauri or web technologies
2. ✅ **Blocking operations OK** - real-time audio is naturally blocking
3. ✅ **Professional standards** - sub-millisecond timing, lock-free buffers
4. ✅ **Hardware abstraction** - handle device complexity internally
5. ✅ **Complete functionality** - fully functional without GUI

### **GUI Layer Rules**  
1. ✅ **No audio processing** - delegate everything to core
2. ✅ **Simple orchestration** - coordinate core functions
3. ✅ **User experience focus** - progress, feedback, configuration
4. ✅ **Async-friendly** - work with Tauri's async model
5. ✅ **Stateless when possible** - core holds the real state

## Current Status

### ✅ **Correctly Separated**
- Device enumeration (core provides devices, GUI displays them)
- MIDI connection management (core handles connections)
- Export functionality (core processes, GUI coordinates)

### 🔄 **Needs Separation**
- Audio recording (currently duplicated in GUI layer)
- Real-time processing (should be core-only)
- Professional timing (belongs in core)

## Next Steps

1. **Remove audio processing from GUI layer**
2. **Fix async/threading in core layer** 
3. **Use existing `SamplingEngine`** instead of reimplementing
4. **Keep GUI as thin orchestration layer**

**Remember**: The core audio engine is the heart of Batcherbird. The GUI is just a nice interface to it.