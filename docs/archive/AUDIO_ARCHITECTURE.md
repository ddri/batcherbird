# Batcherbird Audio Architecture

## Professional Audio Application Standards

Batcherbird follows industry-standard practices used by professional audio sampling tools like SampleRobot, Kontakt, Pro Tools, and Ableton Live.

## Real-Time Audio Architecture

### Core Principles
1. **Dedicated Audio Threads** - Real-time audio uses callback-based threads, not async/await
2. **Lock-Free Communication** - Ring buffers and atomic operations for thread communication
3. **Deterministic Timing** - Sub-millisecond precision for MIDI-audio synchronization
4. **Professional Latency** - Target <10ms round-trip latency for monitoring
5. **Buffer Management** - Double buffering and proper buffer sizing for glitch-free audio

### Threading Model
```
Main Thread          Audio Input Thread       MIDI Thread
     |                       |                      |
     |-- Start Recording --> |                      |
     |                       |-- Ring Buffer ---> [Shared Buffer]
     |-- Send MIDI Note -----|----------------------|
     |                       |                      |
     |-- Wait Duration ------|                      |
     |                       |                      |
     |-- Send Note Off ------|----------------------|
     |                       |                      |
     |-- Stop Recording ---> |                      |
     |                       |                      |
     |<-- Process Buffer ----[Shared Buffer]       |
```

### Audio Buffer Management
- **Ring Buffer**: Lock-free circular buffer for real-time audio capture
- **Double Buffering**: Separate read/write buffers to prevent audio dropouts
- **Sample Rate**: Native interface rate (44.1/48/96kHz) - no unnecessary conversion
- **Bit Depth**: 32-bit float internal processing, configurable export format

### MIDI-Audio Synchronization
- **Pre-roll**: 100ms audio capture before MIDI trigger
- **Timestamp Alignment**: Both MIDI and audio events use system monotonic time
- **Latency Compensation**: User-configurable offset for interface latency
- **Precise Timing**: Sub-millisecond MIDI event scheduling

### Industry Standard Practices

#### Like SampleRobot:
- Automatic gain staging and level detection
- Professional file naming for auto-mapping in samplers
- Batch processing with progress tracking and pause/resume
- Quality validation (clipping detection, signal-to-noise ratio)

#### Like Pro Tools/Ableton:
- Real-time level meters with peak hold
- Low-latency monitoring during recording
- Professional audio formats (BWF metadata, high bit-depth)
- Robust error handling and recovery

#### Like Kontakt:
- Sample detection with onset/offset analysis
- Automatic trimming and fade application  
- Root key metadata for auto-mapping
- Multi-velocity layer support

## Implementation Standards

### Audio Stream Setup
```rust
// Industry standard: Callback-based audio, not async
let stream = device.build_input_stream(
    &config,
    move |data, _info| {
        // Real-time audio callback - no allocations!
        ring_buffer.write(data);
    },
    error_callback,
    None
)?;
```

### Buffer Management
```rust
// Lock-free ring buffer for real-time safety
use ringbuf::RingBuffer;
let (producer, consumer) = RingBuffer::<f32>::new(sample_rate * 10).split();
```

### MIDI Timing
```rust
// Precise timing using monotonic clock
use std::time::Instant;
let midi_timestamp = Instant::now();
// Send MIDI with exact timing correlation to audio
```

## Quality Targets (Professional Standards)

### Performance
- **Latency**: <10ms monitoring latency
- **Dropout-Free**: 0 audio dropouts during normal operation
- **CPU Usage**: <25% on modern systems during recording
- **Memory**: <100MB RAM usage for typical sessions

### Audio Quality  
- **Dynamic Range**: >100dB (24-bit minimum)
- **THD+N**: <0.01% for signal chain
- **Frequency Response**: ±0.1dB 20Hz-20kHz
- **Jitter**: <1ms MIDI timing accuracy

### Reliability
- **Sample Accuracy**: 99.9% successful capture rate
- **Error Recovery**: Graceful handling of device disconnection
- **Session Persistence**: Auto-save every 10 samples
- **Crash Recovery**: Resume interrupted sessions

## Dependencies (Professional Grade)

### Audio
- **CPAL**: Cross-platform audio I/O (industry standard)
- **RingBuf**: Lock-free ring buffer for real-time audio
- **Hound**: Professional WAV file I/O with metadata support

### MIDI  
- **midir**: Cross-platform MIDI I/O with precise timing
- **Precise timing**: Monotonic clock synchronization

### Signal Processing
- **RustFFT**: Professional-grade FFT for onset detection
- **Biquad**: Professional filter implementations
- **Custom DSP**: Sample detection, trimming, normalization

This architecture ensures Batcherbird meets professional audio application standards used by industry-leading sampling tools.