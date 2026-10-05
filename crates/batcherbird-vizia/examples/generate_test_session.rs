//! Generate a portable session for manual QA without connecting any hardware.
use batcherbird_vizia::session::{save_session, SessionSettings};
use std::path::PathBuf;

#[path = "../tests/support/workflow_fixture.rs"]
mod fixtures;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(std::env::args_os().nth(1).ok_or(
        "Usage: cargo run -p batcherbird-vizia --example generate_test_session -- OUTPUT.batcherbird"
    )?);
    let samples = fixtures::instrument();
    let settings = SessionSettings {
        instrument_name: "QA quiet attack and long release".into(),
        start_note: 60,
        end_note: 66,
        note_step: 3,
        velocity_layers: 2,
        note_duration_ms: 500,
        release_duration_ms: 700,
        export_format: 6,
        output_directory: output
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join("QA exports"),
        ..SessionSettings::default()
    };
    save_session(&output, &settings, &samples)?;
    println!("Saved {} samples to {}", samples.len(), output.display());
    println!("48kHz stereo, notes C4/D#4/F#4, velocities 64/127, quiet attacks and long releases.");
    Ok(())
}
