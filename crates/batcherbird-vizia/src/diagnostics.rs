//! Explicit, local-only support reports. Never serialize the application model.
use crate::app_data::AppData;
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Summarize an error without copying untrusted free text (paths, device labels,
/// account identifiers and credentials can occur inside backend error messages).
fn error_category(message: &str) -> &'static str {
    let message = message.to_ascii_lowercase();
    if message.contains("diagnostic") {
        "diagnostics_export"
    } else if message.contains("session")
        || message.contains("manifest")
        || message.contains("sidecar")
    {
        "session"
    } else if message.contains("midi") {
        "midi"
    } else if message.contains("preview") || message.contains("playback") {
        "playback"
    } else if message.contains("audio") || message.contains("input") || message.contains("stream") {
        "audio"
    } else if message.contains("export") {
        "instrument_export"
    } else {
        "other"
    }
}

pub fn report(data: &AppData) -> Value {
    let config = data.build_sampling_config();
    let mut rates: Vec<_> = data
        .recorded_samples
        .iter()
        .map(|sample| sample.sample_rate)
        .collect();
    rates.sort_unstable();
    rates.dedup();
    let mut channels: Vec<_> = data
        .recorded_samples
        .iter()
        .map(|sample| sample.channels)
        .collect();
    channels.sort_unstable();
    channels.dedup();
    let selected = data
        .recorded_samples
        .get(data.selected_sample)
        .map(|sample| {
            json!({
                "index": data.selected_sample, "note": sample.note, "velocity": sample.velocity,
                "sample_rate_hz": sample.sample_rate, "channels": sample.channels,
                "frames": sample.audio_data.len().checked_div(sample.channels as usize),
            })
        });
    let engine_report = data
        .sampling_engine
        .as_ref()
        .map(|engine| engine.get_performance_diagnostics());
    json!({
        "schema_version": 1,
        "application": { "name": "Batcherbird", "version": env!("CARGO_PKG_VERSION"),
            "build_revision": null, "build_revision_note": "This build does not embed a verified Git revision." },
        "platform": { "os": std::env::consts::OS, "architecture": std::env::consts::ARCH,
            "os_version": null, "os_version_note": "Not exposed by the application." },
        "devices": {
            "midi_output": { "selected_index": data.midi_devices.get(data.selected_midi_device).map(|_| data.selected_midi_device),
                "available_count": data.midi_devices.len(), "connected": data.midi_connected, "name": data.midi_devices.get(data.selected_midi_device) },
            "audio_input": { "selected_index": data.audio_input_devices.get(data.selected_audio_input).map(|_| data.selected_audio_input),
                "available_count": data.audio_input_devices.len(), "connected": data.audio_connected, "name": data.audio_input_devices.get(data.selected_audio_input) },
            "name_note": "Selected device labels are included and may contain personal text. Review this locally saved report before sharing it. Indices refer to the application's current device lists."
        },
        "audio_configuration": { "channel_routing": format!("{:?}", data.channel_routing), "input_gain_db": data.input_gain_db,
            "live_sample_rate_hz": null, "live_input_channels": null, "buffer_frames": null,
            "live_configuration_note": "The active CPAL stream configuration is not exposed to the UI. Recorded audio metadata below is not a live configuration claim." },
        "capture_settings": { "start_note": data.start_note, "end_note": data.end_note,
            "note_step": data.note_step, "velocity_layers": data.velocity_layers,
            "note_duration_ms": config.note_duration_ms, "release_duration_ms": config.release_time_ms,
            "pre_delay_ms": config.pre_delay_ms, "post_delay_ms": config.post_delay_ms,
            "midi_channel": config.midi_channel + 1 },
        "export_settings": { "format": format!("{:?}", data.export_format), "trim_silence": data.trim_silence, "auto_loop": data.auto_loop },
        "session": { "state": format!("{:?}", data.app_state), "demo": data.demo_mode,
            "has_unsaved_changes": data.has_unsaved_changes, "recorded_samples": data.recorded_samples.len(),
            "sample_rates_hz": rates, "channel_counts": channels, "selected_sample": selected,
            "replacement_confirmation_pending": data.pending_session.is_some() },
        "streams": { "monitoring_handle_present": data.monitoring_stream.is_some(),
            "playthrough_handle_present": data.playthrough_stream.is_some(), "playthrough_enabled": data.playthrough_enabled,
            "preview_handle_present": data.preview_player.is_some(), "preview_playing": data.is_playing,
            "recording_worker_present": data.recording_worker.is_some(), "test_worker_present": data.test_worker.is_some(),
            "sampling_engine_present": data.sampling_engine.is_some() },
        "operations": { "session_busy": data.session_busy, "instrument_export_busy": data.export_in_progress,
            "diagnostics_export_busy": data.diagnostics_busy, "testing_note": data.is_testing_note },
        "recent_error": { "present": data.error_message.is_some(), "category": data.error_message.as_deref().map(error_category),
            "message": null, "message_note": "Raw errors are omitted because they may contain private paths or identifiers." },
        "performance": { "available": false, "exposed_engine_callback_count": engine_report.as_ref().map(|report| report.callback_count),
            "note": "The current callback paths do not populate the engine performance counters. Zero observations do not establish healthy performance." },
        "privacy": { "audio_included": false, "waveforms_included": false, "session_and_output_paths_collected": false,
            "device_names_included": true, "device_labels_may_contain_personal_text": true, "automatic_upload": false }
    })
}

/// Replace a report only after the full JSON is durable; failed writes retain the
/// previous destination file and remove only this invocation's temporary file.
pub fn save_report(path: &Path, report: &Value) -> Result<(), String> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let temporary = parent.join(format!(
        ".batcherbird-diagnostics-{}-{stamp}-{}.tmp",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let mut created = false;
    let result = (|| -> Result<(), String> {
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&temporary)
            .map_err(|error| error.to_string())?;
        created = true;
        serde_json::to_writer_pretty(&mut file, report).map_err(|error| error.to_string())?;
        file.write_all(b"\n").map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        fs::rename(&temporary, path).map_err(|error| error.to_string())
    })();
    if result.is_err() && created {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(|error| format!("Could not save diagnostics: {error}. Choose a writable folder and retry; your session is unchanged."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn directory() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "batcherbird-diagnostics-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn report_contains_selected_configuration_but_omits_session_paths_audio_and_raw_errors() {
        let mut data = AppData::demo();
        data.midi_devices = vec!["Wrong MIDI device".into(), "Selected synth MIDI".into()];
        data.selected_midi_device = 1;
        data.audio_input_devices = vec!["Selected USB interface".into()];
        data.selected_audio_input = 0;
        data.instrument_name = "private-instrument-name".into();
        data.output_directory = PathBuf::from("/Users/private-account/private-output");
        data.preferences_path = Some(PathBuf::from("/Users/private-account/preferences"));
        data.session_status = "private-session.batcherbird".into();
        data.preferred_audio_device = Some("private-preferred-device".into());
        data.error_message =
            Some("Audio failed /Users/private-account/private-secret token=secret-token".into());
        data.input_gain_db = -3.0;
        let report = report(&data);
        assert_eq!(
            report["devices"]["midi_output"]["name"],
            "Selected synth MIDI"
        );
        assert_eq!(
            report["devices"]["audio_input"]["name"],
            "Selected USB interface"
        );
        assert_eq!(report["audio_configuration"]["input_gain_db"], -3.0);
        assert_eq!(report["capture_settings"]["midi_channel"], 1);
        assert_eq!(report["session"]["recorded_samples"], 18);
        assert_eq!(report["session"]["sample_rates_hz"], json!([24000]));
        assert_eq!(report["session"]["channel_counts"], json!([2]));
        assert_eq!(report["recent_error"]["category"], "audio");
        assert_eq!(report["performance"]["available"], false);
        assert!(report["audio_configuration"]["live_sample_rate_hz"].is_null());
        assert!(report["application"]["build_revision"].is_null());
        let serialized = report.to_string();
        for private in [
            "Wrong MIDI device",
            "private-instrument-name",
            "private-account",
            "private-output",
            "private-session",
            "private-preferred-device",
            "private-secret",
            "secret-token",
            "audio_data",
            "viz_peaks",
        ] {
            assert!(
                !serialized.contains(private),
                "Unexpected private field: {private}"
            );
        }
    }

    #[test]
    fn empty_or_stale_device_selection_is_unavailable_instead_of_defaulted() {
        let data = AppData {
            selected_midi_device: 10,
            selected_audio_input: 8,
            ..AppData::default()
        };
        let report = report(&data);
        assert!(report["devices"]["midi_output"]["name"].is_null());
        assert!(report["devices"]["audio_input"]["selected_index"].is_null());
        assert_eq!(report["session"]["recorded_samples"], 0);
        assert!(report["session"]["selected_sample"].is_null());
        assert!(report["recent_error"]["category"].is_null());
    }

    #[test]
    fn local_report_round_trips_and_failed_commit_preserves_destination_contents() {
        let dir = directory();
        let report = report(&AppData::demo());
        let path = dir.join("support.json");
        save_report(&path, &report).unwrap();
        let loaded: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(loaded, report);
        let blocked = dir.join("existing-folder");
        fs::create_dir(&blocked).unwrap();
        fs::write(blocked.join("keep.txt"), "keep").unwrap();
        assert!(save_report(&blocked, &report)
            .unwrap_err()
            .contains("session is unchanged"));
        assert_eq!(
            fs::read_to_string(blocked.join("keep.txt")).unwrap(),
            "keep"
        );
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 2);
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn denied_write_preserves_previous_report_and_leaves_no_temporary_files() {
        use std::os::unix::fs::PermissionsExt;
        let dir = directory();
        let path = dir.join("support.json");
        let report = report(&AppData::default());
        save_report(&path, &report).unwrap();
        let original = fs::read(&path).unwrap();
        let permissions = fs::metadata(&dir).unwrap().permissions();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o500)).unwrap();
        let result = save_report(&path, &json!({"replacement": true}));
        fs::set_permissions(&dir, permissions).unwrap();
        assert!(result.is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_dir_all(dir).unwrap();
    }
}
