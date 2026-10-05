# A 30-minute hardware acceptance session

Use the packaged local candidate with a real synth and interface. This is a test procedure, **not a record of a passed hardware test**. Keep automatic loops **off** for this first pass. Record failures and keep the affected session/WAVs; the full [release checklist](RELEASE_CHECKLIST.md) still applies.

## 1. Connect and check the signal — 5 minutes

Connect Mac MIDI out → synth MIDI in, and synth audio out → interface input. Set the synth to receive **MIDI channel 1**, which the current desktop capture uses. Choose a patch with a clear attack and an audible release of roughly one second. Choose the intended MIDI output, audio input, input channels, and zero app input gain. Note the interface's actual sample rate.

Click **Check input signal**, then **Test note**. The correct patch should sound and the meters should move without clipping. Set a comfortable level at the synth/interface. For stereo hardware, use a patch with distinguishable left/right sound and confirm both channels retain their identity. For a mono synth, select the connected mono-left or mono-right input; mark the stereo check **not tested**, rather than treating duplicated mono as stereo evidence.

**Pass:** the chosen device/channel carries the expected signal, the note releases, and there are no stuck notes or unexplained input errors.

## 2. Capture six samples and inspect them — 5 minutes

Set this small plan:

| Setting | Value |
| --- | --- |
| Instrument name | Acceptance patch |
| Start / end | MIDI **60–66** (C4–F♯4 in Batcherbird) |
| Note spacing | Every third note |
| Velocity layers | **2** |
| Hold | **1.0 second** |
| Release tail | **1.5 seconds** |
| Automatic loops | Off |

Start recording. Expect **six samples**: roots **60, 63, 66**, each at velocities **64 and 127**. Select every row and play it. The waveform, label, and sound must follow the selected sample. Check pitch, channel identity, attack, and release; listen for clicks or abruptly cut tails. The original take includes capture padding, so its total length need not equal exactly 2.5 seconds.

**Pass:** six complete takes, correct notes/velocities, usable attacks and release tails, no audible recording defects. If the chosen patch has a longer release, lengthen the tail and repeat; do not call an intentionally short setting a successful long-tail test.

## 3. Replace a take, save, and reopen — 5 minutes

Select one row, change the synth patch slightly so the replacement is recognizable, and click **Re-record sample**. Expect that row to change while the other five remain intact. Restore the intended patch if needed, then save as `acceptance.batcherbird` in a new test folder.

Quit, reopen, and open the saved session. Expect six samples with the same audio and settings. Move the `.batcherbird` file **and its referenced `.audio-…` directory together** into another folder and reopen again.

**Pass:** replacement affects only the selected take; saved audio/settings survive restart and the folder move. Automatic recovery is extra protection after a batch, not a substitute for this explicit save.

## 4. Stop a partial batch — 5 minutes

Keep the saved six-sample baseline. Start a **new capture** with range **60–66**, **every note**, two layers, a **5-second hold**, and a **2-second release**. After at least one take completes, stop during a later held note.

Expect **Stopping** while the worker releases the note, then review with completed takes retained and the unfinished take excluded. There must be no hanging note. Save this partial result separately if it exposes a problem. Repeat briefly, stopping during a release tail. Reopen the baseline session afterward.

**Pass:** both stops return to a usable review state, preserve finished takes, discard the interrupted take, and allow another capture. Record the observed stopping time; this test does not establish a MIDI latency specification.

## 5. Move the export and play its boundaries — 10 minutes

Export the baseline to a new folder using the combined **DecentSampler + SFZ** format. Move the **whole export folder**, including adjacent WAVs, to another location. Load its `.dspreset` in DecentSampler and its `.sfz` in an SFZ player. Note both player versions. If a player is unavailable, mark that test **not tested**.

Play MIDI **60, 61, 62, 63, 64, 65, 66** at velocities **1, 64, 95, 96, 127** in each player. These presets should map:

| Played keys | Recorded root |
| --- | --- |
| 60–61 | 60 |
| 62–64 | 63 |
| 65–66 | 66 |

Velocities **1–95** use the recorded velocity-64 layer; **96–127** use velocity-127. Expect every tested key to sound at the correct pitch, with one zone triggered at each boundary. A synth patch that ignores velocity may sound identical across layers; that is not proof of a mapping failure or of correct layer selection. Inspect the preset/player zone display if listening cannot distinguish them.

Listen to the processed exports' attacks and tails. With automatic loops off, notes should not acquire an unexpected repeating sustain. This first pass does not validate experimental looping or every SFZ player's behavior.

**Pass:** both moved presets find their WAVs; no silent intermediate keys, wrong roots, duplicate boundary triggers, or processing defects. Keys outside 60–66 are outside this test's exported range.

## Record the outcome

Copy this template and fill in **Pass / Fail / Not tested**. Do not turn an untested gate into a pass.

```text
Date / tester:
Candidate version, source revision or binary SHA-256:
Mac / macOS:
Synth / patch / MIDI channel:
Audio interface / driver / actual sample rate / channel routing:
DecentSampler version / SFZ player and version:

Signal and channel identity:          Result:        Evidence:
Six-note/velocity takes and tails:    Result:        Evidence:
Selected-take replacement:           Result:        Evidence:
Save, restart, and moved session:    Result:        Evidence:
Stop during hold / during release:   Result:        Evidence:
Moved DecentSampler export/zones:    Result:        Evidence:
Moved SFZ export/zones:              Result:        Evidence:

Session/export paths or screenshots:
Observed failures and reproduction steps:
Remaining untested cases:
```

A passing session covers this synth/interface, rate, routing, patch, and these players. Fresh-install microphone permissions, other sample rates/channel layouts, longer batches, experimental loops, and Intel/other macOS compatibility remain separate release gates.
