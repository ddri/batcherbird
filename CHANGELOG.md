# Changelog

All notable changes to BatcherBird are documented here.

## April 5, 2026

### Added

- New native GUI built from scratch — faster startup, smaller app, no browser engine running underneath
- Sidebar shows all settings at once: devices, note range, velocity layers, duration, export format, and output directory
- MIDI and audio devices are now selectable from dropdown menus
- Export format (WAV 16/24/32, DecentSampler, SFZ) is selectable from a dropdown
- Note range, layers, and duration have +/- controls for quick adjustment
- Real-time level meters in the main stage area
- Piano keyboard strip showing the selected note range
- Waveform display area for monitoring recordings
- Clear error messages shown in the app when something goes wrong (previously errors were invisible)
- Cancel button during recording and a way to back out of armed state
- Review screen after recording completes with playback controls and sample count
- Output directory can be changed by clicking the OUTPUT field
- Linux build support in CI (experimental)

### Improved

- Recording no longer risks audio dropouts during batch range sampling
- File path handling is more secure against directory traversal
- MIDI and audio input values are validated before recording starts
- App binary is smaller thanks to symbol stripping

### Fixed

- Two dependency vulnerabilities patched (memory safety issues in upstream libraries)
- App no longer crashes if the MIDI subsystem is unavailable at startup
- Formatting applied consistently across all source code

### Removed

- Old Tauri/React frontend (replaced by the new native GUI)
- Leftover backup files and duplicate sample recordings from the repository

## February 13, 2026

### Added

- Record samples from hardware synthesizers with real-time waveform visualization
- Single note, note range, and velocity layer recording modes
- 32-bit float WAV export with sub-millisecond MIDI timing
- Automatic loop detection using FFT analysis
- Export to DecentSampler and SFZ formats
- Professional metering with peak, RMS, and clipping detection
- Dark theme interface with device auto-detection
- Keyboard shortcuts: spacebar to play/pause, ESC to cancel
