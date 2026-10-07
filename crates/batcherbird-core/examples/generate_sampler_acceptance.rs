//! Generate pitch-correct exports and MIDI probes for independent sampler QA.
use batcherbird_core::export::{AudioFormat, ExportConfig, SampleExporter};
use batcherbird_core::sampler::Sample;
use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime},
};

fn variable_length(mut value: u32, bytes: &mut Vec<u8>) {
    let mut encoded = vec![(value & 127) as u8];
    while {
        value >>= 7;
        value != 0
    } {
        encoded.push(((value & 127) as u8) | 128);
    }
    bytes.extend(encoded.into_iter().rev());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(std::env::args_os().nth(1).ok_or(
        "Usage: cargo run -p batcherbird-core --example generate_sampler_acceptance -- NEW_DIRECTORY"
    )?);
    // Never overwrite a previous acceptance run or user instrument.
    fs::create_dir(&output)?;
    let instrument = output.join("instrument with spaces");
    let samples: Vec<_> = [64, 127]
        .into_iter()
        .flat_map(|velocity| {
            [60, 63, 66].into_iter().map(move |note| {
                let frequency = 440.0 * 2_f32.powf((note as f32 - 69.0) / 12.0);
                let audio_data = (0..96_000)
                    .flat_map(|frame| {
                        let phase = std::f32::consts::TAU * frequency * frame as f32 / 48_000.0;
                        // The upper layer has a distinct second harmonic. Right channel
                        // inversion/gain reveals channel swaps or accidental mono mixing.
                        let root_marker = match note {
                            60 => 0.1,
                            63 => 0.2,
                            _ => 0.3,
                        };
                        let value = 0.25
                            * (phase.sin()
                                + root_marker * (3.0 * phase).sin()
                                + if velocity == 127 {
                                    0.5 * (2.0 * phase).sin()
                                } else {
                                    0.0
                                });
                        [value, -0.35 * value]
                    })
                    .collect();
                Sample {
                    note,
                    velocity,
                    audio_data,
                    sample_rate: 48_000,
                    channels: 2,
                    recorded_at: SystemTime::UNIX_EPOCH,
                    midi_timing: Duration::ZERO,
                    audio_timing: Duration::from_secs(2),
                }
            })
        })
        .collect();
    let exporter = SampleExporter::new(ExportConfig {
        output_directory: instrument,
        naming_pattern: "Acceptance_{note}_{velocity}.wav".into(),
        sample_format: AudioFormat::DecentSamplerAndSfz,
        apply_detection: false,
        normalize: false,
        auto_loop: false,
        fade_in_ms: 5.0,
        fade_out_ms: 10.0,
        ..Default::default()
    })?;
    exporter.export_samples(&samples)?;
    // Format 0; 480 PPQN at 500000 us/quarter. Every note starts on a
    // two-second boundary, holds for 500ms, and leaves 1.5s release space.
    let mut track = vec![0, 0xff, 0x51, 3, 7, 0xa1, 0x20];
    let mut manifest = String::from("index,start_seconds,note,velocity,root,layer\n");
    let mut index = 0;
    for note in 59..=67 {
        for velocity in [1, 64, 95, 96, 127] {
            variable_length(if index == 0 { 0 } else { 1440 }, &mut track);
            track.extend([0x90, note, velocity]);
            variable_length(480, &mut track);
            track.extend([0x80, note, 0]);
            let root = match note {
                60..=61 => "60",
                62..=64 => "63",
                65..=66 => "66",
                _ => "silent",
            };
            let layer = if root == "silent" {
                "silent"
            } else if velocity <= 95 {
                "low"
            } else {
                "high"
            };
            manifest.push_str(&format!(
                "{index},{},{note},{velocity},{root},{layer}\n",
                index * 2
            ));
            index += 1;
        }
    }
    variable_length(1440, &mut track);
    track.extend([0xff, 0x2f, 0]);
    let mut midi = b"MThd\0\0\0\x06\0\0\0\x01\x01\xe0MTrk".to_vec();
    midi.extend((track.len() as u32).to_be_bytes());
    midi.extend(track);
    fs::write(output.join("boundary-sweep.mid"), midi)?;
    fs::write(output.join("probes.csv"), manifest)?;
    println!(
        "Generated six stereo takes and 45 probes in {}",
        output.display()
    );
    println!(
        "Move the instrument folder before loading/rendering; see docs/SAMPLER_ACCEPTANCE.md."
    );
    Ok(())
}
