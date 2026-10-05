//! Hardware-free workflow checks using real session sidecars, WAVs and presets.
use batcherbird_core::export::{read_wav_metadata, AudioFormat, ExportConfig, SampleExporter};
use batcherbird_core::sampler::Sample;
use batcherbird_vizia::app_data::{AppData, AppState};
use batcherbird_vizia::session;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "support/workflow_fixture.rs"]
mod fixtures;
use fixtures::{instrument, take};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "batcherbird-workflow-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assert_same_take(actual: &Sample, expected: &Sample) {
    assert_eq!(
        (
            actual.note,
            actual.velocity,
            actual.sample_rate,
            actual.channels
        ),
        (
            expected.note,
            expected.velocity,
            expected.sample_rate,
            expected.channels
        )
    );
    assert_eq!(actual.audio_data, expected.audio_data);
    assert_eq!(actual.midi_timing, expected.midi_timing);
    assert_eq!(actual.audio_timing, expected.audio_timing);
}

#[test]
fn partial_batch_and_rerecords_survive_portable_session_and_float_wav_export() {
    let scratch = Scratch::new();
    let complete = instrument();
    let cancel = Arc::new(AtomicBool::new(false));
    let mut data = AppData {
        app_state: AppState::Recording,
        start_note: 60,
        end_note: 66,
        note_step: 3,
        velocity_layers: 2,
        instrument_name: "Quiet attack & long release".into(),
        recording_generation: 8,
        cancel_flag: Some(Arc::clone(&cancel)),
        ..AppData::default()
    };
    assert_eq!(data.total_samples(), 6);
    data.request_stop();
    assert!(cancel.load(Ordering::Acquire));
    assert_eq!(data.app_state, AppState::Stopping);
    assert!(data.is_current_recording(8));
    // The worker completed three takes before cancellation or a later capture error.
    data.accept_captured_samples(complete[..3].to_vec());
    assert_eq!(data.recorded_count, 3);
    assert_eq!(data.app_state, AppState::Review);
    data.select_sample(1);
    assert_eq!(data.selected_sample, 1);
    assert_eq!(
        data.sample_total_len, 72_000,
        "waveform length counts frames, not values"
    );
    assert!(data.selected_sample_label.contains("velocity 127"));
    let original_waveform = data.viz_peaks.clone();
    let original = data.recorded_samples[1].clone();
    data.replacement_sample = Some(1);
    data.app_state = AppState::Stopping;
    data.accept_captured_samples(Vec::new());
    assert_same_take(&data.recorded_samples[1], &original);
    assert_eq!(
        data.viz_peaks, original_waveform,
        "canceled rerecord keeps review waveform"
    );
    let replacement = take(original.note, original.velocity, 0.45);
    data.replacement_sample = Some(1);
    data.accept_captured_samples(vec![replacement.clone()]);
    assert_same_take(&data.recorded_samples[1], &replacement);
    assert_same_take(&data.recorded_samples[0], &complete[0]);
    assert!(
        data.viz_peaks.iter().copied().fold(0.0, f32::max)
            < original_waveform.iter().copied().fold(0.0, f32::max)
    );

    let original_folder = scratch.0.join("original");
    let manifest = original_folder.join("patch.batcherbird");
    session::save_session(&manifest, &data.session_settings(), &data.recorded_samples).unwrap();
    let portable_folder = scratch.0.join("moved");
    fs::rename(&original_folder, &portable_folder).unwrap();
    let (settings, restored) =
        session::load_session(&portable_folder.join("patch.batcherbird")).unwrap();
    for (actual, expected) in restored.iter().zip(&data.recorded_samples) {
        assert_same_take(actual, expected);
    }
    let mut reopened = AppData::default();
    reopened.apply_session_settings(settings);
    reopened.set_recorded_samples(restored);
    reopened.select_sample(1);
    assert_eq!(reopened.viz_peaks, data.viz_peaks);
    assert_eq!(
        reopened.session_settings().instrument_name,
        data.instrument_name
    );
    let exporter = SampleExporter::new(ExportConfig {
        output_directory: scratch.0.join("float"),
        sample_format: AudioFormat::Wav32BitFloat,
        apply_detection: false,
        fade_in_ms: 0.0,
        fade_out_ms: 0.0,
        ..ExportConfig::default()
    })
    .unwrap();
    let files = exporter.export_samples(&reopened.recorded_samples).unwrap();
    for (file, expected) in files.iter().zip(&reopened.recorded_samples) {
        let mut wav = hound::WavReader::open(file).unwrap();
        assert_eq!((wav.spec().channels, wav.spec().sample_rate), (2, 48_000));
        let actual = wav.samples::<f32>().collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(
            actual, expected.audio_data,
            "session-to-float-WAV preserves every value"
        );
        assert_eq!(
            read_wav_metadata(file).unwrap().midi_unity_note,
            Some(expected.note)
        );
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Zone {
    path: String,
    low: u8,
    high: u8,
    root: u8,
    lovel: u8,
    hivel: u8,
}
fn attribute(line: &str, name: &str) -> String {
    let prefix = format!("{name}=\"");
    line.split_once(&prefix)
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap()
        .to_string()
}
fn ds_zones(text: &str) -> Vec<Zone> {
    let mut velocity = (1, 127);
    let mut zones = Vec::new();
    for line in text.lines() {
        if line.contains("<group ") {
            velocity = (
                attribute(line, "loVel").parse().unwrap(),
                attribute(line, "hiVel").parse().unwrap(),
            );
        }
        if line.contains("<sample ") {
            zones.push(Zone {
                path: attribute(line, "path"),
                low: attribute(line, "loNote").parse().unwrap(),
                high: attribute(line, "hiNote").parse().unwrap(),
                root: attribute(line, "rootNote").parse().unwrap(),
                lovel: velocity.0,
                hivel: velocity.1,
            });
        }
    }
    zones
}
fn sfz_zones(text: &str) -> Vec<Zone> {
    let mut velocity = (1, 127);
    let mut regions = Vec::<HashMap<String, String>>::new();
    let mut region = false;
    for line in text.lines().map(str::trim) {
        if line == "<group>" {
            region = false;
        }
        if line == "<region>" {
            region = true;
            regions.push(HashMap::from([
                ("lovel".into(), velocity.0.to_string()),
                ("hivel".into(), velocity.1.to_string()),
            ]));
        }
        if let Some((key, value)) = line.split_once('=') {
            if region {
                regions.last_mut().unwrap().insert(key.into(), value.into());
            } else if key == "lovel" {
                velocity.0 = value.parse::<u8>().unwrap();
            } else if key == "hivel" {
                velocity.1 = value.parse::<u8>().unwrap();
            }
        }
    }
    regions
        .into_iter()
        .map(|r| Zone {
            path: r["sample"].clone(),
            low: r["lokey"].parse().unwrap(),
            high: r["hikey"].parse().unwrap(),
            root: r["pitch_keycenter"].parse().unwrap(),
            lovel: r["lovel"].parse().unwrap(),
            hivel: r["hivel"].parse().unwrap(),
        })
        .collect()
}

#[test]
fn complete_saved_instrument_exports_playable_zones_and_preserves_quiet_attack_long_release_stereo()
{
    let scratch = Scratch::new();
    let mut data = AppData {
        instrument_name: "Weekend pad".into(),
        start_note: 60,
        end_note: 66,
        note_step: 3,
        velocity_layers: 2,
        ..AppData::default()
    };
    data.accept_captured_samples(instrument());
    let manifest = scratch.0.join("pad.batcherbird");
    session::save_session(&manifest, &data.session_settings(), &data.recorded_samples).unwrap();
    let (_, samples) = session::load_session(&manifest).unwrap();
    let output = scratch.0.join("instrument");
    let config = ExportConfig {
        output_directory: output.clone(),
        sample_format: AudioFormat::All,
        naming_pattern: "Weekend pad_{note_name}_{note}_{velocity}.wav".into(),
        fade_out_ms: 0.0,
        detection_config: batcherbird_core::detection::DetectionConfig {
            threshold_db: -60.0,
            window_size_ms: 5.0,
            confirmation_windows: 1,
            pre_trigger_ms: 10.0,
            post_trigger_ms: 10.0,
            ..Default::default()
        },
        ..ExportConfig::default()
    };
    let files = SampleExporter::new(config)
        .unwrap()
        .export_samples(&samples)
        .unwrap();
    assert_eq!(files.len(), 8);
    let preset = files
        .iter()
        .find(|p| p.extension().is_some_and(|e| e == "dspreset"))
        .unwrap();
    let sfz = files
        .iter()
        .find(|p| p.extension().is_some_and(|e| e == "sfz"))
        .unwrap();
    let ds = ds_zones(&fs::read_to_string(preset).unwrap());
    let sfz = sfz_zones(&fs::read_to_string(sfz).unwrap());
    assert_eq!(ds, sfz, "both sampler formats describe the same instrument");
    assert_eq!(ds.len(), 6);
    for note in 60..=66 {
        for velocity in 1..=127 {
            assert_eq!(
                ds.iter()
                    .filter(|z| z.low <= note
                        && note <= z.high
                        && z.lovel <= velocity
                        && velocity <= z.hivel)
                    .count(),
                1,
                "note {note}, velocity {velocity} must play exactly one region"
            );
        }
    }
    for zone in &ds {
        let path = output.join(&zone.path);
        assert!(Path::new(&zone.path).is_relative());
        assert!(
            path.exists(),
            "preset sample reference resolves: {}",
            path.display()
        );
        assert!(zone.path.contains("_vel"));
        assert!(!zone.path.contains("velvel"));
        let mut wav = hound::WavReader::open(&path).unwrap();
        assert_eq!(
            (
                wav.spec().channels,
                wav.spec().sample_rate,
                wav.spec().bits_per_sample
            ),
            (2, 48_000, 24)
        );
        let audio = wav
            .samples::<i32>()
            .map(|s| s.unwrap() as f32 / 8_388_608.0)
            .collect::<Vec<_>>();
        let frames = audio.len() / 2;
        assert!(
            (46_000..53_000).contains(&frames),
            "trim removes silence while retaining the long signal: {frames} frames"
        );
        let quiet_attack = audio
            .chunks_exact(2)
            .take(4000)
            .map(|f| f[0].abs())
            .fold(0.0, f32::max);
        assert!(
            (0.001..0.005).contains(&quiet_attack),
            "quiet attack survives trimming: {quiet_attack}"
        );
        let late_release = audio
            .chunks_exact(2)
            .skip(40_000)
            .map(|f| f[0].abs())
            .fold(0.0, f32::max);
        assert!(late_release > 0.003, "late release remains audible");
        assert!(
            audio
                .chunks_exact(2)
                .all(|f| (f[1] + 0.35 * f[0]).abs() < 0.000001),
            "channels retain their distinct polarity and level"
        );
        let metadata = read_wav_metadata(&path).unwrap();
        assert_eq!(metadata.midi_unity_note, Some(zone.root));
        assert_eq!(metadata.sample_period_ns, Some(1_000_000_000 / 48_000));
        assert_eq!(metadata.loop_points, None, "automatic looping stays opt-in");
    }
}

#[test]
fn saved_untrimmed_export_choice_preserves_the_full_quiet_take() {
    let scratch = Scratch::new();
    let source = take(60, 64, 1.0);
    let mut data = AppData {
        trim_silence: false,
        instrument_name: "Untrimmed pad".into(),
        output_directory: scratch.0.join("exports"),
        export_format: AudioFormat::Wav32BitFloat,
        selected_format_index: 2,
        ..AppData::default()
    };
    data.accept_captured_samples(vec![source.clone()]);
    let manifest = scratch.0.join("untrimmed.batcherbird");
    session::save_session(&manifest, &data.session_settings(), &data.recorded_samples).unwrap();
    let (settings, samples) = session::load_session(&manifest).unwrap();
    let mut reopened = AppData::default();
    reopened.apply_session_settings(settings);
    reopened.set_recorded_samples(samples);
    assert!(!reopened.trim_silence);
    let config = reopened.build_export_config();
    assert!(
        !config.apply_detection,
        "desktop export uses the persisted trim choice"
    );
    let paths = SampleExporter::new(config)
        .unwrap()
        .export_samples(&reopened.recorded_samples)
        .unwrap();
    let mut wav = hound::WavReader::open(&paths[0]).unwrap();
    let exported = wav.samples::<f32>().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(
        exported.len(),
        source.audio_data.len(),
        "preserve leading silence and full tail"
    );
    // The default ending fade touches only trailing silence in this fixture.
    assert_eq!(
        exported, source.audio_data,
        "quiet attack and complete release stay intact"
    );
    assert_eq!(
        read_wav_metadata(&paths[0]).unwrap().midi_unity_note,
        Some(60)
    );
    let legacy: session::SessionSettings =
        serde_json::from_str(r#"{"instrument_name":"Old session"}"#).unwrap();
    assert!(
        legacy.trim_silence,
        "older sessions retain existing automatic-trim behavior"
    );
    assert!(AppData::default().build_export_config().apply_detection);
}
