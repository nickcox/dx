use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use dx::stacks::SessionStack;

mod common;

fn dx_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_dx"))
}

fn write_session(runtime: &Path, session: &str, state: &SessionStack) {
    let dir = runtime.join("dx-sessions");
    fs::create_dir_all(&dir).expect("create session dir");
    fs::write(
        dir.join(format!("{session}.json")),
        serde_json::to_vec(state).expect("serialize session"),
    )
    .expect("write session");
}

#[test]
fn navigate_up_without_selector_returns_first_ancestor() {
    let temp = common::temp_dir("navigate-up-default");
    let cwd = temp.path().join("a/b/c");
    fs::create_dir_all(&cwd).expect("create cwd");

    let output = Command::new(dx_bin())
        .args(["navigate", "up"])
        .current_dir(&cwd)
        .output()
        .expect("run navigate up");

    assert!(output.status.success());
    common::assert_same_path(
        String::from_utf8_lossy(&output.stdout).trim(),
        cwd.parent().expect("parent"),
    );
}

#[test]
fn navigate_up_numeric_selector_returns_nth_ancestor() {
    let temp = common::temp_dir("navigate-up-numeric");
    let cwd = temp.path().join("a/b/c");
    fs::create_dir_all(&cwd).expect("create cwd");

    let output = Command::new(dx_bin())
        .args(["navigate", "up", "2"])
        .current_dir(&cwd)
        .output()
        .expect("run navigate up 2");

    assert!(output.status.success());
    common::assert_same_path(
        String::from_utf8_lossy(&output.stdout).trim(),
        cwd.parent()
            .and_then(|value| value.parent())
            .expect("second parent"),
    );
}

#[test]
fn navigate_up_path_selector_uses_best_match() {
    let temp = common::temp_dir("navigate-up-path");
    let cwd = temp.path().join("code/projects/dx");
    fs::create_dir_all(&cwd).expect("create cwd");

    let output = Command::new(dx_bin())
        .args(["navigate", "up", "code"])
        .current_dir(&cwd)
        .output()
        .expect("run navigate up code");

    assert!(output.status.success());
    common::assert_same_path(
        String::from_utf8_lossy(&output.stdout).trim(),
        temp.path().join("code"),
    );
}

#[test]
fn navigate_back_and_forward_use_stack_entries() {
    let temp = common::temp_dir("navigate-back-forward");
    let runtime = temp.path().join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let state = SessionStack {
        cwd: Some(PathBuf::from("/now")),
        undo: vec![PathBuf::from("/a"), PathBuf::from("/b")],
        redo: vec![PathBuf::from("/x"), PathBuf::from("/y")],
    };
    write_session(&runtime, "s1", &state);

    let back = Command::new(dx_bin())
        .args(["navigate", "back", "--session", "s1"])
        .env("XDG_RUNTIME_DIR", &runtime)
        .current_dir(temp.path())
        .output()
        .expect("run navigate back");
    assert!(back.status.success());
    assert_eq!(String::from_utf8_lossy(&back.stdout).trim(), "/b");

    let forward = Command::new(dx_bin())
        .args(["navigate", "forward", "--session", "s1"])
        .env("XDG_RUNTIME_DIR", &runtime)
        .current_dir(temp.path())
        .output()
        .expect("run navigate forward");
    assert!(forward.status.success());
    assert_eq!(String::from_utf8_lossy(&forward.stdout).trim(), "/y");
}

#[test]
fn navigate_fails_for_out_of_range_and_no_match() {
    let temp = common::temp_dir("navigate-errors");
    let runtime = temp.path().join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let state = SessionStack {
        cwd: Some(PathBuf::from("/now")),
        undo: vec![PathBuf::from("/a")],
        redo: vec![],
    };
    write_session(&runtime, "s1", &state);

    let out_of_range = Command::new(dx_bin())
        .args(["navigate", "back", "2", "--session", "s1"])
        .env("XDG_RUNTIME_DIR", &runtime)
        .current_dir(temp.path())
        .output()
        .expect("run out of range");
    assert!(!out_of_range.status.success());
    assert!(String::from_utf8_lossy(&out_of_range.stderr).contains("out of range"));

    let no_match = Command::new(dx_bin())
        .args(["navigate", "back", "zzz", "--session", "s1"])
        .env("XDG_RUNTIME_DIR", &runtime)
        .current_dir(temp.path())
        .output()
        .expect("run no match");
    assert!(!no_match.status.success());
    assert!(String::from_utf8_lossy(&no_match.stderr).contains("did not match any candidate"));
}
