# Exporting a local diagnostics report

Choose **Export diagnostics…** at the bottom of the settings sidebar and select a
JSON destination in the native Save dialog. The report is a snapshot taken when
you press the button. It is saved locally; Batcherbird never uploads it. Canceling
or failing to save does not replace, edit, or mark your session saved.

The readable, versioned JSON includes:

- Application version, operating system and architecture.
- Selected MIDI/audio device names and indices, routing and gain.
- Capture note range, spacing, velocity layers, timing and MIDI channel.
- Export format, trimming and loop choices.
- Current app state, dirty status, recorded sample count and rate/channel
  summaries, plus the selected sample's numeric metadata.
- Present stream/worker handles and active operation flags.
- Recent error presence/category, without the backend's raw error text.

No recorded audio, waveform values, instrument/session names, recording
timestamps, session/export/preferences paths, account details, or separately
queried hostname/user identifiers are collected. **Device names are included and
can contain personal text. Review the JSON before sharing it.** Unselected and
remembered device labels are omitted.

Unknown values are explicit `null` with an explanation. The current build does
not embed a verified Git revision or expose the active stream rate/channel/buffer
configuration or OS version to the UI. Recorded rate metadata is not a live
stream configuration. The engine's performance counters are not populated by the
current callback paths, so zero observations do not claim healthy performance.
Stream handle presence also does not establish that hardware capture is working.

Reports use an atomic destination replacement: an unsuccessful write/commit keeps
any previous destination intact and removes only its new temporary file. Existing
session audio and settings are unaffected. Diagnostics export is blocked while
another recording, session, export or test-note operation is busy. Exporting the
report temporarily blocks those operations to avoid competing native dialogs.

Automated checks cover selected configuration, omission of private application
fields/raw errors/audio, stale device selections, JSON round-trip, real read-only
folders, failed replacement cleanup, and actual Vizia completion/cancellation/
error event dispatch preserving dirty session data. Native Save-dialog and JSON
export acceptance remain a separate desktop check.
