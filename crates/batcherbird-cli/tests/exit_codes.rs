// Scripts and shells rely on the exit status to know a command failed.
// These inputs are rejected before any MIDI or audio device is opened.

use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_batcherbird"))
        .args(args)
        .output()
        .expect("batcherbird binary runs")
}

#[test]
fn out_of_range_note_fails() {
    for args in [
        &["sample-note", "--note", "200"][..],
        &["sample-export", "--note", "128"][..],
        &["sample-range", "--start", "0", "--end", "130"][..],
    ] {
        let output = run(args);
        assert!(!output.status.success(), "{args:?} exited successfully");
    }
}

#[test]
fn inverted_range_fails() {
    let output = run(&["sample-range", "--start", "72", "--end", "60"]);
    assert!(
        !output.status.success(),
        "inverted range exited successfully"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("72"),
        "error does not name the range: {stderr}"
    );
}
