//! Portable sessions: a small JSON manifest and lossless, unprocessed float WAVs.
//! Each save writes a new audio directory before atomically replacing the manifest.
use batcherbird_core::sampler::Sample;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionSettings {
    pub instrument_name: String,
    pub start_note: u8,
    pub end_note: u8,
    pub velocity_layers: u8,
    pub note_step: u8,
    pub note_duration_ms: u32,
    pub release_duration_ms: u32,
    pub channel_routing: usize,
    pub input_gain_db: f32,
    pub export_format: usize,
    pub output_directory: PathBuf,
    pub midi_device: Option<String>,
    pub audio_device: Option<String>,
    pub auto_loop: bool,
    /// Preserve quiet attacks and complete tails when disabled.
    pub trim_silence: bool,
}

impl Default for SessionSettings {
    fn default() -> Self {
        Self {
            instrument_name: "Untitled instrument".into(),
            start_note: 36,
            end_note: 84,
            velocity_layers: 1,
            note_step: 1,
            note_duration_ms: 2000,
            release_duration_ms: 1000,
            channel_routing: 0,
            input_gain_db: 0.0,
            export_format: 1,
            output_directory: PathBuf::from("."),
            midi_device: None,
            audio_device: None,
            auto_loop: false,
            trim_silence: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredSample {
    file: PathBuf,
    note: u8,
    velocity: u8,
    recorded_at_ms: u64,
    midi_timing_ms: u64,
    audio_timing_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    version: u32,
    settings: SessionSettings,
    samples: Vec<StoredSample>,
}

fn unique_stamp() -> Result<String, String> {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(err)?
        .as_nanos();
    let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Ok(format!("{time}{}{:020}", std::process::id(), count))
}

fn err(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn atomic_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(err)?;
    let stamp = unique_stamp()?;
    let temp = parent.join(format!(".batcherbird-{stamp}.tmp"));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(err)?;
        serde_json::to_writer_pretty(&mut file, value).map_err(err)?;
        file.write_all(b"\n").map_err(err)?;
        file.sync_all().map_err(err)?;
        fs::rename(&temp, path).map_err(err)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

pub fn save_settings(path: &Path, settings: &SessionSettings) -> Result<(), String> {
    atomic_json(path, settings)
}

pub fn load_settings(path: &Path) -> Result<SessionSettings, String> {
    let reader = fs::File::open(path).map_err(err)?.take(1_048_576);
    serde_json::from_reader(reader).map_err(err)
}

fn safe_stem(path: &Path) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Identify only sidecars generated for this exact manifest, with no extra files or links.
/// A malformed/foreign manifest is never grounds for removing anything.
struct OwnedSidecar {
    directory: PathBuf,
    filenames: std::collections::HashSet<String>,
}

fn superseded_sidecar(path: &Path) -> Option<OwnedSidecar> {
    if fs::symlink_metadata(path).ok()?.file_type().is_symlink() {
        return None;
    }
    let file = fs::File::open(path).ok()?;
    if file.metadata().ok()?.len() > 16 * 1024 * 1024 {
        return None;
    }
    let manifest: Manifest = serde_json::from_reader(file).ok()?;
    if manifest.version != 1 || manifest.samples.is_empty() {
        return None;
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let prefix = format!("{}.audio-", safe_stem(path));
    let mut directory = None;
    let mut filenames = std::collections::HashSet::new();
    for sample in manifest.samples {
        let mut components = sample.file.components();
        let (Some(Component::Normal(dir)), Some(Component::Normal(file)), None) =
            (components.next(), components.next(), components.next())
        else {
            return None;
        };
        let dir = dir.to_str()?;
        let stamp = dir.strip_prefix(&prefix)?;
        if stamp.is_empty() || !stamp.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        if directory.as_deref().is_some_and(|previous| previous != dir) {
            return None;
        }
        directory = Some(dir.to_string());
        let filename = file.to_str()?;
        let number = filename.strip_prefix("sample_")?.strip_suffix(".wav")?;
        if number.len() < 4 || !number.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        if !filenames.insert(filename.to_string()) {
            return None;
        }
    }
    let directory = parent.join(directory?);
    if !fs::symlink_metadata(&directory).ok()?.file_type().is_dir() {
        return None;
    }
    let mut found = std::collections::HashSet::new();
    for entry in fs::read_dir(&directory).ok()? {
        let entry = entry.ok()?;
        if !entry.file_type().ok()?.is_file() {
            return None;
        }
        let filename = entry.file_name().into_string().ok()?;
        if !filenames.contains(&filename) {
            return None;
        }
        found.insert(filename);
    }
    if found != filenames {
        return None;
    }
    Some(OwnedSidecar {
        directory,
        filenames,
    })
}

fn clean_superseded_sidecar(sidecar: &OwnedSidecar) {
    let directory = &sidecar.directory;
    // Recheck immediately before removing. Never recurse into subdirectories or links.
    let Ok(metadata) = fs::symlink_metadata(directory) else {
        return;
    };
    if !metadata.file_type().is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    let Ok(entries) = entries.collect::<Result<Vec<_>, _>>() else {
        return;
    };
    let mut actual = std::collections::HashSet::new();
    for entry in &entries {
        if !entry.file_type().is_ok_and(|kind| kind.is_file()) {
            return;
        }
        let Ok(filename) = entry.file_name().into_string() else {
            return;
        };
        actual.insert(filename);
    }
    if actual != sidecar.filenames {
        return;
    }
    for entry in entries {
        if fs::remove_file(entry.path()).is_err() {
            return;
        }
    }
    let _ = fs::remove_dir(directory);
}

pub fn save_session(
    path: &Path,
    settings: &SessionSettings,
    samples: &[Sample],
) -> Result<(), String> {
    // Validate the entire batch before creating any new sidecar files.
    for sample in samples {
        if sample.note > 127
            || sample.velocity == 0
            || sample.velocity > 127
            || sample.channels == 0
            || sample.channels > 2
            || sample.sample_rate == 0
            || sample.audio_data.is_empty()
            || !sample
                .audio_data
                .len()
                .is_multiple_of(sample.channels as usize)
            || sample.audio_data.iter().any(|value| !value.is_finite())
        {
            return Err(
                "Cannot save a sample with invalid note, velocity, or audio frames.".into(),
            );
        }
    }
    let previous_sidecar = superseded_sidecar(path);
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(err)?;
    let stamp = unique_stamp()?;
    let audio_dir_name = format!("{}.audio-{stamp}", safe_stem(path));
    let audio_dir = parent.join(&audio_dir_name);
    if !samples.is_empty() {
        fs::create_dir(&audio_dir).map_err(err)?;
    }
    let result = (|| {
        let mut stored = Vec::with_capacity(samples.len());
        for (index, sample) in samples.iter().enumerate() {
            let filename = format!("sample_{index:04}.wav");
            let mut writer = hound::WavWriter::create(
                audio_dir.join(&filename),
                hound::WavSpec {
                    channels: sample.channels,
                    sample_rate: sample.sample_rate,
                    bits_per_sample: 32,
                    sample_format: hound::SampleFormat::Float,
                },
            )
            .map_err(err)?;
            for &value in &sample.audio_data {
                writer.write_sample(value).map_err(err)?;
            }
            writer.finalize().map_err(err)?;
            fs::File::open(audio_dir.join(&filename))
                .map_err(err)?
                .sync_all()
                .map_err(err)?;
            stored.push(StoredSample {
                file: Path::new(&audio_dir_name).join(filename),
                note: sample.note,
                velocity: sample.velocity,
                recorded_at_ms: sample
                    .recorded_at
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                midi_timing_ms: sample.midi_timing.as_millis() as u64,
                audio_timing_ms: sample.audio_timing.as_millis() as u64,
            });
        }
        atomic_json(
            path,
            &Manifest {
                version: 1,
                settings: settings.clone(),
                samples: stored,
            },
        )
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&audio_dir);
    } else if let Some(previous) = previous_sidecar {
        // Replacement has committed; failure to clean old audio must not invalidate the save.
        clean_superseded_sidecar(&previous);
    }
    result
}

pub fn load_session(path: &Path) -> Result<(SessionSettings, Vec<Sample>), String> {
    let file = fs::File::open(path).map_err(err)?;
    if file.metadata().map_err(err)?.len() > 16 * 1024 * 1024 {
        return Err("Session manifest is too large.".into());
    }
    let manifest: Manifest = serde_json::from_reader(file).map_err(err)?;
    if manifest.version != 1 {
        return Err(format!("Unsupported session version {}.", manifest.version));
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let root = parent.canonicalize().map_err(err)?;
    let mut samples = Vec::with_capacity(manifest.samples.len());
    for stored in manifest.samples {
        if stored.note > 127
            || stored.velocity == 0
            || stored.velocity > 127
            || stored
                .file
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err("Session contains an invalid note, velocity, or audio path.".into());
        }
        let audio_path = root.join(&stored.file).canonicalize().map_err(err)?;
        if !audio_path.starts_with(&root) {
            return Err("Session audio must stay inside its session folder.".into());
        }
        let file_bytes = fs::metadata(&audio_path).map_err(err)?.len();
        let mut reader = hound::WavReader::open(audio_path).map_err(err)?;
        let spec = reader.spec();
        if spec.channels == 0
            || spec.channels > 2
            || spec.sample_rate == 0
            || spec.sample_format != hound::SampleFormat::Float
            || spec.bits_per_sample != 32
        {
            return Err("Session audio must be mono or stereo 32-bit float WAV.".into());
        }
        // Bound reservation by the actual file, not a potentially forged RIFF data length.
        let values = reader.len() as usize;
        if values == 0 || values as u64 > file_bytes / 4 {
            return Err("Session audio has an invalid or truncated data length.".into());
        }
        let mut audio_data = Vec::new();
        audio_data.try_reserve_exact(values).map_err(err)?;
        for value in reader.samples::<f32>() {
            audio_data.push(value.map_err(err)?);
        }
        if audio_data.len() % spec.channels as usize != 0
            || audio_data.iter().any(|v| !v.is_finite())
        {
            return Err("Session audio contains invalid frames or values.".into());
        }
        samples.push(Sample {
            note: stored.note,
            velocity: stored.velocity,
            audio_data,
            sample_rate: spec.sample_rate,
            channels: spec.channels,
            recorded_at: UNIX_EPOCH
                .checked_add(Duration::from_millis(stored.recorded_at_ms))
                .ok_or("Invalid recording timestamp")?,
            midi_timing: Duration::from_millis(stored.midi_timing_ms),
            audio_timing: Duration::from_millis(stored.audio_timing_ms),
        });
    }
    Ok((manifest.settings, samples))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_dir() -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "batcherbird-session-{}-{}",
            std::process::id(),
            unique_stamp().unwrap()
        ));
        fs::create_dir(&p).unwrap();
        p
    }
    #[test]
    fn portable_session_restores_lossless_audio_and_configuration() {
        let dir = test_dir();
        let path = dir.join("patch.batcherbird");
        let settings = SessionSettings {
            instrument_name: "DW6000 / warm pad".into(),
            note_step: 3,
            release_duration_ms: 3000,
            ..Default::default()
        };
        let sample = Sample {
            note: 60,
            velocity: 100,
            audio_data: vec![0.125, -0.25, 0.5, -0.75],
            sample_rate: 48000,
            channels: 2,
            recorded_at: UNIX_EPOCH + Duration::from_millis(1000),
            midi_timing: Duration::from_millis(2000),
            audio_timing: Duration::from_millis(3100),
        };
        save_session(&path, &settings, std::slice::from_ref(&sample)).unwrap();
        let moved = dir.with_extension("moved");
        fs::rename(&dir, &moved).unwrap();
        let (restored, samples) = load_session(&moved.join("patch.batcherbird")).unwrap();
        assert_eq!(restored.instrument_name, settings.instrument_name);
        assert_eq!(restored.release_duration_ms, 3000);
        assert_eq!(samples[0].audio_data, sample.audio_data);
        assert_eq!(samples[0].sample_rate, 48000);
        assert_eq!(samples[0].channels, 2);
        assert_eq!(samples[0].midi_timing, sample.midi_timing);
        fs::remove_dir_all(moved).unwrap();
    }
    #[test]
    fn failed_save_does_not_replace_existing_manifest() {
        let dir = test_dir();
        let path = dir.join("patch.batcherbird");
        save_session(&path, &SessionSettings::default(), &[]).unwrap();
        let original = fs::read(&path).unwrap();
        let bad = Sample {
            note: 60,
            velocity: 100,
            audio_data: vec![0.0],
            sample_rate: 48000,
            channels: 2,
            recorded_at: UNIX_EPOCH,
            midi_timing: Duration::ZERO,
            audio_timing: Duration::ZERO,
        };
        assert!(save_session(&path, &SessionSettings::default(), &[bad]).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn session_rejects_paths_outside_its_folder() {
        let dir = test_dir();
        let path = dir.join("patch.batcherbird");
        let manifest = Manifest {
            version: 1,
            settings: SessionSettings::default(),
            samples: vec![StoredSample {
                file: PathBuf::from("../outside.wav"),
                note: 60,
                velocity: 100,
                recorded_at_ms: 0,
                midi_timing_ms: 0,
                audio_timing_ms: 0,
            }],
        };
        atomic_json(&path, &manifest).unwrap();
        assert!(load_session(&path).unwrap_err().contains("invalid"));
        fs::remove_dir_all(dir).unwrap();
    }
    fn sample() -> Sample {
        Sample {
            note: 60,
            velocity: 100,
            audio_data: vec![0.25, -0.5],
            channels: 2,
            sample_rate: 48000,
            recorded_at: UNIX_EPOCH,
            midi_timing: Duration::ZERO,
            audio_timing: Duration::ZERO,
        }
    }

    fn referenced_directory(path: &Path) -> PathBuf {
        let manifest: Manifest = serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();
        path.parent()
            .unwrap()
            .join(manifest.samples[0].file.parent().unwrap())
    }

    #[test]
    fn successful_overwrite_cleans_only_its_superseded_sidecar() {
        let dir = test_dir();
        let path = dir.join("warm pad.batcherbird");
        save_session(&path, &SessionSettings::default(), &[sample()]).unwrap();
        let old = referenced_directory(&path);
        save_session(&path, &SessionSettings::default(), &[sample()]).unwrap();
        assert!(!old.exists());
        assert!(referenced_directory(&path).is_dir());
        assert_eq!(
            load_session(&path).unwrap().1[0].audio_data,
            sample().audio_data
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn invalid_batch_preserves_existing_manifest_and_audio_without_new_files() {
        let dir = test_dir();
        let path = dir.join("patch.batcherbird");
        save_session(&path, &SessionSettings::default(), &[sample()]).unwrap();
        let manifest = fs::read(&path).unwrap();
        let old = referenced_directory(&path);
        let values = fs::read(old.join("sample_0000.wav")).unwrap();
        for invalid in [
            Sample {
                note: 128,
                ..sample()
            },
            Sample {
                velocity: 0,
                ..sample()
            },
            Sample {
                velocity: 128,
                ..sample()
            },
            Sample {
                channels: 3,
                ..sample()
            },
            Sample {
                audio_data: vec![0.0],
                ..sample()
            },
            Sample {
                audio_data: vec![f32::NAN, 0.0],
                ..sample()
            },
            Sample {
                audio_data: vec![f32::INFINITY, 0.0],
                ..sample()
            },
        ] {
            assert!(
                save_session(&path, &SessionSettings::default(), &[sample(), invalid]).is_err()
            );
            assert_eq!(fs::read(&path).unwrap(), manifest);
            assert_eq!(fs::read(old.join("sample_0000.wav")).unwrap(), values);
            assert_eq!(fs::read_dir(&dir).unwrap().count(), 2);
        }
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn sidecar_with_unowned_files_is_preserved() {
        let dir = test_dir();
        let path = dir.join("patch.batcherbird");
        save_session(&path, &SessionSettings::default(), &[sample()]).unwrap();
        let old = referenced_directory(&path);
        fs::write(old.join("personal-notes.txt"), "keep me").unwrap();
        save_session(&path, &SessionSettings::default(), &[sample()]).unwrap();
        assert!(old.join("sample_0000.wav").is_file());
        assert_eq!(
            fs::read_to_string(old.join("personal-notes.txt")).unwrap(),
            "keep me"
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn empty_sessions_do_not_create_unreferenced_audio_directories() {
        let dir = test_dir();
        let path = dir.join("patch.batcherbird");
        for _ in 0..3 {
            save_session(&path, &SessionSettings::default(), &[]).unwrap();
        }
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn external_audio_symlink_is_rejected_and_never_cleaned() {
        let dir = test_dir();
        let outside = test_dir();
        let path = dir.join("patch.batcherbird");
        save_session(&path, &SessionSettings::default(), &[sample()]).unwrap();
        let old = referenced_directory(&path);
        fs::copy(old.join("sample_0000.wav"), outside.join("sample_0000.wav")).unwrap();
        fs::remove_file(old.join("sample_0000.wav")).unwrap();
        fs::remove_dir(&old).unwrap();
        std::os::unix::fs::symlink(&outside, &old).unwrap();
        assert!(load_session(&path)
            .unwrap_err()
            .contains("inside its session folder"));
        save_session(&path, &SessionSettings::default(), &[sample()]).unwrap();
        assert!(outside.join("sample_0000.wav").is_file());
        assert!(fs::symlink_metadata(&old).unwrap().file_type().is_symlink());
        fs::remove_dir_all(dir).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[test]
    fn cleanup_rechecks_ownership_after_manifest_replacement() {
        let dir = test_dir();
        let path = dir.join("patch.batcherbird");
        save_session(&path, &SessionSettings::default(), &[sample()]).unwrap();
        let owned = superseded_sidecar(&path).unwrap();
        fs::write(owned.directory.join("added-later.txt"), "keep").unwrap();
        clean_superseded_sidecar(&owned);
        assert!(owned.directory.join("added-later.txt").exists());
        assert!(owned.directory.join("sample_0000.wav").exists());
        fs::remove_dir_all(dir).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn symlinked_audio_file_is_not_followed_for_cleanup_or_external_loading() {
        let dir = test_dir();
        let outside = test_dir();
        let path = dir.join("patch.batcherbird");
        save_session(&path, &SessionSettings::default(), &[sample()]).unwrap();
        let old = referenced_directory(&path);
        let wav = old.join("sample_0000.wav");
        let external = outside.join("original.wav");
        fs::rename(&wav, &external).unwrap();
        std::os::unix::fs::symlink(&external, &wav).unwrap();
        assert!(load_session(&path).is_err());
        save_session(&path, &SessionSettings::default(), &[sample()]).unwrap();
        assert!(external.is_file());
        assert!(fs::symlink_metadata(&wav).unwrap().file_type().is_symlink());
        fs::remove_dir_all(dir).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[test]
    fn failed_manifest_commit_removes_only_new_audio_directory() {
        let dir = test_dir();
        let path = dir.join("patch.batcherbird");
        fs::create_dir(&path).unwrap();
        fs::write(path.join("existing-data"), "keep").unwrap();
        assert!(save_session(&path, &SessionSettings::default(), &[sample()]).is_err());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        assert_eq!(
            fs::read_to_string(path.join("existing-data")).unwrap(),
            "keep"
        );
        fs::remove_dir_all(dir).unwrap();
    }
}
