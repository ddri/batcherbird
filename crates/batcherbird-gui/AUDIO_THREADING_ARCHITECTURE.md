# Audio Threading Architecture Guide

## Core Principle: Persistent Audio Streams

**The Golden Rule**: Audio streams should run continuously like a heartbeat. Never create/destroy streams for play/pause operations.

## Why This Pattern?

### 1. Platform Threading Constraints
- `cpal::Stream` is **not** `Send` or `Sync` on many platforms (especially macOS)
- Cannot be stored in static variables or passed between threads
- Must live in the thread that created it

### 2. Professional Audio Requirements
- **Zero-latency play/pause** - Stream is always ready
- **Glitch-free operation** - No stream creation overhead
- **Consistent timing** - Audio clock never stops

### 3. How DAWs Work
Professional DAWs (Ableton, Logic, Pro Tools) all follow this pattern:
```
┌─────────────────┐
│   UI Thread     │
│  (Play/Pause)   │
└────────┬────────┘
         │ Atomic Flags
         ▼
┌─────────────────┐
│  Audio Thread   │ ← Runs Forever
│ (Stream Active) │
└────────┬────────┘
         │ Audio Buffer
         ▼
┌─────────────────┐
│ Audio Hardware  │
└─────────────────┘
```

## The Correct Pattern

### ✅ DO: Persistent Stream with State Management

```rust
pub struct AudioEngine {
    // Stream created once, runs forever
    _audio_thread: Option<std::thread::JoinHandle<()>>,
    
    // Atomic state for lock-free communication
    is_playing: Arc<AtomicBool>,
    playback_position: Arc<AtomicU64>,
    
    // Audio data (accessed by audio thread)
    current_sample: Arc<Mutex<Option<AudioData>>>,
}

impl AudioEngine {
    pub fn new() -> Self {
        let is_playing = Arc::new(AtomicBool::new(false));
        let is_playing_clone = Arc::clone(&is_playing);
        
        // Create stream once, runs forever
        let handle = std::thread::spawn(move || {
            let stream = create_output_stream(move |data: &mut [f32], _| {
                if is_playing_clone.load(Ordering::Relaxed) {
                    // Fill buffer with audio
                } else {
                    // Fill buffer with silence
                    data.fill(0.0);
                }
            });
            
            stream.play().unwrap();
            
            // Keep thread alive
            loop {
                std::thread::park();
            }
        });
        
        Self {
            _audio_thread: Some(handle),
            is_playing,
            // ...
        }
    }
    
    pub fn play(&self) {
        // Just flip the atomic flag!
        self.is_playing.store(true, Ordering::Relaxed);
    }
    
    pub fn pause(&self) {
        // Just flip the atomic flag!
        self.is_playing.store(false, Ordering::Relaxed);
    }
}
```

### ❌ DON'T: Store Streams in Structs

```rust
// THIS WILL NOT COMPILE!
pub struct BadAudioPlayer {
    stream: Option<cpal::Stream>, // ❌ Stream is not Send/Sync!
}

static BAD_GLOBAL: Mutex<Option<Stream>> = Mutex::new(None); // ❌ Won't compile!
```

## Real Example: SamplingEngine Pattern

The `SamplingEngine` in our codebase already implements this correctly for monitoring:

```rust
// From sampler.rs - this is the RIGHT way!
pub fn start_monitoring_stream(&self) -> Result<cpal::Stream> {
    // Creates stream that runs continuously
    let stream = self.build_monitoring_stream(...)?;
    stream.play()?;
    // Stream continues running, controlled by atomic flags
}
```

## Key Implementation Rules

1. **Create audio streams in dedicated threads**
   - Stream must live in the thread that created it
   - Use `std::thread::spawn` for audio threads

2. **Use atomic types for audio thread communication**
   - `AtomicBool` for play/pause state
   - `AtomicU64` for playback position
   - Never use mutex in audio callback!

3. **Audio callbacks must be real-time safe**
   - No blocking operations
   - No memory allocation
   - No mutex locks (use atomics)

4. **Buffer management**
   - Use ring buffers for audio data
   - Separate file I/O from audio thread
   - Pre-load samples or stream from disk thread

## Common Mistakes

### Mistake 1: Creating/Destroying Streams
```rust
// ❌ BAD: Creates latency and glitches
fn play() {
    let stream = create_stream();
    stream.play();
}

fn stop() {
    drop(stream); // Destroying stream causes glitches
}
```

### Mistake 2: Storing Streams Globally
```rust
// ❌ BAD: Won't compile due to Send/Sync constraints
static AUDIO_STREAM: Mutex<Option<Stream>> = Mutex::new(None);
```

### Mistake 3: Blocking in Audio Callback
```rust
// ❌ BAD: Will cause audio dropouts
move |data: &mut [f32], _| {
    let samples = mutex.lock().unwrap(); // NEVER lock in audio thread!
}
```

## Correct Architecture for Batcherbird

```rust
// Playback should follow monitoring pattern:
pub struct AudioPlayback {
    // NO stream storage here!
    
    // Atomic state (like SamplingEngine)
    is_playing: Arc<AtomicBool>,
    playback_position: Arc<AtomicU64>,
    
    // Sample data (loaded separately)
    current_sample: Arc<Mutex<Option<PlaybackSample>>>,
}

// Global state should be:
static PLAYBACK_ENGINE: Mutex<Option<Arc<AudioPlayback>>> = Mutex::new(None);
static PLAYBACK_STREAM_HANDLE: Mutex<Option<std::thread::JoinHandle<()>>> = Mutex::new(None);
```

## References

- [cpal documentation on threading](https://docs.rs/cpal/latest/cpal/#threading)
- [Real-time audio programming best practices](https://www.rossbencina.com/code/real-time-audio-programming-101-time-waits-for-nothing)
- Existing `SamplingEngine` implementation in `sampler.rs`

## Summary

The key insight: **Audio streams are like the heart - they should beat continuously**. Play/pause is just telling the heart what blood (audio/silence) to pump, not starting/stopping the heart itself.

When in doubt, look at how `SamplingEngine` handles monitoring - it's already doing it right!