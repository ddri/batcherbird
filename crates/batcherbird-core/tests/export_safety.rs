use batcherbird_core::export::{AudioFormat, ExportConfig, SampleExporter};
use batcherbird_core::sampler::Sample;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// A fresh, empty directory unique to one test.
fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "batcherbird_export_safety_{}_{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A sine take whose peak is `peak`, as produced by capture with input gain applied.
fn take(note: u8, peak: f32) -> Sample {
    Sample {
        note,
        velocity: 100,
        audio_data: (0..4410).map(|i| peak * (i as f32 * 0.05).sin()).collect(),
        sample_rate: 44100,
        channels: 1,
        recorded_at: SystemTime::now(),
        midi_timing: Duration::ZERO,
        audio_timing: Duration::ZERO,
    }
}

fn exporter(dir: &Path, format: AudioFormat, normalize: bool) -> SampleExporter {
    SampleExporter::new(ExportConfig {
        output_directory: dir.to_path_buf(),
        naming_pattern: "Hot_{note}_{velocity}.wav".into(),
        sample_format: format,
        normalize,
        fade_in_ms: 0.0,
        fade_out_ms: 0.0,
        apply_detection: false,
        auto_loop: false,
        embed_metadata: false,
        ..ExportConfig::default()
    })
    .unwrap()
}

fn peak_of(path: &Path) -> i32 {
    let mut reader = hound::WavReader::open(path).unwrap();
    reader
        .samples::<i32>()
        .map(|s| s.unwrap().abs())
        .max()
        .unwrap()
}

#[test]
fn normalize_turns_a_hot_take_down_to_headroom() {
    let dir = test_dir("normalize_hot");
    let files = exporter(&dir, AudioFormat::Wav24Bit, true)
        .export_samples(&[take(60, 2.0)])
        .unwrap();

    let expected = 0.95 * 8_388_607.0;
    let peak = peak_of(&files[0]) as f32;
    assert!((peak - expected).abs() / expected < 0.01, "peak {peak}");
}

#[test]
fn hot_take_without_normalize_is_clamped_to_full_scale_in_24_bit() {
    let dir = test_dir("clamp_24");
    let files = exporter(&dir, AudioFormat::Wav24Bit, false)
        .export_samples(&[take(60, 2.0)])
        .unwrap();

    assert_eq!(peak_of(&files[0]), 8_388_607);
}

#[test]
fn hot_take_exports_through_sampler_presets() {
    let dir = test_dir("clamp_sfz");
    let files = exporter(&dir, AudioFormat::DecentSamplerAndSfz, false)
        .export_samples(&[take(60, 2.0), take(62, 0.5)])
        .unwrap();

    assert_eq!(files.len(), 4);
}

fn pack_exporter(pack: &Path, format: AudioFormat, normalize: bool) -> SampleExporter {
    exporter(pack, format, normalize)
}

fn names_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn clamped_takes_are_reported_by_note_and_velocity() {
    let pack = test_dir("report_clipped").join("Hot");
    let report = pack_exporter(&pack, AudioFormat::Wav24Bit, false)
        .export_pack(&[take(60, 2.0), take(62, 0.5)], false)
        .unwrap();

    assert_eq!(report.clipped, vec![(60, 100)]);
}

#[test]
fn normalized_and_float_exports_report_no_clipping() {
    let root = test_dir("report_none");
    let normalized = pack_exporter(&root.join("Norm"), AudioFormat::Wav24Bit, true)
        .export_pack(&[take(60, 2.0)], false)
        .unwrap();
    let float = pack_exporter(&root.join("Float"), AudioFormat::Wav32BitFloat, false)
        .export_pack(&[take(60, 2.0)], false)
        .unwrap();

    assert!(normalized.clipped.is_empty());
    assert!(float.clipped.is_empty());
}

#[test]
fn existing_pack_is_not_overwritten_without_replace() {
    let pack = test_dir("no_replace").join("Hot");
    let exporter = pack_exporter(&pack, AudioFormat::SFZ, true);
    exporter.export_pack(&[take(60, 0.5)], false).unwrap();
    let before = std::fs::read(pack.join("Hot_60_vel100.wav")).unwrap();

    let result = exporter.export_pack(&[take(60, 0.9)], false);

    assert!(matches!(
        result,
        Err(batcherbird_core::BatcherbirdError::PackExists(_))
    ));
    assert_eq!(
        std::fs::read(pack.join("Hot_60_vel100.wav")).unwrap(),
        before
    );
}

#[test]
fn replacing_a_pack_removes_files_from_the_old_export() {
    let root = test_dir("replace_stale");
    let pack = root.join("Hot");
    let exporter = pack_exporter(&pack, AudioFormat::SFZ, true);
    exporter
        .export_pack(&[take(60, 0.5), take(62, 0.5)], false)
        .unwrap();

    exporter.export_pack(&[take(60, 0.5)], true).unwrap();

    assert_eq!(names_in(&pack), vec!["Hot.sfz", "Hot_60_vel100.wav"]);
    assert_eq!(
        names_in(&root),
        vec!["Hot"],
        "no staging folders left behind"
    );
}

#[cfg(unix)]
#[test]
fn failed_replacement_leaves_the_old_pack_untouched() {
    use std::os::unix::fs::PermissionsExt;
    let root = test_dir("replace_failure");
    let pack = root.join("Hot");
    let exporter = pack_exporter(&pack, AudioFormat::SFZ, true);
    exporter
        .export_pack(&[take(60, 0.5), take(62, 0.5)], false)
        .unwrap();
    let before: Vec<_> = names_in(&pack)
        .iter()
        .map(|name| std::fs::read(pack.join(name)).unwrap())
        .collect();
    // The pack folder stays writable, but nothing new can be created beside it.
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o555)).unwrap();

    let result = exporter.export_pack(&[take(60, 0.9)], true);

    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(result.is_err());
    let after: Vec<_> = names_in(&pack)
        .iter()
        .map(|name| std::fs::read(pack.join(name)).unwrap())
        .collect();
    assert_eq!(after, before);
}

#[test]
fn replace_refuses_a_folder_holding_files_batcherbird_did_not_write() {
    let pack = test_dir("replace_foreign").join("Hot");
    std::fs::create_dir_all(&pack).unwrap();
    std::fs::write(pack.join("notes.txt"), "keep me").unwrap();

    let result = pack_exporter(&pack, AudioFormat::SFZ, true).export_pack(&[take(60, 0.5)], true);

    assert!(result.is_err());
    assert_eq!(
        std::fs::read_to_string(pack.join("notes.txt")).unwrap(),
        "keep me"
    );
    assert!(!pack.join("Hot_60_vel100.wav").exists());
}
