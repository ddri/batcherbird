# Changelog

All notable changes to BatcherBird are documented here.

## April 5, 2026

### Added

- Native VIZIA GUI replacing the Tauri + React frontend — single Rust binary, no webview, no Node.js
- Hybrid sidebar + stage layout with everything visible on one screen
- PickList dropdowns for MIDI device, audio device, and export format selection
- Increment/decrement controls for note range, velocity layers, and duration
- Custom-drawn meter bars, waveform display, and piano keyboard range visualization
- State-driven stage that adapts to Idle, Armed, Recording, and Review phases
- Error banner with dismiss button (errors were previously only printed to stderr)
- Cancel and Disarm buttons for all interruptible states
- Review state with playback controls and recording summary
- Native file dialog for output directory selection
- 60fps timer for real-time meter updates via lock-free ring buffer polling
- Linux CI build matrix (experimental)
- Formatting check (cargo fmt) in CI pipeline

### Improved

- Replaced Arc<Mutex> with lock-free ring buffer in range sampling audio callbacks — no more mutex in the audio thread
- Path validation uses canonicalize() instead of string checks (prevents symlink traversal)
- MIDI note and velocity inputs are bounds-checked on all commands
- Session IDs validated as UUID before constructing file paths
- Atomic ordering on recording cancellation flag changed from Relaxed to Acquire/Release
- Replaced eprintln! with tracing::error! across all audio code
- Release binaries are now stripped (smaller file size)

### Fixed

- CVE-2026-25541 (bytes integer overflow) and RUSTSEC-2025-0047 (slab out-of-bounds access)
- Panic in MIDI input creation now returns an error instead of crashing
- SessionManager::default() no longer panics on failure
- Path validation added to generate_instrument_files (was missing entirely)
- All clippy warnings resolved (27 → 0)

### Removed

- 562 lines of dead code (deprecated async recording methods, three duplicate note_to_name functions)
- Stale frontend-backup/ directory and duplicate samplesss/ samples
- Tracked .DS_Store file

## February 13, 2026

### Added

- 32-bit float WAV export with sub-millisecond MIDI timing
- Zero-dropout recording with lock-free architecture
- Single note, range, and velocity layer recording modes
- 60fps waveform display during recording
- Professional meters with peak/RMS and clipping detection
- FFT-based loop detection (5-10x faster than traditional methods)
- DecentSampler and SFZ export formats
- Dark theme UI with device auto-detection
- Keyboard shortcuts (spacebar play/pause, ESC cancel)
