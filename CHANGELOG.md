# Changelog

All notable changes to BatcherBird will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-13

Initial public release of BatcherBird.

### Added

#### Professional Audio Engine
- 32-bit float WAV export (studio standard)
- Sub-millisecond MIDI timing precision
- Zero-dropout recording with lock-free architecture
- Automatic release tail capture (500ms)
- Persistent audio streams for reliable recording

#### Recording Modes
- **Single Note Recording**: Individual note sampling with custom velocity/duration
- **Range Recording**: Batch sample entire octaves (C2-C7) automatically
- **Velocity Layer Sampling**: Multi-dynamic recording (2/3/4 layers + custom)
  - Professional velocity curves (pp/mp/mf/ff dynamics)
  - Smart naming convention: `Instrument_C4_60_vel127.wav`

#### Real-Time Visualization
- 60fps waveform display during recording
- Professional VU-style meters with peak/RMS
- Color-coded level zones (green/yellow/red)
- Peak hold indicators with clipping detection

#### Sample Processing
- Intelligent sample detection with RMS window analysis
- Automatic trimming with configurable threshold
- FFT-based loop detection (5-10x faster than traditional methods)
- Multi-algorithm detection engine

#### Export Formats
- **DecentSampler** (.dspreset) - Complete instrument presets
- **SFZ 2.0** - Universal sampler format
- Professional WAV with metadata (SMPL chunk support)

#### User Interface
- Professional dark theme
- Device auto-detection for MIDI and audio
- Keyboard shortcuts (spacebar play/pause, ESC cancel)
- Toast notifications for non-intrusive feedback
- Real-time progress tracking for batch operations

### Technical Notes

- Built with Rust + Tauri 2.6 for native performance
- React 19 + TypeScript frontend
- Lock-free ring buffers for real-time audio (rtrb)
- CPAL for cross-platform audio I/O
- midir for MIDI device management

### Known Limitations

- macOS only (Windows/Linux planned for future releases)
- App is unsigned (Gatekeeper warning on first launch)
- Extended sampling sessions not yet battle-tested
- Quality Validation Dashboard uses mock data (backend not implemented)

[0.1.0]: https://github.com/ddri/batcherbird/releases/tag/v0.1.0
