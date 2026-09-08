use batcherbird_vizia::app_data::{samples_to_peaks, AppData, AppState};
use batcherbird_vizia::app_event::InstrumentPreset;

#[test]
fn initial_state_is_idle() {
    let data = AppData::default();
    assert!(matches!(data.app_state, AppState::Idle));
}

#[test]
fn default_sampling_config() {
    let data = AppData::default();
    assert_eq!(data.start_note, 36); // C2
    assert_eq!(data.end_note, 84); // C5
    assert_eq!(data.velocity_layers, 1);
    assert_eq!(data.note_duration_ms, 2000);
}

#[test]
fn total_notes_calculation() {
    let mut data = AppData::default();
    data.start_note = 60; // C4
    data.end_note = 72; // C5
    data.velocity_layers = 3;
    // 13 notes (C4 to C5 inclusive) * 3 layers = 39
    assert_eq!(data.total_samples(), 39);
}

#[test]
fn single_note_mode() {
    let mut data = AppData::default();
    data.start_note = 60;
    data.end_note = 60;
    data.velocity_layers = 1;
    assert_eq!(data.total_samples(), 1);
}

#[test]
fn note_to_name_conversion() {
    assert_eq!(AppData::note_name(60), "C4");
    assert_eq!(AppData::note_name(69), "A4");
    assert_eq!(AppData::note_name(36), "C2");
    assert_eq!(AppData::note_name(84), "C6");
}

#[test]
fn peaks_empty_input_is_empty() {
    assert!(samples_to_peaks(&[], 512).is_empty());
    // Zero buckets also yields empty.
    assert!(samples_to_peaks(&[0.1, 0.2, 0.3], 0).is_empty());
}

#[test]
fn peaks_takes_max_abs_per_bucket() {
    // 4 samples, 2 buckets => 2 per bucket; max abs of each pair.
    let audio = [0.1, -0.5, 0.25, -0.2];
    let peaks = samples_to_peaks(&audio, 2);
    assert_eq!(peaks.len(), 2);
    assert!((peaks[0] - 0.5).abs() < 1e-6);
    assert!((peaks[1] - 0.25).abs() < 1e-6);
}

#[test]
fn peaks_normalized_at_or_below_one() {
    // Values exceeding 1.0 (and below -1.0) are clamped to <= 1.0.
    let audio = [2.0, -3.0, 0.5, 1.5];
    let peaks = samples_to_peaks(&audio, 4);
    assert!(!peaks.is_empty());
    for p in &peaks {
        assert!(*p <= 1.0, "peak {} should be <= 1.0", p);
        assert!(*p >= 0.0, "peak {} should be >= 0.0", p);
    }
}

#[test]
fn peaks_buckets_capped_by_input_len() {
    // Fewer samples than buckets => at most input-len entries.
    let audio = [0.3, 0.6, 0.9];
    let peaks = samples_to_peaks(&audio, 512);
    assert_eq!(peaks.len(), 3);
    assert!((peaks[0] - 0.3).abs() < 1e-6);
    assert!((peaks[2] - 0.9).abs() < 1e-6);
}

#[test]
fn stepped_notes_calculation() {
    let mut data = AppData::default();
    data.start_note = 60; // C4
    data.end_note = 72;   // C5
    data.velocity_layers = 2;
    data.note_step = 3;   // Every 3rd note: 60, 63, 66, 69, 72 = 5 notes
    assert_eq!(data.total_samples(), 10);

    // Every octave (step 12) from C2 (36) to C6 (84): 36, 48, 60, 72, 84 = 5 notes
    data.start_note = 36;
    data.end_note = 84;
    data.note_step = 12;
    data.velocity_layers = 1;
    assert_eq!(data.total_samples(), 5);
}

#[test]
fn duration_display_and_summary() {
    let mut data = AppData::default();
    data.start_note = 60;
    data.end_note = 72;
    data.note_step = 12; // 2 notes: 60, 72
    data.velocity_layers = 1;
    data.note_duration_ms = 2000;
    // 2 samples * (2000ms + 1500ms) = 7.0s
    assert_eq!(data.estimated_duration_display(), "~7s");

    data.update_summary();
    assert_eq!(data.session_summary_display, "2 samples • ~7s");
}

#[test]
fn instrument_presets_application() {
    let mut data = AppData::default();

    // Lead: C3 to C5, step 1, 2 layers, 2s duration
    data.apply_instrument_preset(InstrumentPreset::Lead);
    assert_eq!(data.start_note, 48);
    assert_eq!(data.end_note, 72);
    assert_eq!(data.note_step, 1);
    assert_eq!(data.velocity_layers, 2);
    assert_eq!(data.note_duration_ms, 2000);
    // 25 notes * 2 layers = 50 samples
    assert_eq!(data.total_samples(), 50);

    // Pad: C2 to C6, step 3, 2 layers, 4s duration
    data.apply_instrument_preset(InstrumentPreset::Pad);
    assert_eq!(data.start_note, 36);
    assert_eq!(data.end_note, 84);
    assert_eq!(data.note_step, 3);
    assert_eq!(data.velocity_layers, 2);
    assert_eq!(data.note_duration_ms, 4000);
    // ((84 - 36) / 3 + 1) * 2 = 17 * 2 = 34 samples
    assert_eq!(data.total_samples(), 34);

    // Bass: C1 to C3, step 1, 2 layers, 1.5s duration
    data.apply_instrument_preset(InstrumentPreset::Bass);
    assert_eq!(data.start_note, 24);
    assert_eq!(data.end_note, 48);
    assert_eq!(data.note_step, 1);
    assert_eq!(data.velocity_layers, 2);
    assert_eq!(data.note_duration_ms, 1500);
    // 25 notes * 2 layers = 50 samples
    assert_eq!(data.total_samples(), 50);

    // Pluck: C2 to C4, step 1, 4 layers, 1s duration
    data.apply_instrument_preset(InstrumentPreset::Pluck);
    assert_eq!(data.start_note, 36);
    assert_eq!(data.end_note, 60);
    assert_eq!(data.note_step, 1);
    assert_eq!(data.velocity_layers, 4);
    assert_eq!(data.note_duration_ms, 1000);
    // 25 notes * 4 layers = 100 samples
    assert_eq!(data.total_samples(), 100);
}
