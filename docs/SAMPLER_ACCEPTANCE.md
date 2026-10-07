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
keys. With a voice log it also requires exactly one active voice for every
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

**Not tested:** DecentSampler actual loading/rendering (no installed player
was found), subjective listening, and experimental loops. These remain
acceptance gates, rather than inferred passes from the SFZ result.
