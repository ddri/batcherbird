# Batcherbird: Tauri + CPAL Audio Architecture on macOS

## Executive Summary

This document establishes the **definitive architecture** for Batcherbird's Tauri GUI + Rust Core Audio Engine on macOS. These patterns are based on 2024 research and proven production implementations.

## Core Architectural Principles

### 1. **Platform Reality: macOS CoreAudio Constraints**
- Apple's CoreAudio requires audio streams to remain on their creation thread
- CPAL streams are intentionally NOT `Send` on macOS (by design, not a bug)  
- This affects ALL audio frameworks on macOS, not just Rust
- Professional audio software (Pro Tools, Logic) uses dedicated audio threads

### 2. **Tauri Command Strategy**
```rust
// ✅ CORRECT: Blocking commands for audio operations
#[tauri::command]  // NOT async
fn audio_operation() -> Result<String, String> {
    // Blocking is acceptable and preferred for audio
}

// ❌ INCORRECT: Async commands with audio streams
#[tauri::command]
async fn audio_operation() -> Result<String, String> {
    // Will fail: streams are not Send
}
```

### 3. **Thread Communication Pattern**
```rust
// ✅ CORRECT: Dedicated audio thread + channels
#[tauri::command]
fn record_sample() -> Result<String, String> {
    let (tx, rx) = mpsc::channel();
    
    std::thread::spawn(move || {
        // Audio processing stays in this dedicated thread
        let stream = device.build_input_stream(/*...*/);
        // Communicate results via channels
        tx.send(result).unwrap();
    });
    
    // Block until audio operation completes
    let result = rx.recv().map_err(|e| e.to_string())?;
    Ok(result)
}
```

## Detailed Architecture

### Layer 1: Core Audio Engine (`batcherbird-core`)
**Responsibility**: Professional audio processing in dedicated threads

```rust
pub struct SamplingEngine {
    // NO async methods - only blocking methods
    pub fn sample_single_note_blocking(&self, connection: &mut MidiConnection, note: u8) -> Result<Sample>;
    pub fn sample_range_blocking(&self, connection: &mut MidiConnection, start: u8, end: u8) -> Result<Vec<Sample>>;
}
```

**Rules**:
- ✅ Blocking methods only
- ✅ Dedicated threads for audio streams  
- ✅ Communication via channels (mpsc)
- ✅ Professional timing and quality
- ❌ No async/await
- ❌ No GUI dependencies

### Layer 2: Tauri GUI (`batcherbird-gui`)
**Responsibility**: User interface and workflow orchestration

```rust
#[tauri::command]  // Blocking commands only
fn record_sample(note: u8, velocity: u8, duration: u32) -> Result<String, String> {
    // 1. Get MIDI connection
    let connection = get_midi_connection()?;
    
    // 2. Spawn dedicated thread for core engine
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let engine = SamplingEngine::new(config)?;
        let result = engine.sample_single_note_blocking(&mut connection, note);
        tx.send((result, connection)).unwrap();
    });
    
    // 3. Block until completion (this is correct for audio)
    let (result, returned_connection) = rx.recv().map_err(|e| e.to_string())?;
    put_connection_back(returned_connection);
    
    // 4. Return result to UI
    match result {
        Ok(sample) => Ok(format!("Recorded {} samples", sample.audio_data.len())),
        Err(e) => Err(e.to_string()),
    }
}
```

**Rules**:
- ✅ Blocking `#[tauri::command]` functions
- ✅ Thread spawning for audio operations
- ✅ Channel-based communication with core
- ✅ Simple orchestration only
- ❌ No async commands for audio
- ❌ No direct audio processing

### Layer 3: Frontend (HTML/JS)
**Responsibility**: User interface only

```javascript
// Frontend calls blocking commands normally
async function recordSample() {
    try {
        const result = await invoke('record_sample', { note, velocity, duration });
        showStatus(result, 'success');
    } catch (error) {
        showStatus(`Recording failed: ${error}`, 'error');
    }
}
```

## macOS-Specific Requirements

### Permissions Setup
```xml
<!-- Info.plist -->
<key>NSMicrophoneUsageDescription</key>
<string>Batcherbird needs microphone access to record audio samples from your synthesizer</string>
```

### Code Signing
- Audio recording works in development mode
- Signed apps require proper entitlements
- Permission dialogs only appear with correct Info.plist

## Implementation Rules

### ✅ DO
1. Use blocking `#[tauri::command]` for all audio operations
2. Spawn dedicated threads for audio processing
3. Communicate via channels (mpsc::channel)
4. Keep streams alive in their creation thread
5. Block command threads until audio operations complete

### ❌ DON'T
1. Use `async fn` for Tauri commands that handle audio
2. Try to send CPAL streams between threads
3. Use tokio::spawn for audio operations
4. Share audio streams across thread boundaries
5. Make audio operations non-blocking in the GUI layer

## Validation Checklist

Before implementing any audio feature:

- [ ] Command is blocking (`#[tauri::command]`, not `async fn`)
- [ ] Audio streams stay in dedicated threads
- [ ] Communication uses channels, not shared state
- [ ] No `Send` requirement on audio streams
- [ ] macOS permissions properly configured

## Reference Implementation

```rust
// File: batcherbird-gui/src-tauri/src/lib.rs
use std::sync::mpsc;
use std::thread;

#[tauri::command]  // BLOCKING - this is correct
fn record_sample(note: u8, velocity: u8, duration: u32) -> Result<String, String> {
    println!("🎛️ GUI: Starting audio recording...");
    
    // Get MIDI connection
    let mut connection = get_midi_connection()?;
    
    // Audio processing in dedicated thread
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        // Core audio engine runs here
        let config = SamplingConfig {
            note_duration_ms: duration as u64,
            release_time_ms: 500,
            pre_delay_ms: 100,
            post_delay_ms: 100,
            midi_channel: 0,
            velocity,
        };
        
        let engine = SamplingEngine::new(config).unwrap();
        let result = engine.sample_single_note_blocking(&mut connection, note);
        
        // Send result back via channel
        tx.send((result, connection)).unwrap();
    });
    
    // Block until audio operation completes
    let (recording_result, returned_connection) = rx.recv()
        .map_err(|e| format!("Audio thread communication failed: {}", e))?;
    
    // Return connection to pool
    put_connection_back(returned_connection);
    
    // Handle result
    match recording_result {
        Ok(sample) => {
            println!("✅ Recording completed: {} samples", sample.audio_data.len());
            Ok(format!("Recording saved: {} samples captured", sample.audio_data.len()))
        },
        Err(e) => {
            println!("❌ Recording failed: {}", e);
            Err(format!("Recording failed: {}", e))
        }
    }
}
```

## Conclusion

This architecture is **proven**, **performant**, and **platform-appropriate**. It respects macOS CoreAudio constraints while delivering professional audio quality. 

**Key Insight**: Blocking commands with dedicated threads are BETTER for audio than async patterns. This is industry standard, not a limitation.