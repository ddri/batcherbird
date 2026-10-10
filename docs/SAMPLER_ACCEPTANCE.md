# Independent sampler acceptance without hardware

This checks an actual player's rendered audio using generated, pitch-correct
samples. Exporter unit tests alone do not establish player compatibility.
DecentSampler and SFZ have separate acceptance results; a pass in one does not
approve the other. Hardware capture and listening remain separate checks.

## Generate the instrument and MIDI probes

Run from the repository root, choosing a **new directory** for each run:

```sh
cargo run -p batcherbird-core --example generate_sampler_acceptance -- target/sampler-acceptance
mv "target/sampler-acceptance/instrument with spaces" "target/sampler-acceptance/moved instrument with spaces"
```

The generator refuses an existing output directory. It exports six stereo,
48kHz/24-bit takes through the production combined DecentSampler/SFZ exporter,
with trimming, normalization, and automatic looping disabled. Each take lasts
two seconds and has short fades. Roots are MIDI 60, 63, 66; recorded velocities
are 64 and 127. The tones have their actual root-note frequency.

The high layer has a second harmonic at half the fundamental's amplitude; the
low layer has none. Third harmonics identify roots 60/63/66 at ratios
0.1/0.2/0.3. The right channel is the left multiplied by -0.35. These markers
make wrong roots, layers, mono mixing, and channel swaps measurable.

`boundary-sweep.mid` plays MIDI 59–67 at velocities 1, 64, 95, 96, 127:
45 probes, each starting two seconds apart, held for 500ms. The accompanying
`probes.csv` defines expected root/layer and timing independently of preset
parsing. MIDI 59 and 67 should be silent. Within 60–66, roots should map
60–61 → 60, 62–64 → 63, 65–66 → 66; velocities 1–95 → low,
96–127 → high.

## Render with sfizz

Use an existing `sfizz_render` from the official SFZTools project. Its
[build instructions](https://www.sfz.tools/sfizz/development/build/) describe
the renderer; installing a plug-in is unnecessary. Record the version/source
revision and build options used. Nothing in this repository installs sfizz.

For sfizz **1.2.3**, prefer float WAV output: the stock renderer converts to
16-bit PCM, which loses the fixture's velocity-1 signal. The supplied patch
changes only WAV encoding and the buffer written, preserving the synthesis
engine and MIDI handling. Apply it to the pinned official source before
building; retain its license files in that temporary checkout:

```sh
git clone --branch 1.2.3 --depth 1 --recurse-submodules --shallow-submodules \
  https://github.com/sfztools/sfizz.git /private/tmp/batcherbird-sfizz-1.2.3
git -C /private/tmp/batcherbird-sfizz-1.2.3 rev-parse HEAD
# Expected: 4e70dc0bef53b41f2853ed46e26f5911114c92d0
git -C /private/tmp/batcherbird-sfizz-1.2.3 apply \
  /absolute/path/to/batcherbird/scripts/sfizz-render-float.patch
cmake -S /private/tmp/batcherbird-sfizz-1.2.3 \
  -B /private/tmp/batcherbird-sfizz-1.2.3/build -G Ninja \
  -DCMAKE_BUILD_TYPE=Release -DSFIZZ_JACK=OFF -DSFIZZ_SHARED=OFF \
  -DSFIZZ_TESTS=OFF -DSFIZZ_RENDER=ON -DENABLE_LTO=OFF \
  -DCMAKE_POLICY_VERSION_MINIMUM=3.5 \
  -DCMAKE_OSX_DEPLOYMENT_TARGET=11.0 -DPROJECT_SYSTEM_PROCESSOR=aarch64 \
  -DCMAKE_CXX_FLAGS=-Wno-missing-template-arg-list-after-template-kw
cmake --build /private/tmp/batcherbird-sfizz-1.2.3/build --target sfizz_render -j 4
```

These build flags describe the tested Apple Silicon/AppleClang 17 setup;
`aarch64` avoids this older source's ARM32-only compiler flags. Adapt target
settings for other platforms. The resulting CLI lives in
`build/library/bin/sfizz_render`. No `install` step is required. A newer
renderer may support float output directly; verify its options first.

Replace the fixture and renderer paths below with absolute paths. Run from
another working directory to check that sample references resolve relative
to the moved preset, rather than the shell's current directory:

```sh
cd /private/tmp
/absolute/path/to/sfizz_render \
  --sfz "/absolute/path/to/target/sampler-acceptance/moved instrument with spaces/Acceptance.sfz" \
  --midi /absolute/path/to/target/sampler-acceptance/boundary-sweep.mid \
  --wav /absolute/path/to/target/sampler-acceptance/sfizz-render.wav \
  --log /absolute/path/to/target/sampler-acceptance/sfizz-voices.csv \
  --samplerate 48000 --blocksize 256
```

Back in the repository root:

```sh
python3 scripts/check-sampler-render.py target/sampler-acceptance \
  target/sampler-acceptance/sfizz-render.wav \
  --voice-log target/sampler-acceptance/sfizz-voices.csv \
  --report target/sampler-acceptance/sfizz-report.json
python3 scripts/test-sampler-render.py
```

The checker measures 150–350ms into each probe, checks pitch within 10 cents,
root/layer harmonics and stereo identity, and rejects audible out-of-range
keys in either channel. It rejects unequal channel lengths, incomplete
stereo frames, and non-finite float audio before accepting a probe.
With a voice log it also requires exactly one active voice for every
in-range note and zero outside. Without a voice log, it cannot rule out
duplicate identical regions by spectral analysis. Velocity 1 may approach
16-bit quantization limits; use float output to measure it. All in-range
velocities must pass pitch, root/layer, and stereo checks. A failure is a
failure or a required investigation, never an automatic waiver.

The script accepts stereo integer PCM WAV at 16/24/32-bit or IEEE float WAV
at 32-bit. Prefer float for this quiet velocity-1 probe. It intentionally
does not handle effects, host latency, altered MIDI timing, WAV extensible,
or mono renders. Host renders must start at time zero, with the sweep unmodified.

## DecentSampler in a host

Install/use DecentSampler only through the user's normal approved setup.
Load the **moved** `Acceptance.dspreset`, put `boundary-sweep.mid` on its
instrument track, disable host effects, and render from time zero to a stereo
float WAV covering the whole 90-second sweep. Run the same audio checker with
that render, omitting `--voice-log`. Record the host and player versions.
Inspect the player's zone/voice display at velocity and key boundaries to
rule out duplicate identical zones, which the audio checker alone cannot do.
Listen for clicks, incorrect tails, and unexpected loops in both players.

The [official DecentSampler sample/group documentation](https://decentsampler-developers-guide.readthedocs.io/en/latest/the-groups-element.html)
defines relative sample paths, root/key ranges, and velocity ranges used by
these exports. Reading the specification does not replace opening the preset.

## Record evidence

```text
Date / tester / Batcherbird source revision:
Player / exact version or source revision / host / build options:
Moved preset path / render path / JSON report / voice log:
Moved sample resolution: Pass / Fail / Not tested
Pitch and root mapping: Pass / Fail / Not tested
Velocity boundaries: Pass / Fail / Not tested
Exactly one zone/voice: Pass / Fail / Not tested
Stereo identity: Pass / Fail / Not tested
Listening (clicks/tails/no unexpected loops): Pass / Fail / Not tested
Failures and remaining cases:
```

This baseline does not validate experimental sustain loops, every SFZ player,
hardware recording, sample-rate negotiation, or every possible instrument.

## Observed result — October 7, 2026

The moved, space-containing SFZ export loaded in the isolated sfizz 1.2.3
renderer described above, executed from `/private/tmp`. The float-output
renderer passed **45/45 probes at 48kHz, 256-frame blocks**, including all
velocity-1 probes, spectral root/layer checks, pitch, channel identity, silent
outside keys, and exactly one active voice for each in-range probe.
The production exporter was unchanged from Batcherbird base revision
`e47220f7ebe683f26ad5aa0f75a76bed2b95b258`. Every in-range probe's best
pitch match was zero cents on the checker's five-cent grid; maximum relative
stereo residual was 0.000269.

The stock 16-bit renderer initially passed 38/45 with the checker's less
strict low-level checks; velocity-1 output became constant one-LSB DC and
could not establish pitch. Switching only output encoding to float preserved
measurable audio; no exporter change or synthesis-engine patch was needed.
The final checker applies full precision checks to every velocity.

Local evidence (generated artifacts, excluded from Git):

- `target/sampler-acceptance-20261007-marked/sfizz-float-report.json`
- `target/sampler-acceptance-20261007-marked/sfizz-float-voices.csv`
- `target/sampler-acceptance-20261007-marked/sfizz-float-render.wav`
- Render SHA-256: `715c5f16f4c971c5a4f9e965c997bbafda5f4b1f396e569c5b22b4e1a9cdae88`
- Renderer patch SHA-256: `3ce52a1b85b7ed5047ced66cea566e7ab41deb700b849a8331245ae4476272bb`

At the October 7 baseline, DecentSampler rendering, subjective listening,
and experimental loops were not tested. The October 8 follow-up below adds
actual DecentSampler engine evidence without treating the SFZ pass as proof.

## Actual DecentSampler VST3 follow-up — October 8, 2026

The official [download page](https://store.decentsamples.com/downloads/decent-sampler/versions)
provides DecentSampler **1.36.1** directly, without an account. Its macOS
package was extracted into `/private/tmp`; nothing was installed into
Applications or the system plug-in directories. Package verification
reported Developer ID Installer **Decidedly, LLC (5K8EG37W74)** and trusted
Apple notarization. The extracted VST3 passed strict code-signature checks.

The player ran through Spotify's [Pedalboard MIDI instrument host](https://spotify.github.io/pedalboard/reference/pedalboard.html),
version **0.9.26**, with Python 3.12 and Mido **1.3.3**. The new
`scripts/render-decent-sampler.py` reads the actual generated MIDI file and
restores the exported instrument's XML into the player's VST3 component
state. It preserves musical mappings and sample attributes, adds the
player's private sample-directory/library context, and preserves its JUCE
wrapper/private data. This is **state restoration into the real player
engine**, rather than a native preset-picker test or a byte-identical preset
load. The tested state protocol is specific to this player/host combination;
an unknown state format fails instead of silently substituting another player.

The complete instrument folder was relocated to `moved Élan synth 你好`,
covering spaces and non-ASCII directory names. Actual rendering resolved all
six adjacent sample WAVs. The 45-probe run at **48kHz / 256-frame blocks**
passed **31/45**; the remaining probes expose a reproducible compatibility
discrepancy:

| Probe | Expected | Observed in this VST3 host |
| --- | --- | --- |
| All seven in-range keys, velocity 1 | Low layer sounds | Silent |
| All seven in-range keys, velocity 96 | High layer sounds | Low layer sounds |
| Other 31 probes | Expected pitch/root/layer/stereo or outside-key silence | Pass |

A separate sweep of **all velocities 1–127 at MIDI 60** reproduced silence
only at velocity 1 and the high-layer transition at **97**, rather than 96.
This is consistent with a floating-point velocity conversion/truncation at
the host/player boundary, but the responsible layer is **not established**.
The exporter still specifies the correct musical MIDI boundaries: low
1–95, high 96–127. Do not shift export zones to compensate without comparing
another host or the native player. This DecentSampler gate is **not passed**.

An additional velocity-127 C4 note held for 3.5 seconds sounded normally
(left peak approximately 0.179) and was exactly silent in both channels after
2.1 seconds, demonstrating that this two-second, loops-off sample did not
acquire an unexpected repeating sustain. This numerical check does not
establish subjective click/release quality. Pedalboard exposes no player
voice counter here, so audio alone cannot rule out duplicate identical zones.

### Repeat the isolated player test

Use a new temporary directory and fixture output. Verify the package/plugin
signatures before loading it. The tested download's SHA-256 is
`300ebf938d127bec0103009abd49e8eb62200230fae1e4cf54b88a6427edc026`.

```sh
mkdir -p /private/tmp/batcherbird-decent-1.36.1
curl -fL https://cdn.decentsamples.com/production/builds/ds/1.36.1/Decent_Sampler-1.36.1-Mac.zip \
  -o /private/tmp/batcherbird-decent-1.36.1/download.zip
unzip /private/tmp/batcherbird-decent-1.36.1/download.zip \
  -d /private/tmp/batcherbird-decent-1.36.1/archive
pkgutil --check-signature /private/tmp/batcherbird-decent-1.36.1/archive/Decent_Sampler-1.36.1-Mac.pkg
pkgutil --expand-full /private/tmp/batcherbird-decent-1.36.1/archive/Decent_Sampler-1.36.1-Mac.pkg \
  /private/tmp/batcherbird-decent-1.36.1/expanded
codesign --verify --strict --verbose=2 \
  /private/tmp/batcherbird-decent-1.36.1/expanded/VST3.pkg/Payload/Library/Audio/Plug-Ins/VST3/DecentSampler.vst3
python3.12 -m venv /private/tmp/batcherbird-decent-1.36.1/python312
/private/tmp/batcherbird-decent-1.36.1/python312/bin/python -m pip install pedalboard==0.9.26 mido==1.3.3
mkdir -p /private/tmp/batcherbird-decent-1.36.1/user
```

Generate/move a fresh fixture using the earlier section. Then run:

```sh
CFFIXED_USER_HOME=/private/tmp/batcherbird-decent-1.36.1/user \
  /private/tmp/batcherbird-decent-1.36.1/python312/bin/python scripts/render-decent-sampler.py \
  /private/tmp/batcherbird-decent-1.36.1/expanded/VST3.pkg/Payload/Library/Audio/Plug-Ins/VST3/DecentSampler.vst3 \
  "target/sampler-acceptance/moved instrument with spaces/Acceptance.dspreset" \
  target/sampler-acceptance/boundary-sweep.mid \
  target/sampler-acceptance/decent-render.wav \
  --tail-probe target/sampler-acceptance/held-note.wav \
  --velocity-sweep target/sampler-acceptance/velocity-sweep.csv
python3 scripts/check-sampler-render.py target/sampler-acceptance \
  target/sampler-acceptance/decent-render.wav \
  --report target/sampler-acceptance/decent-report.json
python3 scripts/test-decent-render.py
```

`CFFIXED_USER_HOME` keeps this player's logs/database in the temporary user
directory; it leaves the real home directory unchanged. The renderer refuses
existing output paths, emits float WAV to preserve quiet probes, and records
host/player versions and load method beside the render. The four-second
held-note check requires actual early sound, rejecting an entirely silent
render. The optional velocity sweep reports every observed layer in CSV.

Local evidence, excluded from Git, is under
`target/sampler-acceptance-20261008-decentsampler/`: `decent-confirm.wav`,
`decent-confirm.metadata.json`, `decent-confirm-report.json`,
`held-note-confirm.wav`, and `velocity-sweep-confirm.csv`.

**Remaining gates:** compare the velocity discrepancies in a second/native
host, native `.dspreset` picker loading, actual voice/zone count, subjective
listening, and experimental sustain loops. No compatibility workaround has
been applied to the exporter.

## Independent VST3 SDK diagnosis — October 9, 2026

The discrepancy was reproduced in a second host built directly against
[Steinberg's official VST3 SDK](https://github.com/steinbergmedia/vst3sdk),
revision `3cdf9ca5d1f5b1b21e0a86832aa4abe55607bd96` (SDK 3.8.1).
`scripts/render-decent-vst3` uses the SDK's module loader, component/controller
interfaces, event list, and float audio buffers. It sends normalized VST3
note events directly to DecentSampler; no Pedalboard/JUCE host conversion,
native editor, or audio device is involved. The SDK's
[note-on interface](https://github.com/steinbergmedia/vst3_pluginterfaces/blob/4f547e8e102b47de4a8b8aaf343c73b700786372/vst/ivstevents.h)
defines velocity as a float in the 0–1 range.

The production export still passed **31/45** in this independent host,
with the same fourteen failures as Pedalboard. The standard SDK render was
byte-identical to the original Pedalboard render: SHA-256
`6bc3efd7416f3f69398814296c4a3b6dbd9809d2d2ace0a33a889f18e50cae0c`.
Per-event CSV traces record the exact float delivered to the player.

| Controlled change | Result | Implication |
| --- | --- | --- |
| Standard float MIDI velocity / 127 | 31/45 | Failure reproduces outside Pedalboard |
| Move incoming float one representable step upward | 31/45; identical audio | An incoming-float nudge alone does not fix it |
| Restore minimal private path context, without factory settings | 31/45 | Inherited factory metadata is not required to reproduce it |
| Set private velocity-preprocessor output minimum to 1 | 31/45 | That setting alone does not fix it |
| Subtract 0.00001 from diagnostic group velocity thresholds, leaving upper 127 unchanged | 45/45 | Failure is sensitive to the player's velocity-zone boundary comparisons |

These controls isolate the discrepancy to the **DecentSampler component's
velocity-zone comparison/conversion path**, rather than a Pedalboard-only
problem or incorrect integer export mapping. The proprietary player's exact
internal implementation has not been inspected. The incoming-float trace
alone is not proof of a particular truncation operation: the unsuccessful
`nextafter` control is retained to make that limitation explicit.

The fractional-threshold result is **diagnostic evidence, not a compatibility
pass or an exporter workaround**. It alters the component's musical mapping
for the control run, and fractional attributes are outside the documented
integer mapping baseline. Production presets continue to use low 1–95 and
high 96–127. The standard DecentSampler acceptance gate remains failed;
native file-picker/another player API comparison and a vendor resolution
remain follow-up work. Subjective listening and identical duplicate-voice
detection are still untested.

### Reproduce the independent host

This small host is macOS-specific. Keep the SDK checkout/license files in
an isolated directory; nothing is installed or added to the Rust dependency
graph. Fetch the recorded SDK revision and its matching submodules:

```sh
git clone --no-checkout https://github.com/steinbergmedia/vst3sdk.git \
  /private/tmp/batcherbird-vst3-sdk-20261009
git -C /private/tmp/batcherbird-vst3-sdk-20261009 checkout --detach \
  3cdf9ca5d1f5b1b21e0a86832aa4abe55607bd96
git -C /private/tmp/batcherbird-vst3-sdk-20261009 submodule update --init --recursive
cmake -S scripts/render-decent-vst3 -B /private/tmp/batcherbird-decent-vst3-build \
  -G Ninja -DCMAKE_BUILD_TYPE=Release -DCMAKE_OSX_DEPLOYMENT_TARGET=11.0 \
  -DVST3_SDK_ROOT=/private/tmp/batcherbird-vst3-sdk-20261009
cmake --build /private/tmp/batcherbird-decent-vst3-build \
  --target batcherbird_decent_vst3 -j 4
mkdir -p /private/tmp/batcherbird-decent-vst3-user
```

With the verified extracted DecentSampler package from the preceding section,
generate its own initial component state. Use fresh output names for each
run; both host and preparation script refuse existing output files:

```sh
CFFIXED_USER_HOME=/private/tmp/batcherbird-decent-vst3-user \
  /private/tmp/batcherbird-decent-vst3-build/batcherbird_decent_vst3 \
  /private/tmp/batcherbird-decent-1.36.1/expanded/VST3.pkg/Payload/Library/Audio/Plug-Ins/VST3/DecentSampler.vst3 \
  --seed /private/tmp/batcherbird-decent-vst3-seed.bin
python3 scripts/render-decent-vst3/prepare-state.py \
  /private/tmp/batcherbird-decent-vst3-seed.bin \
  "target/sampler-acceptance/moved instrument with spaces/Acceptance.dspreset" \
  target/sampler-acceptance/component-state.bin
CFFIXED_USER_HOME=/private/tmp/batcherbird-decent-vst3-user \
  /private/tmp/batcherbird-decent-vst3-build/batcherbird_decent_vst3 \
  /private/tmp/batcherbird-decent-1.36.1/expanded/VST3.pkg/Payload/Library/Audio/Plug-Ins/VST3/DecentSampler.vst3 \
  target/sampler-acceptance/component-state.bin \
  target/sampler-acceptance/probes.csv \
  target/sampler-acceptance/sdk-standard.wav \
  target/sampler-acceptance/sdk-events.csv
python3 scripts/check-sampler-render.py target/sampler-acceptance \
  target/sampler-acceptance/sdk-standard.wav \
  --report target/sampler-acceptance/sdk-standard-report.json
```

The optional host `--nextafter` flag and preparation script
`--diagnostic-fractional-thresholds` flag reproduce the respective controls.
Use separate component-state/render/report paths, and keep those reports
distinct from standard acceptance. The SDK host follows the generated
`probes.csv` timings and holds each note for 500ms; it bypasses MIDI-file
decoding to investigate the actual VST3 note-event interface.

Local evidence is under `target/sampler-acceptance-20261009-sdk/`, including
`standard-report.json`, `standard-events.csv`, `standard-confirm-report.json`,
`nextafter-report.json`, `minimal-context-report.json`,
`min-output-one-report.json`, and `fractional-threshold-report.json`.
The fractional control render SHA-256 is
`0c921d08dc6bad1e54d97ff1dce1bffd5112ec3cd89f030cf4b8e456e6ff579d`.

The SDK host's CLI preflight rejects seed mode with extra arguments and refuses
existing destinations in either mode. Render and event trace must resolve to
distinct paths (including directory symlink aliases). Probe numbers must parse
completely, be finite, and lie within the documented bounds: start 0–3600s,
note 0–127, velocity 1–127. These checks run before loading the player; sample
rate and block size are fixed at 48kHz and 256 frames.

Run the plugin-free overwrite, path-collision, and numeric-input regressions
against the compiled host:

```sh
python3 scripts/render-decent-vst3/test-cli.py \
  /private/tmp/batcherbird-decent-vst3-build/batcherbird_decent_vst3
```

All ten CLI tests passed on October 9, including the previously reproduced
extra-argument seed overwrite and case-insensitive destination collision. Both
outputs are reserved before loading the player with atomic exclusive creation
and written through those original file descriptors. Failure removes only
reservations whose paths still identify the owned inode. Seed output uses the
same exclusive creation. Four state-format and ten audio-analyzer tests
also passed using the isolated Python 3.12 environment. After the CLI fixes,
`exclusive-buffered-fix.wav` and `exclusive-buffered-fix-report.json` repeat the standard 31/45
result; the WAV SHA-256 remains
`6bc3efd7416f3f69398814296c4a3b6dbd9809d2d2ace0a33a889f18e50cae0c`.

### Native standalone attempt — October 9, 2026

The extracted official DecentSampler 1.36.1 standalone app passed complete
code-signature verification and launched via CUA. App binding took 252.7 seconds.
The welcome UI initially offered store-account setup; no account was created and
no credentials or agreements were entered. After dismissing the welcome overlay,
the main keyboard and FILE button were visible. Native button, coordinate and
keyboard attempts did not expose the local preset menu or a file picker, so
**native `.dspreset` loading was not verified**. This automation limitation does
not establish a player defect. The second-host evidence above uses the actual
component engine and state restoration, with its stated limits.
