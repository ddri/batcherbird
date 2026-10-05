# macOS release checklist

**Status: release candidate pending validation.** This checklist is a gate for a release tag, not a claim that the checks have been performed. Record revision, tester, date, Mac model/architecture, macOS version, interface and driver/firmware, synth, and sampler version alongside results. Start a real-device check with the [30-minute hardware acceptance session](HARDWARE_ACCEPTANCE.md).

The first release promises a complete capture → review → saved session → exported instrument workflow on macOS. Supported source features include stereo/mono input routing and gain, MIDI-triggered note ranges and velocity layers, configurable hold/release timing, cancellation with completed-take retention, sample browsing and audition, selected-sample re-recording, sessions and settings, RMS-based export trimming/fades, WAV export, DecentSampler and SFZ presets. Automatic loop suggestions are experimental and opt-in. Manual waveform trim/loop editing, signed/notarized distribution, and cross-platform release support are not established by this candidate.

## Automated checks

- [x] Run `cargo fmt --all -- --check` — passed on the working tree on October 5, 2026.
- [x] Run `cargo test --workspace` and record pass/fail/ignored counts: 137 passed, 0 failed, 1 ignored (`--offline`) on October 5, 2026. Subsequent code changes require a new recorded check. Hardware-ignored tests remain a separate gate.
- [x] Run `cargo clippy --workspace --all-targets --offline -- -D warnings`: passed on October 5, 2026.
- [x] Confirm regression coverage for capture frame integrity, cancellation, sample/key/velocity mapping, actual SFZ paths, stereo trimming/fades, loop coordinates, alternate WAV depths, and session save/load failures — targeted suites and the October 5 workspace run passed. Changes after that run require a new recorded check.
- [x] Run `bash -n scripts/package-macos.sh scripts/test-package-macos.sh` and `bash scripts/test-package-macos.sh` — passed. Fake tools verify success, preservation of unrelated files, rollback after build/DMG/install failure, empty-DMG rejection, output-root guards, and minimum-OS metadata for multiple/legacy Mach-O slices and inspection fallback.
- [ ] Review any changes after the recorded checks; rerun affected checks before release.

## Interface and session behavior

- [ ] Launch both normal mode and `cargo run -p batcherbird-vizia -- --demo`; inspect readable text, dropdown contrast, focus/keyboard navigation, scrolling, and resizing.
- [ ] Inspect setup, armed, recording, stopping, review, and error states. Confirm asynchronous operations disable conflicting actions.
- [ ] Select different samples and verify waveform, note/velocity label, and playback match. Stop playback and switch samples safely.
- [ ] Re-record a selected sample; confirm it replaces that take and retains the rest. Cancel or fail re-recording and verify the original survives.
- [ ] Save a `.batcherbird` session, quit, reopen, and compare audio, channels, rates, settings, and sample count.
- [ ] Move the manifest **and its referenced `<name>.audio-<timestamp>/` directory together** to another folder and reopen successfully.
- [ ] Save again and confirm a new audio directory precedes manifest replacement. Confirm the superseded app-owned sidecar is removed only after success; foreign files, symlinks, or directories prevent cleanup. A failed save must preserve the previous session and its audio.
- [ ] Confirm preferences restore from the platform configuration directory's `batcherbird/settings.json`.
- [ ] Complete/stop a batch, restart, and verify `batcherbird/recovery.batcherbird` and its sidecar restore the saved recordings. Recovery is post-batch, not protection for an interrupted in-progress take.
- [ ] Exercise missing/corrupt manifest or sidecar, unavailable devices, unwritable destination, and canceled dialogs. Confirm errors are actionable and existing recordings survive.

## Hardware capture gates

- [ ] Use a real MIDI/audio setup; record the stream sample rate and actual input/output channel configuration. Test supported 44.1kHz and 48kHz configurations where available.
- [ ] Capture stereo, mono-left, mono-right, and a mono input device. Verify channel identity, frame alignment, pitch, duration, and audible input-gain changes in the saved audio.
- [ ] Test no signal, quiet attacks, long release tails, clipping, and interface disconnect/stream failure. Confirm meters and errors match the situation.
- [ ] Record a short chromatic range and a sparse range with multiple velocity layers. Verify ordering, file count, correct root notes, and the intended synth patch.
- [ ] Stop during a long held note and during its release tail. Verify notes release promptly, UI shows stopping until the worker finishes, partial take is discarded, completed takes remain, and another recording can start.
- [ ] Test MIDI cleanup and note audition; confirm no hanging notes after cancellation, errors, or repeated sampling.
- [ ] Run a longer batch and listen for clicks, truncation, or gaps. Record observed timing and errors rather than asserting zero dropouts or sub-millisecond MIDI timing.
- [ ] Recheck the previously reported Korg DW6000 / Arturia MiniFuse setup against this revision; historical reports do not validate the new pipeline.

## Export and target-sampler gates

- [ ] Export WAV 16-bit, 24-bit, and 32-bit float; inspect rate, channels, frame length, level, and MIDI root metadata in an independent reader.
- [ ] Compare original session audio with processed export: start, release tail, fades, and channel pairing. Confirm originals remain unchanged.
- [ ] Load DecentSampler and SFZ presets from a moved export directory with adjacent WAVs. Check missing-file behavior and paths including spaces and non-ASCII instrument names.
- [ ] Play every key between captured endpoints and velocities around every zone boundary. Verify there are no silent gaps or unintended duplicate triggers, and pitches are correct.
- [ ] Export combined formats plus alternate WAV depths; verify preset WAVs remain 24-bit and alternate depths use their own directories.
- [ ] With looping off, confirm one-shot export has no auto-loop markers. With looping enabled, compare WAV `smpl`, DecentSampler, and SFZ loop frame positions after trimming; audition the seam and release behavior.
- [ ] Confirm DecentSampler crossfade is expressed in frames; SFZ `loop_crossfade` is seconds. Test the chosen SFZ player explicitly: many players do not implement that opcode.
- [ ] Test quiet/noisy/nonperiodic patches; reject unsuitable loop suggestions. Looping remains experimental until listening evidence supports it.

## Packaging and publication gates

The current `scripts/package-macos.sh` builds `batcherbird-vizia` in release mode, obtains version `0.1.0` from its Cargo manifest, writes bundle metadata and microphone usage text, and creates `dist/Batcherbird.app` and `dist/Batcherbird.dmg` on macOS. It stages new artifacts before replacing only its own app and DMG paths; unrelated output files are preserved. Use `BATCHERBIRD_DIST_DIR=target/release-candidate` to keep the default output untouched. It applies no Developer ID signing or notarization and creates a binary for the build host's architecture. The generated plist derives its minimum macOS version from the executable (highest minimum across slices); when inspection is unavailable, it uses 11.0 as a fallback. Local artifact inspection on October 4, 2026 found `arm64`, Mach-O minimum macOS 11.0 / SDK 26.2, and an ad hoc linker signature without a Team ID. Those are artifact metadata checks, not fresh-install or hardware validation. The October 5 hardware-test preparation does not establish any new hardware pass.

- [ ] Decide supported macOS versions and Intel/Apple Silicon architectures; test each advertised configuration. Do not advertise a universal binary without producing and checking one.
- [ ] Confirm the crate version, intended `v*` tag, bundle versions, binary, resources, and release notes agree. The plist is generated by the script; there is no checked-in resource plist to update.
- [ ] Build/package with `BATCHERBIRD_DIST_DIR=target/release-candidate` for local review; verify failure preserves any previous app and DMG. Verify the packaged app works independently of the repository and finds its embedded UI assets.
- [ ] On a clean user account/Mac, install via the DMG, launch, verify microphone permission prompt, deny then grant access, and capture a real sample.
- [ ] Confirm signing/notarization status and document the actual installation experience. Decide whether distribution without Developer ID signing/notarization is acceptable before public release.
- [ ] Check the DMG mounts, contains the current app and Applications link, and installs/launches correctly. Inspect the actual artifact version and architecture.
- [ ] Record all remaining limitations and failed checks. Do not push the release tag until required gates pass: `.github/workflows/release.yml` uploads `dist/Batcherbird.dmg` on a `v*` tag.

## Evidence log

These results refer to the uncommitted working tree based on `8df2a5c` on October 4, 2026. Targeted tests and fake packaging tools verify software behavior; they do not satisfy hardware, player-listening, or fresh-install gates. That October 4 candidate was rebuilt and packaged locally; its binary matches the release build SHA-256, its plist validates, and its DMG checksum verifies.

| Gate | Revision / environment | Result and evidence | Tester / date |
| --- | --- | --- | --- |
| Formatting and packaging fixtures | Working tree based on `8df2a5c`; local macOS | Cargo formatting and Bash syntax checks passed; all fake-tool packaging transactions and metadata cases passed | Codex / October 4, 2026 |
| Hardware-free follow-up | Working tree; generated 48kHz stereo fixtures | 137 tests passed, 0 failed, 1 hardware test ignored; strict Clippy, formatting, diff checks, and packaging regression fixtures passed. Added real session/WAV/preset workflow coverage and a reusable six-take session generator | Codex / October 5, 2026 |
| Interface follow-up | `2624733` plus accessibility-value follow-up; isolated native demo | Revised layout visually inspected at 1240 × 820 logical pixels. Tab/Return activated controls; native Open loaded six takes, sample selection updated the waveform, native Save wrote six sample records, and combined export produced six stereo 48kHz 24-bit WAVs plus both presets. Accessibility activation/checked-label limitations and remaining native checks are recorded in [desktop checks](DESKTOP_ACCEPTANCE.md) | Codex / October 5, 2026 |
| Refreshed package | Local release-candidate app/DMG | Release build and packaging passed; plist valid, packaged executable matches build SHA-256 `ad9beb3e7e97a454db4271b5602de7b2027018afee8297b6095ae36d5d4b8c9d`, DMG checksum valid. Fresh installation and signing/notarization remain pending | Codex / October 5, 2026 |
| Targeted regression suites | Working tree during implementation | Earlier workspace run (October 4): 134 passed, 0 failed, 1 hardware test ignored; strict Clippy, formatting, and diff whitespace checks passed | Codex / October 4, 2026 |
| Initial artifact metadata | Initial local Apple Silicon candidate | `arm64`, Mach-O minimum 11.0 / SDK 26.2, ad hoc signature without Team ID; no Developer ID signing or notarization. Confirmed on final rebuilt candidate; bundle minimum OS also 11.0 | Codex / October 4, 2026 |
| UI and sessions | Native demo preview, final debug build | Review layout, readable sample list and waveform visually inspected. Automated session tests pass. UI click automation did not produce observable changes; interactive save/load and state checks remain pending | Codex / October 4, 2026 |
| Hardware capture | Pending | Pending | Pending |
| DecentSampler | Pending | Pending | Pending |
| SFZ player | Pending | Pending | Pending |
| Final package and fresh installation | Local arm64 release candidate | App/DMG rebuilt; plist valid; packaged binary SHA-256 matches release build; hdiutil checksum valid. Clean installation, permission flow, and hardware capture remain pending | Codex / October 4, 2026 |

Useful format references: [DecentSampler groups/sample attributes](https://decentsampler-developers-guide.readthedocs.io/en/1.20.0/the-groups-element.html), [SFZ loop crossfade](https://sfzformat.com/opcodes/loop_crossfade/).
