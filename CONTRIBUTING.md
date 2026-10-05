# Contributing to Batcherbird

The current application is Rust with a native Vizia interface. The Tauri/React application has been removed; Node.js, npm, and the Tauri CLI are not required.

## Development

Use a current stable Rust toolchain. macOS is the desktop release target. Hardware testing requires a MIDI-connected synth and an audio input/interface; interface work can use generated demo samples.

```bash
git clone https://github.com/ddri/batcherbird.git
cd batcherbird
cargo run -p batcherbird-vizia -- --demo
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
```

Use `--offline` for Cargo checks when dependencies are already cached. The minimum supported Rust version has not been established. Linux builds in CI are experimental and require the platform audio/display development libraries.

## Structure

- `batcherbird-core`: MIDI, recording, channel routing, meters, preview playback, trimming, loop suggestions, and exports.
- `batcherbird-vizia`: UI, model/event handling, portable session files, preferences, and recovery.
- `batcherbird-cli`: hardware diagnostics and command-line sampling.
- `docs/RELEASE_CHECKLIST.md`: release gates; `docs/archive/`: historical designs, not current requirements.

## Changes and validation

Create a focused branch, describe the user-visible problem and result in the PR, and include the checks you ran. Conventional commit prefixes (`feat:`, `fix:`, `docs:`, `test:`, `refactor:`) are welcome.

Add regression tests for consequential audio, mapping, persistence, or cancellation changes. Test observable results: exported file contents, frame alignment, key and velocity coverage, and restored audio. Do not treat a passing unit suite as proof of hardware behavior.

Audio callbacks must remain bounded and avoid allocations, blocking locks, filesystem work, and logging. Keep interleaved values and audio frames distinct: WAV loop markers are frame coordinates, and every stereo buffer operation must preserve channel pairing.

For UI changes, inspect setup, armed, recording, stopping, review, and error states. Check resizing, readable labels, dropdown contrast, keyboard navigation, selected sample playback, and controls disabled while workers are busy. Demo mode is useful for visual review; microphone permissions and recording still require the real application and hardware.

Session changes must preserve originals and avoid replacing a valid manifest after a failed save. Keep session manifests and their sidecar WAV directories together. Export processing must operate on a copy of the recording.

## Reports and release work

A useful bug report includes the app revision, macOS version and architecture, interface/synth models, input sample rate and channel setup, steps, expected behavior, and displayed errors. Attach a small reproducible session only when you are comfortable sharing its audio.

Before packaging or tagging, complete the [release checklist](docs/RELEASE_CHECKLIST.md). The packaging script stages and replaces only its app/DMG outputs, preserving unrelated files; the tag-triggered workflow uploads a public DMG. Signing, notarization, fresh-install permissions, hardware timing, and listening checks must be recorded separately from automated test results.

Contributions are licensed under [AGPL-3.0-or-later](LICENSE).
