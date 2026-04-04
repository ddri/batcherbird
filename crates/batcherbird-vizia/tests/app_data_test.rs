use batcherbird_vizia::app_data::{AppData, AppState};
use batcherbird_vizia::app_event::AppEvent;

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
