# Native playback acceptance

The ignored BlackHole test exercises the production CPAL output callback and preview preparation, completion drain, Stop, and replacement lifecycle against an actual virtual driver. It does not open the default input/output, a microphone, or speakers; it does not change system audio defaults.

Run only with **BlackHole 16ch** installed, already available at 48kHz, and channels **3–4** reserved for this test. The test explicitly finds that named input and output. Before each case it requires silence on channels 3–4 and refuses to proceed if another source is active. All other input channels are discarded. All other output channels receive zeros.

```sh
BATCHERBIRD_PLAYBACK_ACCEPTANCE_DIR=/tmp/batcherbird-playback-acceptance \
  cargo test -p batcherbird-core --lib \
  preview_player::driver_acceptance::blackhole_preview_driver_acceptance \
  --offline -- --ignored --exact --nocapture
```

A sandbox that exposes no audio devices cannot run this acceptance check. The ordinary unit tests remain hardware-free; this test is ignored unless explicitly requested.

## Cases and assertions

- Three complete stereo plays: a quiet 100ms attack, sustained 440Hz tone, long decaying release, and a distinct terminal marker. Captured active-frame count, sample values, channel differences, and final marker must match the reference after stream cleanup.
- Mono source: the production frame mapper duplicates it to two channels, then the **test fixture** pads those channels into a 16-channel buffer at channels 3–4.
- 44.1kHz source through an explicitly configured 48kHz output: the production resampler must retain the one-second duration and approximately 440Hz pitch.
- Stop during a two-second take: completion becomes immediate, the captured prefix matches the source, and subsequent output is silent rather than continuing the take.
- Replace a 330Hz take with a 660Hz take: the new take matches its reference after a 50ms startup allowance, with no old-note bleed. Its tail must finish before stream destruction.

The loopback cases feed a **16-channel routed source** to the production player so that channels 1–2 and 5–16 remain silent. Mono/stereo mapping happens in test-only fixture preparation using the existing production `map_frame` function; ordinary mapping tests separately exercise all supported channel topologies. There is no acceptance-only routing branch or public target-device option in the shipped player.

`report.json` contains measured frame counts, maximum absolute sample error, correlation, and pitch. Stereo float WAVs contain the isolated generated capture and reference. Stop error/correlation are measured from the actual captured prefix. The report records `listener_verified: false`.

## Observed run, October 8, 2026

All seven cases passed on the installed BlackHole 16ch driver. Complete 48kHz, repeated, mono-mapped, and 44.1→48kHz takes contained 47,999 active frames over their one-second reference, with maximum absolute error 0 and correlation 1. Measured pitch was approximately 439.95Hz; the switched take measured approximately 660.08Hz. The terminal marker survived natural completion and cleanup. Stop ended the output around the requested 150ms point and left silence.

The artifacts were written to `/tmp/batcherbird-playback-acceptance-2026-10-08`. They are local QA evidence, not repository fixtures or a listening certification.

Still pending: physical speaker/headphone listening, physical-interface behavior, device-unplug/error behavior on real hardware, and a native GUI Play/Stop/selection pass. The driver check establishes callback and lifecycle behavior, not subjective audio quality or synth sampling acceptance.
