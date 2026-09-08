# Batcherbird

Auto-sampling tool for hardware synthesizers. Records MIDI-triggered samples with professional audio quality.

Built with Rust and Vizia native GUI. macOS only.

## Features

**Audio & Sampling**
- 32-bit float WAV export (also 16-bit and 24-bit PCM)
- Multi-format sampler export: **DecentSampler** (`.dspreset`) and **SFZ 2.0** (`.sfz`)
- Sub-millisecond MIDI timing
- Lock-free recording engine (zero dropouts)
- FFT autocorrelation auto-loop detection with visual waveform overlays
- Automatic release tail capture (500ms) and RMS-based trimming

**Recording Modes & Ranges**
- Standardized velocity layers (1, 2, 3, or 4 layers)
- Flexible note ranges with quick octave presets (1, 2, 4 Octaves)
- Configurable step intervals: Every Note, Every 3rd Note, Every Octave
- Dynamic session time and sample count estimator

**Interface**
- 100% native Rust GUI (Vizia) — lightweight, no embedded web browser
- Real-time hardware-grade level meters (peak, RMS, peak hold)
- Interactive piano keyboard visualizer with stepped key targeting
- Device auto-detection for MIDI and Audio interfaces

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

# Clone and run native GUI
git clone https://github.com/ddri/batcherbird.git
cd batcherbird
cargo run -p batcherbird-vizia

# Run tests
cargo test --workspace

# Or run headless CLI
cargo run -p batcherbird-cli -- --help
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
 
Rust backend with CPAL for low-latency audio I/O, midir for hardware MIDI communication, and lock-free ring buffers (`rtrb`) for 60 FPS glitch-free visualization. Declarative native desktop GUI built with Vizia.


## License

AGPL-3.0. See [LICENSE](LICENSE).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).
