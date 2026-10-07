# Session resilience acceptance

Hardware is not required for these checks. A session consists of the `.batcherbird`
manifest and its adjacent `.audio-*` directory; move both together. Keep a copy
before deliberately damaging any fixture.

## Automated evidence (October 7, 2026)

`cargo +1.99.0 test -p batcherbird-vizia --lib --offline` passes 27 tests,
including storage failure tests and the actual Vizia model event dispatcher.
`cargo +1.99.0 clippy -p batcherbird-vizia --lib --tests --offline -- -D warnings`
also passes.

Verified behavior:

- Corrupt JSON and unsupported versions fail with the manifest path and an
  explanation; missing WAVs identify the missing sidecar and explain how to move
  a portable session.
- Edited WAVs with three channels, NaN, or infinity are rejected. Existing tests
  also cover invalid MIDI values, incomplete frames, external paths and symlinks.
- A real read-only destination and a destination blocked by a regular file fail
  without changing the existing manifest or its lossless audio.
- A failed manifest replacement removes only its newly created sidecar; existing
  destination contents remain intact.
- Saving a copied manifest cannot remove WAVs referenced by a sibling session
  with the same sanitized filename stem. Corrupt, unreadable or oversized sibling
  `.batcherbird` manifests conservatively prevent cleanup because exclusive
  ownership cannot be established. This can retain old audio directories; keep
  uncertain sidecars until the associated sessions have been checked.
- Open/save failure and dialog cancellation events clear busy state while keeping
  current settings, samples, filename, revision and unsaved-change state. Failure
  replaces a stale success notification. Successful retry clears the old error.
- Replacement confirmation applies only a fully loaded candidate; choosing to
  keep the current session retains its edited settings and original samples.

The storage checks do not simulate power failure, disk exhaustion or concurrent
external edits during cleanup. Atomic manifest replacement prevents ordinary
failed saves from committing a partial session, but is not a claim of recovery
from every filesystem or power-loss scenario.

## Native interface checks still required

Automated model events are not equivalent to exercising native macOS dialogs.
Record the candidate commit and results for each item:

1. Generate a valid fixture with:
   `cargo run -p batcherbird-vizia --example generate_test_session -- target/manual-qa/quiet-stereo.batcherbird`.
   Open it in the app and select a sample.
2. Open a separate `.batcherbird` file containing `{ truncated`. Confirm an
   actionable error appears and the original list, waveform and instrument stay.
3. Copy the valid manifest to a new name beside it; change one sample's `file`
   field to `missing-audio/sample_0000.wav`. Open that copy. Confirm the error names
   the missing audio and the current six samples remain playable/exportable.
4. Cancel Open and Save dialogs. Confirm the same session remains usable and the
   toolbar controls become enabled again.
5. Edit the instrument name, open the original fixture, choose **Keep current
   session**, then repeat and choose **Replace session**. Confirm both outcomes.
6. Save into a deliberately read-only test folder (outside any valuable data).
   Confirm the error explains retrying in a writable folder. Then save to a
   writable folder, reopen the result, and check settings and sample audio.
7. Confirm long error paths remain readable by scrolling or wrapping at the
   supported minimum window size.

Native results: pending unless separately recorded in `DESKTOP_ACCEPTANCE.md`.
