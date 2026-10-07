# Desktop checks without a synthesizer

Use a disposable test session and output directory. Demo mode uses generated audio,
does not connect to MIDI/audio input, and does not load normal preferences or recovery.
Sample playback still uses the Mac's audio output; start at a comfortable volume.

## Review and navigation

Run `cargo run -p batcherbird-vizia -- --demo`. For a reproducible 1040 × 720
minimum-size fixture, add `--demo-minimum`; combine it with any `--demo-state`
fixture. `BATCHERBIRD_DEMO_MINIMUM=1` is also supported for isolated QA app bundles.

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
failed expectation. Unperformed checks remain pending.

## Recorded native checks — October 5, 2026

An isolated demo app built from `2624733` plus the accessibility-value follow-up
was inspected on the local Mac at 1240 × 820 logical pixels. The revised review
layout, collapsed settings, expanded export section, sample list, and waveform
were readable. Tab focus was visible; Return activated disclosure headers and
buttons. A native Open dialog loaded the six-take fixture, and keyboard sample
selection updated the caption and waveform to C4 / velocity 64.

The native folder chooser selected an absolute disposable output directory.
Combined export reported eight files, confirmed on disk as six stereo 48kHz
24-bit WAVs plus DecentSampler and SFZ presets. The native Save dialog wrote
`native-saved.batcherbird` with six sample records and sibling audio files.
These checks establish the observed load, selection, save, and export paths;
they do not establish player compatibility or audio quality.

Vizia 0.3 does not forward `.name()` to AccessKit labels in this build. Explicit
text values now make ordinary action buttons identifiable in the macOS
accessibility tree. Checked sample rows and disclosure toggles still lack useful
labels in that tree, and accessibility mouse activation produced no observable
changes. Keyboard activation worked. Full VoiceOver support needs a framework
fix and native verification. Synthetic modifier-key checks were inconclusive;
verify Shift+Tab and application shortcuts manually.

Playback/listening, minimum-size resizing, edited-session replacement warnings,
invalid-file and dialog cancellation recovery, and the other static layouts
still need hands-on checks. Automated session and export regressions supplement
these observations without satisfying the remaining native gates.

In a follow-up on the same isolated demo, an instrument-name edit reached the
model and displayed **Modified**. Opening a valid saved session then reached a
pending native confirmation, but the automation did not expose its buttons;
the app later returned to usable controls with the edited name and six samples
intact. This was inconclusive for the system alert's appearance and button behavior.
The follow-up replaces that alert with an in-app banner offering **Keep current
session** and **Open replacement**, and retains a validated candidate without
changing current settings or audio until the user chooses. Model regressions
verify both choices and harmless stale responses. Native automation disconnected
while retesting the rebuilt app, so the new banner's visual and keyboard check
remains pending. A code audit also found preview
completion could drop the stream before its final queued buffer played. The fix
waits for a CPAL timestamp-based playback deadline with a minimum allowance of
100ms or two output buffers, whichever is longer, because backend timestamps may
omit device latency. Regression tests check drain timing and immediate Stop.
This does not replace a listening check on the Mac.


## Desktop polish checks — October 7, 2026

An isolated demo app built from the desktop-readiness working branch was launched
at the configured 1040 × 720 logical content size. A native screenshot showed the
review title, selected sample caption, waveform, both secondary sample actions,
Play/Export transport, folded keyboard header, toolbar actions, and expanded export
settings within the window. The visible demo notice used its natural height, not
the full 180px notification limit. This verifies the observed minimum-size review
layout; it does not establish other states or dynamic resizing.

Notifications now share a bounded scrolling region so a long error or replacement
notice cannot consume the whole stage. The replacement filename occupies a separate
ellipsis line, keeping both choices available for long names. Setup guidance also
scrolls independently, leaving its transport and keyboard outside the scrollable
content. These additional states still require native visual/keyboard checks.

The sample list now enables single selection and selection-following-focus, routes
row activation through the same list selection callback as arrow navigation, and
clears an existing selection before selecting a clicked/activated row. This avoids
the framework's repeated-row toggle clearing internal selection/focus. The native
arrow-key and repeat-selection checks remain pending after the final rebuild.

Native automation can stall while binding an app: selecting this QA app took about
seven minutes despite a requested ten-second tool timeout. Once bound, screenshot
capture completed in a few seconds. Unperformed desktop checks remain pending.

## Accessibility implementation checks — October 7, 2026

The earlier October 5 native label/activation limitation prompted a focused patch
of `vizia_core` 0.3.0, documented in `vendor/vizia_core/PATCHES.md`. Generated
AccessKit nodes now receive application names as labels, expose enabled built-in
button Click actions, and reject stale disabled activation requests. Plain action
controls no longer repeat their names as values. Input meters have names and
numeric levels; keyboard audition supports Tab focus, arrows/Home/End, and holding
Space, with note-off on release and focus loss.

Three integration tests inspect the real generated AccessKit tree and dispatch
activation/key events. They pass alongside a production-stage regression that
activates the same sample twice, then moves next and previous through the actual
list. These results verify implementation behavior. Native VoiceOver speech,
AXPress activation, focus announcements, and meter usability remain manual gates;
the compact screenshot above used the earlier debug binary and does not verify
this accessibility patch.

## Review follow-up — October 7, 2026

Independent review reproduced initial keyboard navigation jumping to row zero
when the model already selected another sample. The production list now binds its
selection to the current sample. The focused Vizia patch replaces bound selections
instead of retaining old indices, keeping reverse navigation and external selection
changes consistent. The sample list is disabled during conflicting operations.
Two production-stage event tests pass: repeated activation followed by next/previous,
and initial/external selection followed by next/previous. Native keyboard and
VoiceOver acceptance remain pending.
