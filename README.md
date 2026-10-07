# Batcherbird

Batcherbird is a macOS desktop tool for turning hardware synthesizer patches into playable sample instruments. It sends MIDI notes, records your audio interface, and exports WAV recordings with DecentSampler or SFZ presets.

Built in Rust with a native Vizia interface. The current development build is a **release candidate pending hardware, sampler, and installer validation**. See [the release checklist](docs/RELEASE_CHECKLIST.md) for the remaining gates.

## What it does

- Select MIDI output, audio input, stereo or mono input routing, and input gain.
- Check levels, monitor input, audition notes, and capture a configurable note range.
- Choose note spacing, velocity layers, hold duration, and release-tail duration.
- Stop a batch while keeping completed samples; inspect and audition each recording, or re-record a selected sample.
- Save and reopen portable sessions, remember capture settings, and recover the last saved recording batch.
- Export 16-bit PCM, 24-bit PCM, or 32-bit float WAV; DecentSampler; SFZ; or a combined instrument package.
- Apply RMS-based silence trimming and fades to an export copy. Sparse sampled notes receive key zones between the captured endpoints; velocity zones cover 1–127 without overlapping boundaries.

Automatic silence trimming uses a −40dB threshold and can remove very quiet
attacks or tails. Turn **Trim silence** off in Export settings when you need to
preserve the full take; export fades still apply.

Automatic loop detection is experimental and off by default. It uses zero crossings and waveform correlation, with frame-aware stereo checks. A suggested loop still needs listening in the target player. SFZ crossfades are supported by only some players. The review waveform shows the original recording, not the processed export.

## Run it

You need a current stable Rust toolchain and macOS. The current local candidate is an Apple Silicon (`arm64`) build with a macOS 11.0 binary minimum. Intel builds and fresh-install compatibility across macOS versions remain unverified. Recording requires a MIDI connection and an audio input; the demo requires neither.

```bash
git clone https://github.com/ddri/batcherbird.git
cd batcherbird
cargo run -p batcherbird-vizia

# Explore the review interface with generated samples
cargo run -p batcherbird-vizia -- --demo

# Generate a portable six-take session for Open/Save/Export testing
cargo run -p batcherbird-vizia --example generate_test_session -- target/manual-qa/quiet-stereo.batcherbird

# Tests and command-line diagnostics
cargo test --workspace
cargo run -p batcherbird-cli -- --help
```

Demo samples are synthetic and do not establish hardware recording quality. Demo mode does not restore your normal preferences or recovery session.

Use the [desktop acceptance checks](docs/DESKTOP_ACCEPTANCE.md) to exercise review,
dialogs, sessions, and export without a synth. The [session resilience checks](docs/SESSION_ACCEPTANCE.md)
cover corrupt files, missing audio, copied sessions, failed saves, and cancellation.
The [hardware acceptance procedure](docs/HARDWARE_ACCEPTANCE.md)
provides a short test with six takes and explicit expected key/velocity boundaries
for when your synth is available.

## Capture a patch

1. Connect the Mac's MIDI output to the synth and the synth's audio output to your audio interface. Select the intended patch on the synth.
2. In **Connections**, select MIDI output, audio input, and input channels. Refresh devices if needed.
3. Set the note range, note spacing, velocity layers, hold, and release tail. Choose an instrument name.
4. Click **Check input signal**, then **Test note**. Check the meters and adjust the synth/interface level or input gain to avoid clipping. Enable monitoring only when your routing is suitable.
5. Click **Start recording**. **Stop recording** releases notes and retains completed recordings.
6. In review, select samples to inspect their waveforms and **Play sample**. Use **Re-record sample** when a take needs replacement.
7. **Save session** to preserve original audio and settings. Choose an export destination and format, then export. Open the result in the intended sampler and audition the whole range.

### Sessions and recovery

A `.batcherbird` file is a JSON manifest accompanied by a sibling `<name>.audio-<timestamp>/` directory containing unprocessed 32-bit float WAV files. Keep the manifest and its referenced audio directory together when moving or sharing a session. Saving creates a new audio directory before replacing the manifest, then removes the superseded directory only when its name, referenced WAVs, and actual files establish that this session owns it. Foreign files or symlinks prevent automatic cleanup; older directories may then remain. Keep separate copies of both manifest and audio for backups. Do not remove an audio directory referenced by a session you intend to keep.

Preferences and the automatic recovery snapshot live in the platform configuration directory under `batcherbird/` (`~/Library/Application Support/batcherbird/` on macOS). Recovery is written after a recording batch finishes or stops and restores that captured batch together with its capture settings. Opening or editing another saved session does not update this checkpoint. On startup, a recovered batch's settings take precedence over remembered preferences; reopen your saved session to continue other work. Recovery is not continuous protection for an in-progress take. Save sessions explicitly for recordings you want to keep.

## Packaging and validation

The [macOS packaging script](scripts/package-macos.sh) builds `batcherbird-vizia`, stages an `.app`, and creates a DMG when `hdiutil` is available. It replaces only `Batcherbird.app` and `Batcherbird.dmg` after successful staging, preserves unrelated output files, and restores previous artifacts if installation fails. Local builds do not apply Developer ID signing or notarization by default. Optional signing and Keychain-profile notarization are described in the [distribution guide](docs/DISTRIBUTION.md); a linker-generated ad hoc signature is not Developer ID distribution signing. The bundle minimum macOS version follows the executable's Mach-O metadata. Tag pushes trigger the [release workflow](.github/workflows/release.yml), so complete the release checklist before pushing a release tag.

Choose a separate local candidate directory to review a build while preserving existing distribution artifacts:

```bash
BATCHERBIRD_DIST_DIR=target/release-candidate ./scripts/package-macos.sh
```

The default output is `dist/`. `CARGO_TARGET_DIR` selects the Cargo build directory and the packaged binary's source; a relative override is resolved from the repository root. The script refuses filesystem/home/repository roots and ambiguous output symlinks. Packaging fixture checks run with `bash scripts/test-package-macos.sh`.

The Korg DW6000 and Arturia MiniFuse were reported working in earlier development. They need to be checked again against the current recording pipeline. Automated checks do not establish device latency or absence of dropouts. Independent SFZ renderer probes and the remaining DecentSampler listening gate are documented in [sampler acceptance](docs/SAMPLER_ACCEPTANCE.md).

## Troubleshooting

- **No input signal:** confirm synth/interface cabling and selected channels; grant microphone access to the packaged app in macOS Privacy & Security.
- **No MIDI device:** reconnect the interface, refresh devices, and check MIDI routing and the synth's receive channel.
- **Clipping:** lower the synth/interface gain or the app's input gain before recording.
- **Stuck notes:** stop recording or disarm to release audition notes; use the synth/interface panic function if needed and check the synth's MIDI configuration.
- **Recovery or session error:** retain the manifest and audio directories and report the displayed error; do not replace the only copy of a recording.

## Repository

| Component | Purpose |
| --- | --- |
| `crates/batcherbird-core` | Recording, MIDI, routing, processing, playback, and export |
| `crates/batcherbird-vizia` | Native desktop interface, sessions, and preferences |
| `crates/batcherbird-cli` | Command-line sampling and diagnostics |
| `docs/archive` | Historical plans, including the retired Tauri frontend |

CPAL handles audio, midir handles MIDI, and ring buffers transfer audio between threads. Linux CI is experimental; the first desktop release targets macOS.

## Contributing and license

See [CONTRIBUTING.md](CONTRIBUTING.md). Licensed under [AGPL-3.0-or-later](LICENSE).
