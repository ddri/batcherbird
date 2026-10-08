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
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

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
    /// Explicit stop / output error bypasses the natural drain deadline.
    stopped: AtomicBool,
    output_failed: AtomicBool,
    /// Monotonic wall-clock origin and estimated audible completion deadline.
    origin: Instant,
    finish_deadline_ns: AtomicU64,
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
        Self::play_configured(audio, sample_rate, channels, &device, supported)
    }

    // Private acceptance entry point: exercises the production preparation and
    // callback on a named virtual device without changing system defaults.
    fn play_configured(
        audio: Arc<[f32]>,
        sample_rate: u32,
        channels: u16,
        device: &cpal::Device,
        supported: cpal::SupportedStreamConfig,
    ) -> Result<Self> {
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
            stopped: AtomicBool::new(false),
            output_failed: AtomicBool::new(false),
            origin: Instant::now(),
            finish_deadline_ns: AtomicU64::new(u64::MAX),
        });

        let stream = match sample_format {
            SampleFormat::F32 => {
                build_stream::<f32>(device, &config, dst_channels, shared.clone())?
            }
            SampleFormat::F64 => {
                build_stream::<f64>(device, &config, dst_channels, shared.clone())?
            }
            SampleFormat::I8 => build_stream::<i8>(device, &config, dst_channels, shared.clone())?,
            SampleFormat::I16 => {
                build_stream::<i16>(device, &config, dst_channels, shared.clone())?
            }
            SampleFormat::I32 => {
                build_stream::<i32>(device, &config, dst_channels, shared.clone())?
            }
            SampleFormat::I64 => {
                build_stream::<i64>(device, &config, dst_channels, shared.clone())?
            }
            SampleFormat::U8 => build_stream::<u8>(device, &config, dst_channels, shared.clone())?,
            SampleFormat::U16 => {
                build_stream::<u16>(device, &config, dst_channels, shared.clone())?
            }
            SampleFormat::U32 => {
                build_stream::<u32>(device, &config, dst_channels, shared.clone())?
            }
            SampleFormat::U64 => {
                build_stream::<u64>(device, &config, dst_channels, shared.clone())?
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
        self.shared.stop();
    }

    /// A runtime output failure ends playback and can be surfaced by the UI.
    pub fn has_output_error(&self) -> bool {
        self.shared.output_failed.load(Ordering::Acquire)
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

    /// `true` after the final submitted buffer has drained, or after an explicit stop/error.
    pub fn is_finished(&self) -> bool {
        self.shared.is_finished_at(elapsed_ns(self.shared.origin))
    }
}

impl Shared {
    fn stop(&self) {
        self.stopped.store(true, Ordering::Release);
        self.playing.store(false, Ordering::Relaxed);
    }

    fn is_finished_at(&self, now_ns: u64) -> bool {
        self.stopped.load(Ordering::Acquire)
            || self.src_frames == 0
            || (!self.playing.load(Ordering::Relaxed)
                && now_ns >= self.finish_deadline_ns.load(Ordering::Acquire))
    }
}

fn elapsed_ns(origin: Instant) -> u64 {
    origin.elapsed().as_nanos().min(u64::MAX as u128) as u64
}

/// Include device queue latency plus the valid final frames, rather than
/// treating "submitted" as "played". Backend timestamps may omit device latency,
/// so always allow at least 100ms or two full output buffers before the final
/// valid frames, while honoring larger reported queue delays. Audio never blocks.
fn drain_deadline_ns(
    now_ns: u64,
    queue_delay: Option<Duration>,
    valid_frames: usize,
    buffer_frames: usize,
    sample_rate: u32,
) -> u64 {
    let frame_duration =
        |frames: usize| Duration::from_secs_f64(frames as f64 / sample_rate as f64);
    let delay = queue_delay
        .unwrap_or_default()
        .max(Duration::from_millis(100))
        .max(frame_duration(buffer_frames).saturating_mul(2));
    let remaining = delay.saturating_add(frame_duration(valid_frames));
    now_ns.saturating_add(remaining.as_nanos().min(u64::MAX as u128) as u64)
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
fn render_output<T>(output: &mut [T], dst_channels: usize, shared: &Shared) -> Option<usize>
where
    T: cpal::Sample + cpal::FromSample<f32>,
{
    output.fill(T::EQUILIBRIUM);
    if !shared.playing.load(Ordering::Relaxed) {
        return None;
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
        return Some(end - start);
    }
    None
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
    let sample_rate = config.sample_rate.0;
    device
        .build_output_stream(
            config,
            move |output: &mut [T], info: &cpal::OutputCallbackInfo| {
                let callback_now = elapsed_ns(shared.origin);
                if let Some(valid_frames) = render_output(output, dst_channels, &shared) {
                    let timestamp = info.timestamp();
                    let queue_delay = timestamp.playback.duration_since(&timestamp.callback);
                    shared.finish_deadline_ns.store(
                        drain_deadline_ns(
                            callback_now,
                            queue_delay,
                            valid_frames,
                            output.len() / dst_channels,
                            sample_rate,
                        ),
                        Ordering::Release,
                    );
                }
            },
            move |err| {
                error_state.output_failed.store(true, Ordering::Release);
                error_state.stop();
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
    fn submitted_final_buffer_waits_for_playback_deadline() {
        let shared = Shared {
            audio: Arc::from([0.25, -0.25, 0.75, -0.75]),
            src_channels: 2,
            src_frames: 2,
            cursor: AtomicUsize::new(0),
            playing: AtomicBool::new(true),
            stopped: AtomicBool::new(false),
            output_failed: AtomicBool::new(false),
            origin: Instant::now(),
            finish_deadline_ns: AtomicU64::new(u64::MAX),
        };
        let mut output = [0.0f32; 8];
        let final_frames = render_output(&mut output, 2, &shared).unwrap();
        assert_eq!(final_frames, 2);
        assert_eq!(&output[..4], &[0.25, -0.25, 0.75, -0.75]);
        assert!(
            !shared.is_finished_at(1_000_000_000),
            "submission alone must not destroy the stream"
        );
        let deadline = drain_deadline_ns(
            1_000_000_000,
            Some(Duration::from_millis(40)),
            final_frames,
            4,
            1000,
        );
        assert_eq!(
            deadline, 1_102_000_000,
            "conservative minimum plus two valid frames"
        );
        shared.finish_deadline_ns.store(deadline, Ordering::Release);
        assert!(!shared.is_finished_at(deadline - 1));
        assert!(shared.is_finished_at(deadline));
        let mut subsequent = [9.0f32; 8];
        assert!(render_output(&mut subsequent, 2, &shared).is_none());
        assert_eq!(subsequent, [0.0; 8]);
        assert_eq!(
            shared.finish_deadline_ns.load(Ordering::Acquire),
            deadline,
            "silent callbacks must not postpone completion"
        );
    }

    #[test]
    fn timestamp_budget_has_conservative_floor_honors_large_delays_and_stop_is_immediate() {
        assert_eq!(drain_deadline_ns(0, None, 480, 480, 48_000), 110_000_000);
        assert_eq!(drain_deadline_ns(0, None, 9600, 9600, 48_000), 600_000_000);
        assert_eq!(
            drain_deadline_ns(0, Some(Duration::ZERO), 480, 480, 48_000),
            110_000_000
        );
        assert_eq!(
            drain_deadline_ns(0, Some(Duration::from_millis(20)), 9600, 9600, 48_000),
            600_000_000
        );
        assert_eq!(
            drain_deadline_ns(0, Some(Duration::from_millis(250)), 480, 480, 48_000),
            260_000_000
        );
        let shared = Shared {
            audio: Arc::from([0.5]),
            src_channels: 1,
            src_frames: 1,
            cursor: AtomicUsize::new(0),
            playing: AtomicBool::new(true),
            stopped: AtomicBool::new(false),
            output_failed: AtomicBool::new(false),
            origin: Instant::now(),
            finish_deadline_ns: AtomicU64::new(u64::MAX),
        };
        shared.stop();
        assert!(shared.is_finished_at(0));
        let mut output = [3.0f32; 2];
        assert!(render_output(&mut output, 1, &shared).is_none());
        assert_eq!(output, [0.0; 2]);
    }

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
            stopped: AtomicBool::new(false),
            output_failed: AtomicBool::new(false),
            origin: Instant::now(),
            finish_deadline_ns: AtomicU64::new(u64::MAX),
        };
        let mut output = [9.0f32; 6];
        let _ = render_output(&mut output, 2, &shared);
        assert_eq!(output, [0.5, -0.5, 0.25, -0.25, 0.0, 0.0]);
        assert_eq!(shared.cursor.load(Ordering::Relaxed), 2);
        assert!(!shared.playing.load(Ordering::Relaxed));
        let mut unsigned = [0u16; 4];
        let _ = render_output(&mut unsigned, 2, &shared);
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
            stopped: AtomicBool::new(false),
            output_failed: AtomicBool::new(false),
            origin: Instant::now(),
            finish_deadline_ns: AtomicU64::new(u64::MAX),
        };
        let mut output = [0i16; 4];
        let _ = render_output(&mut output, 1, &shared);
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

#[cfg(test)]
mod driver_acceptance {
    use super::*;
    use rtrb::{Consumer, RingBuffer};
    use std::fs;
    use std::path::Path;

    const RATE: u32 = 48_000;
    const DEVICE: &str = "BlackHole 16ch";
    const OFFSET: usize = 2; // Zero-based: reserved channels 3 and 4 only.

    struct Capture {
        _stream: cpal::Stream,
        consumer: Consumer<f32>,
        failed: Arc<AtomicBool>,
        samples: Vec<f32>,
    }
    impl Capture {
        fn start(device: &cpal::Device) -> Result<Self> {
            let supported = device
                .supported_input_configs()
                .map_err(|e| BatcherbirdError::Audio(e.to_string()))?
                .find(|c| {
                    c.channels() == 16
                        && c.sample_format() == SampleFormat::F32
                        && c.min_sample_rate().0 <= RATE
                        && c.max_sample_rate().0 >= RATE
                })
                .ok_or_else(|| {
                    BatcherbirdError::Audio("BlackHole needs 16-channel F32 input at 48kHz".into())
                })?
                .with_sample_rate(cpal::SampleRate(RATE));
            let (mut producer, consumer) = RingBuffer::new(RATE as usize * 8);
            let failed = Arc::new(AtomicBool::new(false));
            let overflow = Arc::clone(&failed);
            let stream_error = Arc::clone(&failed);
            let stream = device
                .build_input_stream(
                    &supported.config(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        for frame in data.as_chunks::<16>().0 {
                            if producer.slots() < 2 {
                                overflow.store(true, Ordering::Release);
                                break;
                            }
                            let _ = producer.push(frame[OFFSET]);
                            let _ = producer.push(frame[OFFSET + 1]);
                        }
                    },
                    move |_err| {
                        stream_error.store(true, Ordering::Release);
                    },
                    None,
                )
                .map_err(|e| BatcherbirdError::Audio(e.to_string()))?;
            stream
                .play()
                .map_err(|e| BatcherbirdError::Audio(e.to_string()))?;
            Ok(Self {
                _stream: stream,
                consumer,
                failed,
                samples: Vec::new(),
            })
        }
        fn drain(&mut self) {
            while let Ok(value) = self.consumer.pop() {
                self.samples.push(value);
            }
            assert!(
                !self.failed.load(Ordering::Acquire),
                "virtual input failed or its ring overflowed"
            );
        }
        fn collect_for(&mut self, duration: Duration) {
            let deadline = Instant::now() + duration;
            while Instant::now() < deadline {
                self.drain();
                std::thread::sleep(Duration::from_millis(2));
            }
            self.drain();
        }
        fn assert_idle(&mut self) {
            self.collect_for(Duration::from_millis(200));
            assert!(
                !self.samples.is_empty(),
                "BlackHole must deliver input callbacks"
            );
            assert!(
                self.samples.iter().all(|v| v.abs() < 0.000001),
                "Reserved BlackHole channels 3/4 are active: refusing to capture another source"
            );
            self.samples.clear();
        }
        fn wait_finished(&mut self, player: &PreviewPlayer) {
            let deadline = Instant::now() + Duration::from_secs(5);
            while !player.is_finished() && Instant::now() < deadline {
                self.drain();
                std::thread::sleep(Duration::from_millis(2));
            }
            self.drain();
            assert!(
                !player.has_output_error(),
                "actual preview output callback failed"
            );
            assert!(
                player.is_finished(),
                "preview failed to reach its drain deadline"
            );
            assert_eq!(player.playback_position(), 1.0);
        }
    }

    fn source(rate: u32, channels: usize, frequency: f32, seconds: f32) -> Vec<f32> {
        let frames = (seconds * rate as f32).round() as usize;
        (0..frames)
            .flat_map(|frame| {
                let time = frame as f32 / rate as f32;
                let amplitude = if time < 0.1 {
                    0.003
                } else if time < 0.25 {
                    0.15
                } else {
                    0.15 * (-4.0 * (time - 0.25) / (seconds - 0.275)).exp()
                };
                let left = if time >= seconds - 0.025 {
                    0.08 * (std::f32::consts::TAU * 880.0 * time).sin()
                } else {
                    amplitude * (std::f32::consts::TAU * frequency * time).sin()
                };
                if channels == 1 {
                    vec![left]
                } else {
                    vec![left, -0.35 * left]
                }
            })
            .collect()
    }
    fn configured_output(device: &cpal::Device) -> cpal::SupportedStreamConfig {
        device
            .supported_output_configs()
            .unwrap()
            .find(|c| {
                c.channels() == 16
                    && c.sample_format() == SampleFormat::F32
                    && c.min_sample_rate().0 <= RATE
                    && c.max_sample_rate().0 >= RATE
            })
            .expect("BlackHole needs 16-channel F32 output at 48kHz")
            .with_sample_rate(cpal::SampleRate(RATE))
    }
    fn player(device: &cpal::Device, audio: &[f32], rate: u32, channels: u16) -> PreviewPlayer {
        // Test-only fixture routing. Production playback has no acceptance-only
        // channel branch: mono/stereo is mapped then padded to sixteen channels.
        let mut routed = vec![0.0; audio.len() / channels as usize * 16];
        for (source, destination) in audio
            .chunks_exact(channels as usize)
            .zip(routed.as_chunks_mut::<16>().0.iter_mut())
        {
            let mut stereo = [0.0; 2];
            map_frame(source, channels as usize, 2, &mut stereo);
            destination[OFFSET..OFFSET + 2].copy_from_slice(&stereo);
        }
        PreviewPlayer::play_configured(
            Arc::from(routed),
            rate,
            16,
            device,
            configured_output(device),
        )
        .unwrap()
    }
    fn active_bounds(audio: &[f32]) -> (usize, usize) {
        let active = |frame: &[f32; 2]| frame.iter().any(|v| v.abs() > 0.00001);
        let first = audio
            .as_chunks::<2>()
            .0
            .iter()
            .position(active)
            .expect("generated signal was not captured");
        let last = audio.as_chunks::<2>().0.iter().rposition(active).unwrap();
        (first, last)
    }
    fn max_error(actual: &[f32], expected: &[f32]) -> f32 {
        actual
            .iter()
            .zip(expected)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f32::max)
    }
    fn correlation(actual: &[f32], expected: &[f32]) -> f64 {
        let (mut dot, mut a2, mut b2) = (0.0_f64, 0.0_f64, 0.0_f64);
        for (&a, &b) in actual.iter().zip(expected) {
            dot += a as f64 * b as f64;
            a2 += (a as f64).powi(2);
            b2 += (b as f64).powi(2);
        }
        dot / (a2 * b2).sqrt()
    }
    fn pitch(audio: &[f32]) -> f64 {
        let crossings: Vec<usize> = audio
            .as_chunks::<2>()
            .0
            .iter()
            .map(|f| f[0])
            .collect::<Vec<_>>()
            .windows(2)
            .enumerate()
            .filter(|(_, f)| f[0] <= 0.0 && f[1] > 0.0)
            .map(|(i, _)| i)
            .collect();
        assert!(crossings.len() > 10);
        RATE as f64 * (crossings.len() - 1) as f64
            / (crossings.last().unwrap() - crossings[0]) as f64
    }
    fn write_wav(path: &Path, audio: &[f32]) {
        let mut writer = hound::WavWriter::create(
            path,
            hound::WavSpec {
                channels: 2,
                sample_rate: RATE,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            },
        )
        .unwrap();
        for &value in audio {
            writer.write_sample(value).unwrap();
        }
        writer.finalize().unwrap();
    }
    struct Measurement {
        name: &'static str,
        frames: usize,
        error: f32,
        correlation: f64,
        frequency: f64,
    }
    impl Measurement {
        fn json(&self) -> String {
            format!("{{\"case\":\"{}\",\"pass\":true,\"signal_frames\":{},\"max_abs_error\":{},\"correlation\":{},\"measured_hz\":{}}}",
                self.name, self.frames, self.error, self.correlation, self.frequency)
        }
    }
    fn full_case(
        name: &'static str,
        input: &cpal::Device,
        output: &cpal::Device,
        rate: u32,
        channels: u16,
        directory: &Path,
    ) -> Measurement {
        let audio = source(rate, channels as usize, 440.0, 1.0);
        let expected = prepare_preview_audio(&audio, channels as usize, rate, 2, RATE).unwrap();
        let mut capture = Capture::start(input).unwrap();
        capture.assert_idle();
        let playback = player(output, &audio, rate, channels);
        capture.wait_finished(&playback);
        drop(playback); // Same cleanup as the GUI's completion Tick.
        capture.collect_for(Duration::from_millis(120));
        let (first, last) = active_bounds(&capture.samples);
        let (expected_first, expected_last) = active_bounds(&expected);
        let offset = first
            .checked_sub(expected_first)
            .expect("loopback starts before generated audio");
        assert_eq!(
            last - first,
            expected_last - expected_first,
            "{name}: tail must play before stream cleanup"
        );
        let aligned = &capture.samples[offset * 2..offset * 2 + expected.len()];
        let error = max_error(aligned, &expected);
        let similarity = correlation(aligned, &expected);
        assert!(
            error < 0.00001 && similarity > 0.99999,
            "{name}: stereo/sample mismatch error={error} correlation={similarity}"
        );
        let quiet = aligned[..RATE as usize / 20 * 2]
            .iter()
            .map(|v| v.abs())
            .fold(0.0, f32::max);
        assert!(
            quiet > 0.0025 && quiet < 0.0031,
            "quiet attack must survive actual playback"
        );
        let tail = &aligned[aligned.len() - RATE as usize / 50 * 2..];
        assert!(
            tail.iter().any(|v| v.abs() > 0.07),
            "terminal marker must survive completion/drop"
        );
        let frequency =
            pitch(&aligned[(RATE as usize * 12 / 100) * 2..(RATE as usize * 23 / 100) * 2]);
        assert!(
            (frequency - 440.0).abs() < 2.0,
            "{name}: pitch must stay440Hz, measured{frequency}"
        );
        write_wav(
            &directory.join(format!("{name}_capture.wav")),
            &capture.samples,
        );
        write_wav(&directory.join(format!("{name}_expected.wav")), &expected);
        Measurement {
            name,
            frames: last - first + 1,
            error,
            correlation: similarity,
            frequency,
        }
    }

    #[test]
    #[ignore = "requires explicit isolated BlackHole 16ch virtual loopback; never run on default/microphone devices"]
    fn blackhole_preview_driver_acceptance() {
        let directory = std::env::var_os("BATCHERBIRD_PLAYBACK_ACCEPTANCE_DIR")
            .expect("Set BATCHERBIRD_PLAYBACK_ACCEPTANCE_DIR to an artifact directory before explicitly running this ignored test");
        let directory = Path::new(&directory);
        fs::create_dir_all(directory).unwrap();
        let manager = AudioManager::new().unwrap();
        let input = manager.find_input_device(Some(DEVICE)).unwrap();
        let output = manager.find_output_device(Some(DEVICE)).unwrap();
        assert_eq!(input.name().unwrap(), DEVICE);
        assert_eq!(output.name().unwrap(), DEVICE);
        // This is not a listener test. No default-device, microphone or speaker stream is opened.
        let mut measurements = Vec::new();
        for name in ["stereo48_first", "stereo48_repeat", "stereo48_third"] {
            measurements.push(full_case(name, &input, &output, RATE, 2, directory));
        }
        measurements.push(full_case("mono48", &input, &output, RATE, 1, directory));
        measurements.push(full_case(
            "stereo441_resampled48",
            &input,
            &output,
            44_100,
            2,
            directory,
        ));

        let mut stop_capture = Capture::start(&input).unwrap();
        stop_capture.assert_idle();
        let audio = source(RATE, 2, 440.0, 2.0);
        let stopped = player(&output, &audio, RATE, 2);
        stop_capture.collect_for(Duration::from_millis(150));
        stopped.stop();
        assert!(
            stopped.is_finished(),
            "explicit Stop bypasses natural drain"
        );
        assert!(!stopped.has_output_error());
        drop(stopped);
        stop_capture.collect_for(Duration::from_millis(250));
        let (first, last) = active_bounds(&stop_capture.samples);
        assert!(
            last - first < RATE as usize * 35 / 100,
            "Stop must not continue the two-second take"
        );
        assert!(
            stop_capture.samples[stop_capture.samples.len() - RATE as usize / 20 * 2..]
                .iter()
                .all(|v| v.abs() < 0.000001),
            "Stop must leave silent output"
        );
        write_wav(&directory.join("stop_capture.wav"), &stop_capture.samples);
        let (expected_first, _) = active_bounds(&audio);
        let actual_prefix = &stop_capture.samples[first * 2..(last + 1) * 2];
        let expected_prefix = &audio[expected_first * 2..expected_first * 2 + actual_prefix.len()];
        let stop_error = max_error(actual_prefix, expected_prefix);
        let stop_similarity = correlation(actual_prefix, expected_prefix);
        assert!(
            stop_error < 0.00001 && stop_similarity > 0.99999,
            "stopped prefix must match the generated take"
        );
        measurements.push(Measurement {
            name: "stop",
            frames: last - first + 1,
            error: stop_error,
            correlation: stop_similarity,
            frequency: pitch(actual_prefix),
        });
        drop(stop_capture);

        let mut switch_capture = Capture::start(&input).unwrap();
        switch_capture.assert_idle();
        let old_audio = source(RATE, 2, 330.0, 2.0);
        let old = player(&output, &old_audio, RATE, 2);
        switch_capture.collect_for(Duration::from_millis(150));
        old.stop();
        drop(old);
        let new_audio = source(RATE, 2, 660.0, 1.0);
        let new = player(&output, &new_audio, RATE, 2);
        switch_capture.wait_finished(&new);
        drop(new);
        switch_capture.collect_for(Duration::from_millis(120));
        let expected = prepare_preview_audio(&new_audio, 2, RATE, 2, RATE).unwrap();
        let (_, last) = active_bounds(&switch_capture.samples);
        let (_, expected_last) = active_bounds(&expected);
        let offset = last - expected_last;
        let aligned = &switch_capture.samples[offset * 2..offset * 2 + expected.len()];
        // Allow one startup buffer, then prove no previous-note bleed remains.
        let ignore = RATE as usize / 20 * 2;
        let error = max_error(&aligned[ignore..], &expected[ignore..]);
        let similarity = correlation(&aligned[ignore..], &expected[ignore..]);
        assert!(
            error < 0.00001 && similarity > 0.99999,
            "switch left stale audio error={error} correlation={similarity}"
        );
        let frequency =
            pitch(&aligned[(RATE as usize * 12 / 100) * 2..(RATE as usize * 23 / 100) * 2]);
        assert!((frequency - 660.0).abs() < 2.0);
        write_wav(
            &directory.join("switch_capture.wav"),
            &switch_capture.samples,
        );
        write_wav(&directory.join("switch_expected.wav"), &expected);
        measurements.push(Measurement {
            name: "switch",
            frames: expected_last + 1,
            error,
            correlation: similarity,
            frequency,
        });
        let cases = measurements
            .iter()
            .map(Measurement::json)
            .collect::<Vec<_>>()
            .join(",\n");
        fs::write(directory.join("report.json"), format!("{{\"device\":\"BlackHole 16ch\",\"output_rate\":48000,\"channels\":[3,4],\"listener_verified\":false,\"cases\":[{cases}]}}\n")).unwrap();
        println!("PASS: {} isolated virtual-driver cases. Artifacts: {}. Listener acceptance is pending.", measurements.len(), directory.display());
    }
}
