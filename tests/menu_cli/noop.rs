//! Cases where `dx menu` declines to open and emits a noop action.

#[cfg(not(unix))]
use std::process::Command;

use super::common;
use super::support::*;

// --- 4.2 Non-interactive / noop behavior ---

#[test]
fn menu_without_tty_outputs_noop_json() {
    // When run non-interactively (no TTY), dx menu should output {"action":"noop"}
    // unless the single-candidate fast path applies.
    let cwd = common::temp_dir("without-tty-noop");
    let output = dx()
        .args(["menu", "--buffer", "cd foo", "--cursor", "6"])
        .current_dir(cwd.path())
        .output()
        .expect("dx menu should run");

    assert!(output.status.success(), "exit code should be 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed, serde_json::json!({ "action": "noop" }));
}

#[test]
fn menu_unrecognized_command_outputs_noop() {
    let output = dx()
        .args(["menu", "--buffer", "ls -la", "--cursor", "5"])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success(), "exit code should be 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed, serde_json::json!({ "action": "noop" }));
}

#[test]
fn menu_empty_buffer_outputs_noop() {
    let output = dx()
        .args(["menu", "--buffer", "", "--cursor", "0"])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success(), "exit code should be 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed, serde_json::json!({ "action": "noop" }));
}

// --- 4.2 Selection output contract ---

#[test]
fn menu_noop_json_has_only_action_field() {
    let output = dx()
        .args(["menu", "--buffer", "cd x", "--cursor", "4"])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success(), "exit code should be 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed, serde_json::json!({ "action": "noop" }));
}
