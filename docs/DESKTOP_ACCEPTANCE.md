# Desktop checks without a synthesizer

Use a disposable test session and output directory. Demo mode uses generated audio,
does not connect to MIDI/audio input, and does not load normal preferences or recovery.
Sample playback still uses the Mac's audio output; start at a comfortable volume.

## Review and navigation

Run `cargo run -p batcherbird-vizia -- --demo`.

- Select samples at different notes and velocities. The selected row, note/velocity
  caption, and waveform must agree. Scroll to the last sample and back.
- Play a sample, stop it, then change samples. Playback must stop on selection;
  the playback cursor and Play/Stop label must match the actual playback state.
- Resize to the minimum window size. Settings must remain scrollable, and sample
  review, transport, and primary actions must remain reachable.
- Use Tab and Shift+Tab to reach buttons, disclosure headers, pickers, and the
  instrument-name field. Activate a focused button with the keyboard and check
  that focus stays visible. Check dropdown text against its background.
- Expand advanced settings and collapse them again. Current values must survive.

## Sessions and exports

Generate a reusable fixture with
`cargo run -p batcherbird-vizia --example generate_test_session -- target/manual-qa/quiet-stereo.batcherbird`,
then Open that file in demo mode. It contains six 48kHz stereo takes with quiet
attacks, long releases, and different left/right levels and polarity. It is the
same signal fixture used by the artifact-based workflow tests. The generator
creates a manifest and sibling WAV directory; rerunning replaces this test session.
The quiet attack is deliberately below the default −40dB silence threshold.
Compare exports with **Trim silence** on and off: automatic trimming can remove
that low-level onset, while turning it off keeps the capture padding and full
attack. Export fades still apply. The automated trimming-preservation check uses
a −60dB threshold; it does not establish preservation below the default threshold.

- Save to a new `.batcherbird` file. Confirm the manifest and sibling audio directory
  exist. Change the instrument name, then Open the saved session: cancel the
  unsaved-changes warning once, and confirm the edited name survives. Open again
  and approve replacement; saved settings and recordings must return.
- Cancel Save and Open dialogs. Samples must remain available and controls must
  become usable again. Try opening an invalid file; an actionable error should
  appear without replacing the current recordings.
- Move a copy of the manifest **and its referenced audio directory** together to
  another directory. Open the copy and compare sample count, labels, and playback.
- Export combined WAV/DecentSampler/SFZ into a disposable directory. Confirm
  referenced WAVs exist beside the presets. Listen to the resulting instrument in
  the intended sampler; automated file checks alone do not establish compatibility.

Demo mode deliberately cannot capture or re-record through dummy devices. Automated
workflow tests verify replacement/cancellation retention with generated samples;
real capture and replacement remain part of the hardware acceptance test.

## Other visual states

Launch separate runs with `--demo-state=setup`, `--demo-state=armed`,
`--demo-state=recording`, and `--demo-state=stopping`. These are static layout
fixtures, not simulated recording workers. Check readable guidance, progress,
and disabled conflicting actions. Do not treat a static fixture as proof that a
device operation or cancellation works.

Record the build revision, macOS version, window size, steps attempted, and any
failed expectation. All interactive checks remain pending until performed.
