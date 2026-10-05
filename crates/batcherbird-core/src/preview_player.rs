//! In-memory one-shot preview player.
//!
//! Plays a recorded sample (interleaved `f32`) ONCE on the default output
//! device. Designed for the GUI Review screen's Play/Stop controls.
//!
//! Real-time safety (see CLAUDE.md audio rules): the cpal output callback does
//! NO allocation, NO locking, and NO blocking. It only performs atomic loads /
//! stores and copies from a shared immutable [`Arc<[f32]>`] buffer. The audio
//! is never mutated after construction, so no lock is needed to read it.

use crate::audio::AudioManager;
use crate::{BatcherbirdError, Result};
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::SampleFormat;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// Map a single source frame to a single destination frame, handling channel
/// count mismatches without allocation.
///
/// - Mono source -> any destination: the mono sample is written to every
///   destination channel (duplicated).
/// - Stereo (or wider) source -> mono destination: the source channels are
///   averaged into the single destination channel.
/// - Otherwise: channels are copied 1:1 up to `min(src_ch, dst_ch)`, and any
///   extra destination channels are filled with silence.
///
/// `src` must contain at least `src_ch` samples; `dst` at least `dst_ch`.
/// This is a pure function (no allocation) and is unit-tested below.
pub fn map_frame(src: &[f32], src_ch: usize, dst_ch: usize, dst: &mut [f32]) {
    debug_assert!(src.len() >= src_ch);
    debug_assert!(dst.len() >= dst_ch);

    if src_ch == 1 {
        // Mono -> N channels: duplicate.
        let s = src[0];
        for d in dst.iter_mut().take(dst_ch) {
            *d = s;
        }
        return;
    }

    if dst_ch == 1 {
        // N channels -> mono: average all source channels.
        let mut sum = 0.0f32;
        for &s in src.iter().take(src_ch) {
            sum += s;
        }
        dst[0] = sum / src_ch as f32;
        return;
    }

    // General case: copy matching channels, zero-fill the rest.
    let common = src_ch.min(dst_ch);
    dst[..common].copy_from_slice(&src[..common]);
    for d in dst.iter_mut().take(dst_ch).skip(common) {
        *d = 0.0;
    }
}

/// State shared with the cpal output callback. All fields are read/written
/// with atomics or are immutable, so the callback never locks.
struct Shared {
    /// Immutable interleaved source audio. Never mutated after construction.
    audio: Arc<[f32]>,
    /// Number of channels in `audio`.
    src_channels: usize,
    /// Number of source frames (`audio.len() / src_channels`).
    src_frames: usize,
    /// Next source frame index to play.
    cursor: AtomicUsize,
    /// Whether playback is active. The callback outputs silence when false.
    playing: AtomicBool,
}

/// A one-shot, in-memory preview player. Holds the live cpal stream; dropping
/// it stops audio.
pub struct PreviewPlayer {
    _stream: cpal::Stream,
    shared: Arc<Shared>,
}

impl PreviewPlayer {
    /// Start playing `audio` (interleaved, `channels`-ch at `sample_rate`) once
    /// on the default output device. Returns `Err` if there is no output device
    /// or the stream fails to build/start.
    ///
    /// Prefers the original sample rate where the device supports it. Otherwise
    /// resamples before starting the stream, preserving playback speed and pitch.
    pub fn play(audio: Arc<[f32]>, sample_rate: u32, channels: u16) -> Result<Self> {
        if sample_rate == 0 || channels == 0 || !audio.len().is_multiple_of(channels as usize) {
            return Err(BatcherbirdError::Config(
                "Preview requires a positive rate and channels with complete audio frames".into(),
            ));
        }
        let manager = AudioManager::new()?;
        let device = manager.get_default_output_device()?;
        let default = device.default_output_config().map_err(|e| {
            BatcherbirdError::Audio(format!("Failed to get default output config: {e}"))
        })?;
        // Avoid resampling whenever the device accepts the source rate while
        // keeping its normal channel topology and sample format.
        let supported = device
            .supported_output_configs()
            .ok()
            .and_then(|mut configs| {
                configs
                    .find(|c| {
                        c.channels() == default.channels()
                            && c.sample_format() == default.sample_format()
                            && c.min_sample_rate().0 <= sample_rate
                            && c.max_sample_rate().0 >= sample_rate
                    })
                    .map(|c| c.with_sample_rate(cpal::SampleRate(sample_rate)))
            })
            .unwrap_or(default);
        let sample_format = supported.sample_format();
        let config: cpal::StreamConfig = supported.into();
        let dst_channels = config.channels as usize;
        let prepared = prepare_preview_audio(
            &audio,
            channels as usize,
            sample_rate,
            dst_channels,
            config.sample_rate.0,
        )?;
        let src_frames = prepared.len() / dst_channels;
        let shared = Arc::new(Shared {
            audio: prepared.into(),
            src_channels: dst_channels,
            src_frames,
            cursor: AtomicUsize::new(0),
            playing: AtomicBool::new(true),
        });

        let stream = match sample_format {
            SampleFormat::F32 => {
                build_stream::<f32>(&device, &config, dst_channels, shared.clone())?
            }
            SampleFormat::F64 => {
                build_stream::<f64>(&device, &config, dst_channels, shared.clone())?
            }
            SampleFormat::I8 => build_stream::<i8>(&device, &config, dst_channels, shared.clone())?,
            SampleFormat::I16 => {
                build_stream::<i16>(&device, &config, dst_channels, shared.clone())?
            }
            SampleFormat::I32 => {
                build_stream::<i32>(&device, &config, dst_channels, shared.clone())?
            }
            SampleFormat::I64 => {
                build_stream::<i64>(&device, &config, dst_channels, shared.clone())?
            }
            SampleFormat::U8 => build_stream::<u8>(&device, &config, dst_channels, shared.clone())?,
            SampleFormat::U16 => {
                build_stream::<u16>(&device, &config, dst_channels, shared.clone())?
            }
            SampleFormat::U32 => {
                build_stream::<u32>(&device, &config, dst_channels, shared.clone())?
            }
            SampleFormat::U64 => {
                build_stream::<u64>(&device, &config, dst_channels, shared.clone())?
            }
            other => {
                return Err(BatcherbirdError::Audio(format!(
                    "Unsupported preview format: {other:?}"
                )))
            }
        };

        stream.play().map_err(|e| {
            BatcherbirdError::Audio(format!("Failed to start preview stream: {}", e))
        })?;

        Ok(Self {
            _stream: stream,
            shared,
        })
    }

    /// Stop playback. The callback then outputs silence on subsequent calls.
    pub fn stop(&self) {
        self.shared.playing.store(false, Ordering::Relaxed);
    }

    /// Fraction of the take submitted to the audio device, clamped to 0..=1.
    /// The device's output buffering can introduce a small audible delay.
    pub fn playback_position(&self) -> f32 {
        if self.shared.src_frames == 0 {
            return 1.0;
        }
        (self.shared.cursor.load(Ordering::Relaxed) as f32 / self.shared.src_frames as f32)
            .clamp(0.0, 1.0)
    }

    /// `true` once playback has reached the end of the buffer or been stopped.
    pub fn is_finished(&self) -> bool {
        !self.shared.playing.load(Ordering::Relaxed)
            || self.shared.cursor.load(Ordering::Relaxed) >= self.shared.src_frames
    }
}

/// Prepare channels and rate off the audio callback. Linear interpolation is
/// intended for audition; original recorded/exported audio is never modified.
fn prepare_preview_audio(
    audio: &[f32],
    src_channels: usize,
    src_rate: u32,
    dst_channels: usize,
    dst_rate: u32,
) -> Result<Vec<f32>> {
    if src_channels == 0
        || dst_channels == 0
        || src_rate == 0
        || dst_rate == 0
        || !audio.len().is_multiple_of(src_channels)
    {
        return Err(BatcherbirdError::Config(
            "Invalid preview audio format".into(),
        ));
    }
    let frames = audio.len() / src_channels;
    let output_frames = (frames as u64)
        .checked_mul(dst_rate as u64)
        .and_then(|n| n.checked_add(src_rate as u64 / 2))
        .and_then(|n| usize::try_from(n / src_rate as u64).ok())
        .ok_or_else(|| BatcherbirdError::Config("Preview is too large".into()))?;
    let length = output_frames
        .checked_mul(dst_channels)
        .ok_or_else(|| BatcherbirdError::Config("Preview is too large".into()))?;
    let mut prepared = vec![0.0; length];
    let mut interpolated = vec![0.0; src_channels];
    for (index, output) in prepared.chunks_exact_mut(dst_channels).enumerate() {
        let position = index as f64 * src_rate as f64 / dst_rate as f64;
        let low = (position.floor() as usize).min(frames - 1);
        let high = (low + 1).min(frames - 1);
        let fraction = (position - low as f64) as f32;
        for (channel, value) in interpolated.iter_mut().enumerate() {
            let first = audio[low * src_channels + channel];
            *value = first + (audio[high * src_channels + channel] - first) * fraction;
        }
        map_frame(&interpolated, src_channels, dst_channels, output);
    }
    Ok(prepared)
}

/// Output callback reads immutable prepared audio with no allocation or locks.
fn render_output<T>(output: &mut [T], dst_channels: usize, shared: &Shared)
where
    T: cpal::Sample + cpal::FromSample<f32>,
{
    output.fill(T::EQUILIBRIUM);
    if !shared.playing.load(Ordering::Relaxed) {
        return;
    }
    let out_frames = output.len() / dst_channels;
    let start = shared.cursor.load(Ordering::Relaxed);
    let end = start.saturating_add(out_frames).min(shared.src_frames);
    let source = &shared.audio[start * shared.src_channels..end * shared.src_channels];
    for (destination, &sample) in output.iter_mut().zip(source) {
        *destination = T::from_sample_(if sample.is_finite() {
            sample.clamp(-1.0, 1.0)
        } else {
            0.0
        });
    }
    shared.cursor.store(end, Ordering::Relaxed);
    if end == shared.src_frames {
        shared.playing.store(false, Ordering::Relaxed);
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    dst_channels: usize,
    shared: Arc<Shared>,
) -> Result<cpal::Stream>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    let error_state = Arc::clone(&shared);
    device
        .build_output_stream(
            config,
            move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
                render_output(output, dst_channels, &shared)
            },
            move |err| {
                error_state.playing.store(false, Ordering::Relaxed);
                tracing::error!("Preview output stream error: {err}");
            },
            None,
        )
        .map_err(|e| BatcherbirdError::Audio(format!("Failed to build preview output stream: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resampling_preserves_duration_pitch_and_stereo_alignment() {
        let source_rate = 44_100;
        let output_rate = 48_000;
        let audio: Vec<f32> = (0..source_rate)
            .flat_map(|i| {
                let value = (std::f32::consts::TAU * 440.0 * i as f32 / source_rate as f32).sin();
                [value, -value]
            })
            .collect();
        let converted = prepare_preview_audio(&audio, 2, source_rate, 2, output_rate).unwrap();
        assert_eq!(converted.len(), 48_000 * 2, "one second stays one second");
        assert!(converted
            .as_chunks::<2>()
            .0
            .iter()
            .all(|frame| (frame[0] + frame[1]).abs() < 1e-6));
        let upward_crossings = converted
            .as_chunks::<2>()
            .0
            .iter()
            .map(|f| f[0])
            .collect::<Vec<_>>()
            .windows(2)
            .filter(|pair| pair[0] <= 0.0 && pair[1] > 0.0)
            .count();
        assert!(
            (439..=441).contains(&upward_crossings),
            "440Hz tone must keep its pitch"
        );
    }

    #[test]
    fn native_rate_preserves_samples_and_maps_mono() {
        let converted = prepare_preview_audio(&[0.1, -0.7, 0.4], 1, 48_000, 2, 48_000).unwrap();
        assert_eq!(converted, vec![0.1, 0.1, -0.7, -0.7, 0.4, 0.4]);
        assert!(prepare_preview_audio(&[0.1], 2, 48_000, 2, 48_000).is_err());
        assert!(prepare_preview_audio(&[], 1, 48_000, 2, 48_000)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn output_tail_is_silent_and_cursor_finishes_at_final_frame() {
        let shared = Shared {
            audio: Arc::from([0.5, -0.5, 0.25, -0.25]),
            src_channels: 2,
            src_frames: 2,
            cursor: AtomicUsize::new(0),
            playing: AtomicBool::new(true),
        };
        let mut output = [9.0f32; 6];
        render_output(&mut output, 2, &shared);
        assert_eq!(output, [0.5, -0.5, 0.25, -0.25, 0.0, 0.0]);
        assert_eq!(shared.cursor.load(Ordering::Relaxed), 2);
        assert!(!shared.playing.load(Ordering::Relaxed));
        let mut unsigned = [0u16; 4];
        render_output(&mut unsigned, 2, &shared);
        assert_eq!(unsigned, [32768; 4], "unsigned PCM silence is midpoint");
    }

    #[test]
    fn integer_output_converts_signal_and_clamps_excess_gain() {
        let shared = Shared {
            audio: Arc::from([0.5, -0.5, 4.0, f32::NAN]),
            src_channels: 1,
            src_frames: 4,
            cursor: AtomicUsize::new(0),
            playing: AtomicBool::new(true),
        };
        let mut output = [0i16; 4];
        render_output(&mut output, 1, &shared);
        assert_eq!(output[0], 16384);
        assert_eq!(output[1], -16384);
        assert_eq!(output[2], i16::MAX);
        assert_eq!(output[3], 0);
    }

    #[test]
    fn map_frame_mono_to_stereo_duplicates() {
        let src = [0.5f32];
        let mut dst = [0.0f32; 2];
        map_frame(&src, 1, 2, &mut dst);
        assert_eq!(dst, [0.5, 0.5]);
    }

    #[test]
    fn map_frame_stereo_to_mono_averages() {
        let src = [0.2f32, 0.8f32];
        let mut dst = [0.0f32; 1];
        map_frame(&src, 2, 1, &mut dst);
        assert!((dst[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn map_frame_stereo_to_stereo_passthrough() {
        let src = [-0.3f32, 0.7f32];
        let mut dst = [0.0f32; 2];
        map_frame(&src, 2, 2, &mut dst);
        assert_eq!(dst, [-0.3, 0.7]);
    }

    #[test]
    fn map_frame_mono_to_mono() {
        let src = [0.42f32];
        let mut dst = [0.0f32; 1];
        map_frame(&src, 1, 1, &mut dst);
        assert_eq!(dst, [0.42]);
    }

    #[test]
    fn map_frame_stereo_to_more_channels_zero_fills() {
        let src = [0.1f32, 0.2f32];
        let mut dst = [9.0f32; 4];
        map_frame(&src, 2, 4, &mut dst);
        assert_eq!(dst, [0.1, 0.2, 0.0, 0.0]);
    }

    #[test]
    fn map_frame_wider_source_to_stereo_truncates() {
        // 4-channel source down to stereo: take the first two channels.
        let src = [0.1f32, 0.2f32, 0.3f32, 0.4f32];
        let mut dst = [0.0f32; 2];
        map_frame(&src, 4, 2, &mut dst);
        assert_eq!(dst, [0.1, 0.2]);
    }
}
