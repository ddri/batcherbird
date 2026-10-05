//! Shared deterministic audio fixture for workflow tests and manual QA.
use batcherbird_core::sampler::Sample;
use std::time::{Duration, UNIX_EPOCH};

/// Deliberately quiet 125ms attack, sustained tone, 667ms release and silence.
/// Distinct right-channel polarity/amplitude makes stereo corruption observable.
pub fn take(note: u8, velocity: u8, gain: f32) -> Sample {
    let rate = 48_000;
    let frequency = 440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0);
    let audio_data = (0..72_000)
        .flat_map(|frame| {
            let amplitude = match frame {
                0..=3999 => 0.0,
                4000..=9999 => 0.003,
                10000..=19999 => 0.18,
                20000..=51999 => 0.18 * (-4.0f32 * (frame - 20000) as f32 / 32000.0).exp(),
                _ => 0.0,
            };
            let left = gain
                * (0.5 + velocity as f32 / 254.0)
                * amplitude
                * (std::f32::consts::TAU * frequency * frame as f32 / rate as f32).sin();
            [left, -0.35 * left]
        })
        .collect();
    Sample {
        note,
        velocity,
        audio_data,
        sample_rate: rate,
        channels: 2,
        recorded_at: UNIX_EPOCH + Duration::from_secs(123),
        midi_timing: Duration::from_millis(400),
        audio_timing: Duration::from_millis(1500),
    }
}
pub fn instrument() -> Vec<Sample> {
    [60, 63, 66]
        .into_iter()
        .flat_map(|note| [64, 127].map(|velocity| take(note, velocity, 1.0)))
        .collect()
}
