# Changelog

All notable changes to BatcherBird are documented here.

## Unreleased — October 4, 2026

Release candidate work; hardware, target-sampler, and installer validation remain pending.

### Added

- A redesigned native workspace with a connections/capture/export inspector, sample browser, selected waveform, and clear setup, recording, stopping, and review states.
- Generated review samples with `cargo run -p batcherbird-vizia -- --demo`.
- Portable `.batcherbird` sessions with lossless original WAV sidecars, saved preferences, and post-batch recovery snapshots.
- Selected-sample audition and re-recording, configurable release-tail duration, and instrument naming.

### Fixed

- Capture metadata now follows the actual audio stream configuration; selected input channels and gain apply to recorded audio.
- Cancellation interrupts note waits, releases MIDI notes, and preserves completed samples; buffer overflow and stream failures are surfaced.
- Preview completion waits for the final buffer's estimated playback deadline with a conservative drain allowance; explicit Stop still acts immediately.
- Long session filenames stay within the toolbar, preserving space for session actions.
- Unsaved-session replacement uses an in-app confirmation instead of a separate system alert.
- Sparse-note key zones fill the captured range; midpoint velocity zones cover 1–127 without duplicate boundaries.
- WAVs and presets use the same trimmed/faded export audio timeline, with frame-aware stereo trimming and fades.
- SFZ paths resolve to adjacent WAVs; alternate WAV depths no longer overwrite shared preset WAVs; duplicate filename patterns fail before writing audio.
- Automatic loop suggestions check correlation across channels; DecentSampler crossfade lengths use frames, while SFZ uses seconds.

### Changed

- Automatic looping is explicitly experimental and off by default, independently of silence trimming.
- Documentation reflects the Vizia application and separates automated checks from hardware and release claims.

Historical entries below describe earlier development. Their feature and performance claims are not a validation record for the current application.

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

- Recording uses ring buffers during batch range sampling; dropout behavior requires hardware testing
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
- 32-bit float WAV export and MIDI timing diagnostics
- Automatic loop suggestions using zero crossings and waveform correlation
- Export to DecentSampler and SFZ formats
- Professional metering with peak, RMS, and clipping detection
- Dark theme interface with device auto-detection
- Keyboard shortcuts: spacebar to play/pause, ESC to cancel
