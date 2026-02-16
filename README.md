# Batcherbird

Auto-sampling tool for hardware synthesizers. Records MIDI-triggered samples with professional audio quality.

Built with Rust and Tauri. macOS only.

## Features

**Audio**
- 32-bit float WAV export
- Sub-millisecond MIDI timing
- Automatic release tail capture (500ms)
- Lock-free recording engine (no dropouts)
- RMS-based sample detection and trimming

**Recording Modes**
- Single note with custom velocity/duration
- Range recording (batch entire octaves)
- Velocity layers (2/3/4 layers or custom values)

**Interface**
- Real-time level meters (peak, RMS, peak hold)
- Device auto-detection
- Progress tracking during batch operations

## Requirements

- macOS 10.15+
- Audio interface
- MIDI interface

## Installation

Download from [Releases](https://github.com/ddri/batcherbird/releases), open the DMG, drag to Applications.

First launch: Right-click > Open (bypasses Gatekeeper for unsigned apps).

## Development

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js
brew install node

# Clone and setup
git clone https://github.com/ddri/batcherbird.git
cd batcherbird/crates/batcherbird-gui
npm install

# Run
npm run dev

# Build
cargo tauri build
```

## Usage

1. Connect hardware: Synth audio out > interface > Mac. Mac > MIDI interface > synth.
2. Launch app, click Setup, select MIDI output and audio input devices.
3. Set sample name and save location.
4. Choose recording mode (single note or range), configure parameters, record.

Files save as: `InstrumentName_C4_60_vel127.wav`

## Tested Hardware

| Synthesizer | Audio Interface | Status |
|-------------|-----------------|--------|
| Korg DW6000 | Arturia MiniFuse | Working |

## Troubleshooting

**No MIDI devices**: Connect interface before launching. Check System Preferences > Security & Privacy > Input Monitoring.

**No audio input**: Grant microphone permission. Check System Preferences > Security & Privacy > Microphone.

**Stuck notes**: Use MIDI Panic button. Some vintage synths need longer delays between notes.

## Architecture

Rust backend with CPAL for audio I/O, midir for MIDI, lock-free ring buffers (rtrb) for real-time data. React/TypeScript frontend via Tauri.

See [TAURI_AUDIO_ARCHITECTURE.md](TAURI_AUDIO_ARCHITECTURE.md) for details.

## License

AGPL-3.0. See [LICENSE](LICENSE).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).
