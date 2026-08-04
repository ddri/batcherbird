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
    /// If the source `sample_rate` differs from the device's rate, playback
    /// pitch will be off (no resampling is performed); a warning is logged.
    pub fn play(audio: Arc<[f32]>, sample_rate: u32, channels: u16) -> Result<Self> {
        let manager = AudioManager::new()?;
        let device = manager.get_default_output_device()?;

        let supported = device.default_output_config().map_err(|e| {
            BatcherbirdError::Audio(format!("Failed to get default output config: {}", e))
        })?;
        let sample_format = supported.sample_format();
        let config: cpal::StreamConfig = supported.into();
        let dst_channels = config.channels as usize;

        if config.sample_rate.0 != sample_rate {
            tracing::warn!(
                "Preview sample rate {} Hz differs from output device rate {} Hz; \
                 playback pitch will be off (no resampling).",
                sample_rate,
                config.sample_rate.0
            );
        }

        let src_channels = (channels as usize).max(1);
        let src_frames = if src_channels == 0 {
            0
        } else {
            audio.len() / src_channels
        };

        let shared = Arc::new(Shared {
            audio,
            src_channels,
            src_frames,
            cursor: AtomicUsize::new(0),
            playing: AtomicBool::new(true),
        });

        let stream = match sample_format {
            SampleFormat::F32 => build_stream(&device, &config, dst_channels, shared.clone())?,
            other => {
                return Err(BatcherbirdError::Audio(format!(
                    "Unsupported output sample format for preview: {:?}",
                    other
                )));
            }
        };

        stream
            .play()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to start preview stream: {}", e)))?;

        Ok(Self {
            _stream: stream,
            shared,
        })
    }

    /// Stop playback. The callback then outputs silence on subsequent calls.
    pub fn stop(&self) {
        self.shared.playing.store(false, Ordering::Relaxed);
    }

    /// `true` once playback has reached the end of the buffer or been stopped.
    pub fn is_finished(&self) -> bool {
        !self.shared.playing.load(Ordering::Relaxed)
            || self.shared.cursor.load(Ordering::Relaxed) >= self.shared.src_frames
    }
}

/// Build the F32 output stream. The data callback is allocation- and lock-free:
/// it only reads atomics and copies from the immutable `shared.audio` buffer.
fn build_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    dst_channels: usize,
    shared: Arc<Shared>,
) -> Result<cpal::Stream> {
    let err_fn = |err| tracing::error!("Preview output stream error: {}", err);

    device
        .build_output_stream(
            config,
            move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Not playing: write silence and return. No allocation, no lock.
                if !shared.playing.load(Ordering::Relaxed) {
                    for s in output.iter_mut() {
                        *s = 0.0;
                    }
                    return;
                }

                let src_ch = shared.src_channels;
                let audio = &shared.audio;
                let total_frames = shared.src_frames;

                // Reserve a contiguous run of source frames for this callback.
                let out_frames = output.len() / dst_channels;
                let start = shared.cursor.fetch_add(out_frames, Ordering::Relaxed);

                for (i, frame) in output.chunks_mut(dst_channels).enumerate() {
                    let src_frame = start + i;
                    if src_frame >= total_frames {
                        for s in frame.iter_mut() {
                            *s = 0.0;
                        }
                        continue;
                    }
                    let base = src_frame * src_ch;
                    // `base + src_ch <= audio.len()` because src_frame < src_frames.
                    let src_slice = &audio[base..base + src_ch];
                    map_frame(src_slice, src_ch, dst_channels, frame);
                }

                // If we've consumed the whole buffer, stop. Clamp the cursor so
                // is_finished() reports end-of-stream rather than overflowing.
                if start + out_frames >= total_frames {
                    shared.cursor.store(total_frames, Ordering::Relaxed);
                    shared.playing.store(false, Ordering::Relaxed);
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| BatcherbirdError::Audio(format!("Failed to build preview output stream: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

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
