// The CLI runs under #[tokio::main] and calls the synchronous *_blocking API.
// A virtual CoreMIDI port stands in for hardware, so this runs on macOS CI.
#![cfg(target_os = "macos")]

use batcherbird_core::sampler::{SamplingConfig, SamplingEngine};
use midir::os::unix::VirtualOutput;
use std::sync::atomic::AtomicBool;

#[tokio::test(flavor = "multi_thread")]
async fn blocking_sampling_api_does_not_panic_inside_tokio_main() {
    let output = midir::MidiOutput::new("batcherbird-test").unwrap();
    let mut conn = output.create_virtual("batcherbird-test-port").unwrap();
    let engine = SamplingEngine::new(SamplingConfig::default()).unwrap();
    // Already cancelled, so the call returns before any audio capture starts.
    let cancel = AtomicBool::new(true);

    let result = engine.sample_note_velocity_with_cancel_blocking(&mut conn, 60, 100, &cancel);

    assert!(matches!(result, Ok(None)), "unexpected result: {result:?}");
}
