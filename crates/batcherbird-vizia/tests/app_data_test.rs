#![allow(clippy::field_reassign_with_default)]

use batcherbird_vizia::app_data::{samples_to_peaks, AppData, AppState};
use batcherbird_vizia::app_event::InstrumentPreset;
use batcherbird_vizia::views::hit_test_note;

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

#[test]
fn test_gain_staging_evaluation() {
    // 1. Clipping (>= -0.1 dB or linear >= 0.99)
    let (msg, color) = AppData::evaluate_gain_staging(0.0, 1.0);
    assert!(msg.contains("Clipping detected"));
    assert_eq!(color, "#ff4444");

    let (msg_clip2, color_clip2) = AppData::evaluate_gain_staging(-0.05, 0.995);
    assert!(msg_clip2.contains("Clipping detected"));
    assert_eq!(color_clip2, "#ff4444");

    // 2. Hot signal (> -3.0 dB)
    let (msg_hot, color_hot) = AppData::evaluate_gain_staging(-1.5, 0.84);
    assert!(msg_hot.contains("Hot signal"));
    assert_eq!(color_hot, "#ffaa00");

    // 3. Optimal headroom (-18.0 dB to -3.0 dB)
    let (msg_opt, color_opt) = AppData::evaluate_gain_staging(-12.0, 0.25);
    assert!(msg_opt.contains("Optimal headroom"));
    assert_eq!(color_opt, "#00e676");

    let (msg_opt2, color_opt2) = AppData::evaluate_gain_staging(-3.1, 0.7);
    assert!(msg_opt2.contains("Optimal headroom"));
    assert_eq!(color_opt2, "#00e676");

    // 4. Low level (-45.0 dB to -18.0 dB)
    let (msg_low, color_low) = AppData::evaluate_gain_staging(-24.0, 0.063);
    assert!(msg_low.contains("Level low"));
    assert_eq!(color_low, "#4a9eff");

    // 5. No signal (<= -45.0 dB)
    let (msg_silent, color_silent) = AppData::evaluate_gain_staging(-60.0, 0.001);
    assert!(msg_silent.contains("No signal detected"));
    assert_eq!(color_silent, "#888899");
}

#[test]
fn test_set_start_and_end_note() {
    let mut data = AppData::default();
    // Default: 36 (C2) to 84 (C6), 1 layer
    assert_eq!(data.start_note, 36);
    assert_eq!(data.end_note, 84);

    // Set valid start note
    data.set_start_note(48); // C3
    assert_eq!(data.start_note, 48);
    assert_eq!(data.end_note, 84);
    assert_eq!(data.total_samples(), 37); // 48..=84 is 37 notes

    // Set start note higher than end note: auto-swap
    data.set_start_note(96);
    assert_eq!(data.start_note, 84);
    assert_eq!(data.end_note, 96);
    assert_eq!(data.total_samples(), 13); // 84..=96 is 13 notes

    // Set valid end note
    data.set_end_note(108);
    assert_eq!(data.start_note, 84);
    assert_eq!(data.end_note, 108);

    // Set end note lower than start note: auto-swap
    data.set_end_note(60);
    assert_eq!(data.start_note, 60);
    assert_eq!(data.end_note, 84);

    // Setting note capped at 127
    data.set_end_note(200);
    assert_eq!(data.end_note, 127);
}

#[test]
fn test_keyboard_hit_testing() {
    // 29 white keys for 4 octaves (C2..=C6, 36..=84)
    // 290.0 wide bounds => exactly 10.0 pixels per white key
    let bounds_x = 0.0;
    let bounds_y = 0.0;
    let bounds_w = 290.0;
    let bounds_h = 50.0;
    let display_start = 36; // C2
    let display_end = 84;   // C6

    // 1. Out-of-bounds checks
    assert_eq!(hit_test_note(bounds_x, bounds_y, bounds_w, bounds_h, display_start, display_end, -1.0, 25.0), None);
    assert_eq!(hit_test_note(bounds_x, bounds_y, bounds_w, bounds_h, display_start, display_end, 291.0, 25.0), None);
    assert_eq!(hit_test_note(bounds_x, bounds_y, bounds_w, bounds_h, display_start, display_end, 150.0, -1.0), None);
    assert_eq!(hit_test_note(bounds_x, bounds_y, bounds_w, bounds_h, display_start, display_end, 150.0, 51.0), None);
    assert_eq!(hit_test_note(bounds_x, bounds_y, 0.0, bounds_h, display_start, display_end, 10.0, 25.0), None);

    // 2. White key in bottom 40% (y = 40.0, bounds_h = 50.0 => below black keys):
    // First white key (0.0..10.0) is C2 (note 36)
    assert_eq!(hit_test_note(bounds_x, bounds_y, bounds_w, bounds_h, display_start, display_end, 5.0, 40.0), Some(36));
    // Second white key (10.0..20.0) is D2 (note 38)
    assert_eq!(hit_test_note(bounds_x, bounds_y, bounds_w, bounds_h, display_start, display_end, 15.0, 40.0), Some(38));
    // Third white key (20.0..30.0) is E2 (note 40)
    assert_eq!(hit_test_note(bounds_x, bounds_y, bounds_w, bounds_h, display_start, display_end, 25.0, 40.0), Some(40));

    // 3. Black key in top 60% (y = 15.0, bounds_h = 50.0 => black_h is 30.0):
    // Black key C#2 (note 37) is centered at x = 10.0 with width = 6.0 (7.0..13.0)
    assert_eq!(hit_test_note(bounds_x, bounds_y, bounds_w, bounds_h, display_start, display_end, 10.0, 15.0), Some(37));
    // Black key D#2 (note 39) is centered at x = 20.0 with width = 6.0 (17.0..23.0)
    assert_eq!(hit_test_note(bounds_x, bounds_y, bounds_w, bounds_h, display_start, display_end, 20.0, 15.0), Some(39));

    // 4. White key in top 60% away from black keys:
    // Left edge of C2 (e.g. x = 2.0, y = 15.0) is not covered by C#2 (starts at 7.0)
    assert_eq!(hit_test_note(bounds_x, bounds_y, bounds_w, bounds_h, display_start, display_end, 2.0, 15.0), Some(36));
}

#[test]
fn test_playthrough_settings() {
    let mut data = AppData::default();
    // Default: playthrough is disabled
    assert!(!data.playthrough_enabled);
    assert!(data.playthrough_stream.is_none());

    data.set_playthrough(true);
    assert!(data.playthrough_enabled);

    data.set_playthrough(false);
    assert!(!data.playthrough_enabled);
}

#[test]
fn test_channel_routing_selection_and_cycling() {
    use batcherbird_core::channel_routing::ChannelRouting;

    let mut data = AppData::default();
    // Default: Stereo (1+2)
    assert_eq!(data.channel_routing, ChannelRouting::Stereo);
    assert_eq!(data.selected_channel_routing, 0);
    assert_eq!(data.channel_routing_display, "Stereo (1+2)");

    // Select Mono In 1 (index 1)
    data.set_channel_routing_index(1);
    assert_eq!(data.channel_routing, ChannelRouting::MonoLeft);
    assert_eq!(data.selected_channel_routing, 1);
    assert_eq!(data.channel_routing_display, "Mono In 1 (L)");

    // Select Mono In 2 (index 2)
    data.set_channel_routing_index(2);
    assert_eq!(data.channel_routing, ChannelRouting::MonoRight);
    assert_eq!(data.selected_channel_routing, 2);
    assert_eq!(data.channel_routing_display, "Mono In 2 (R)");

    // Cycling wraps: 2 -> 0 -> 1 -> 2
    data.cycle_channel_routing();
    assert_eq!(data.channel_routing, ChannelRouting::Stereo);
    assert_eq!(data.selected_channel_routing, 0);

    data.cycle_channel_routing();
    assert_eq!(data.channel_routing, ChannelRouting::MonoLeft);
    assert_eq!(data.selected_channel_routing, 1);
}

#[test]
fn test_input_gain_controls() {
    let mut data = AppData::default();

    // Default: 0.0 dB
    assert_eq!(data.input_gain_db, 0.0);
    assert_eq!(data.input_gain_display, "0.0 dB");

    // Adjust +1.0 dB
    data.adjust_input_gain_db(1.0);
    assert_eq!(data.input_gain_db, 1.0);
    assert_eq!(data.input_gain_display, "+1.0 dB");

    // Adjust -2.5 dB
    data.adjust_input_gain_db(-2.5);
    assert_eq!(data.input_gain_db, -1.5);
    assert_eq!(data.input_gain_display, "-1.5 dB");

    // Test clamping to +12.0 dB max
    data.set_input_gain_db(18.0);
    assert_eq!(data.input_gain_db, 12.0);
    assert_eq!(data.input_gain_display, "+12.0 dB");

    // Test clamping to -12.0 dB min
    data.set_input_gain_db(-25.0);
    assert_eq!(data.input_gain_db, -12.0);
    assert_eq!(data.input_gain_display, "-12.0 dB");

    // Test reset
    data.reset_input_gain_db();
    assert_eq!(data.input_gain_db, 0.0);
    assert_eq!(data.input_gain_display, "0.0 dB");

    // Verify build_sampling_config carries input_gain_db
    data.set_input_gain_db(4.5);
    let config = data.build_sampling_config();
    assert_eq!(config.input_gain_db, 4.5);
}




