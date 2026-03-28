use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use dx::stacks::{storage, SessionStack};
use dx::test_support;

fn make_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "dx-it-navigate-{label}-{nonce}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

fn dx_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_dx"))
}

fn canonical(path: &PathBuf) -> PathBuf {
    fs::canonicalize(path).expect("canonical path")
}

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    test_support::env_lock()
}

#[test]
fn navigate_up_without_selector_returns_first_ancestor() {
    let temp = make_temp_dir("up-default");
    let cwd = temp.join("a/b/c");
    fs::create_dir_all(&cwd).expect("create cwd");

    let output = Command::new(dx_bin())
        .args(["navigate", "up"])
        .current_dir(&cwd)
        .output()
        .expect("run navigate up");

    assert!(output.status.success());
    let expected = canonical(&cwd.parent().expect("parent").to_path_buf());
    let actual = fs::canonicalize(String::from_utf8_lossy(&output.stdout).trim())
        .expect("canonical output path");
    assert_eq!(actual, expected);

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn navigate_up_numeric_selector_returns_nth_ancestor() {
    let temp = make_temp_dir("up-numeric");
    let cwd = temp.join("a/b/c");
    fs::create_dir_all(&cwd).expect("create cwd");

    let output = Command::new(dx_bin())
        .args(["navigate", "up", "2"])
        .current_dir(&cwd)
        .output()
        .expect("run navigate up 2");

    assert!(output.status.success());
    let expected = canonical(
        &cwd.parent()
            .and_then(|value| value.parent())
            .expect("second parent")
            .to_path_buf(),
    );
    let actual = fs::canonicalize(String::from_utf8_lossy(&output.stdout).trim())
        .expect("canonical output path");
    assert_eq!(actual, expected);

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn navigate_up_path_selector_uses_best_match() {
    let temp = make_temp_dir("up-path");
    let cwd = temp.join("code/projects/dx");
    fs::create_dir_all(&cwd).expect("create cwd");

    let output = Command::new(dx_bin())
        .args(["navigate", "up", "code"])
        .current_dir(&cwd)
        .output()
        .expect("run navigate up code");

    assert!(output.status.success());
    let expected = canonical(&temp.join("code"));
    let actual = fs::canonicalize(String::from_utf8_lossy(&output.stdout).trim())
        .expect("canonical output path");
    assert_eq!(actual, expected);

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn navigate_back_and_forward_use_stack_entries() {
    let _guard = env_lock();
    let temp = make_temp_dir("back-forward");
    let runtime = temp.join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");
    env::set_var("XDG_RUNTIME_DIR", runtime.display().to_string());

    let dir = storage::ensure_session_dir().expect("session dir");
    let state = SessionStack {
        cwd: Some(PathBuf::from("/now")),
        undo: vec![PathBuf::from("/a"), PathBuf::from("/b")],
        redo: vec![PathBuf::from("/x"), PathBuf::from("/y")],
    };
    storage::write_session(&dir, "s1", &state).expect("write session");

    let back = Command::new(dx_bin())
        .args(["navigate", "back", "--session", "s1"])
        .current_dir(&temp)
        .output()
        .expect("run navigate back");
    assert!(back.status.success());
    assert_eq!(String::from_utf8_lossy(&back.stdout).trim(), "/b");

    let forward = Command::new(dx_bin())
        .args(["navigate", "forward", "--session", "s1"])
        .current_dir(&temp)
        .output()
        .expect("run navigate forward");
    assert!(forward.status.success());
    assert_eq!(String::from_utf8_lossy(&forward.stdout).trim(), "/y");

    env::remove_var("XDG_RUNTIME_DIR");
    let _ = fs::remove_dir_all(temp);
}

#[test]
fn navigate_fails_for_out_of_range_and_no_match() {
    let _guard = env_lock();
    let temp = make_temp_dir("errors");
    let runtime = temp.join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");
    env::set_var("XDG_RUNTIME_DIR", runtime.display().to_string());

    let dir = storage::ensure_session_dir().expect("session dir");
    let state = SessionStack {
        cwd: Some(PathBuf::from("/now")),
        undo: vec![PathBuf::from("/a")],
        redo: vec![],
    };
    storage::write_session(&dir, "s1", &state).expect("write session");

    let out_of_range = Command::new(dx_bin())
        .args(["navigate", "back", "2", "--session", "s1"])
        .current_dir(&temp)
        .output()
        .expect("run out of range");
    assert!(!out_of_range.status.success());
    assert!(String::from_utf8_lossy(&out_of_range.stderr).contains("out of range"));

    let no_match = Command::new(dx_bin())
        .args(["navigate", "back", "zzz", "--session", "s1"])
        .current_dir(&temp)
        .output()
        .expect("run no match");
    assert!(!no_match.status.success());
    assert!(String::from_utf8_lossy(&no_match.stderr).contains("did not match any candidate"));

    env::remove_var("XDG_RUNTIME_DIR");
    let _ = fs::remove_dir_all(temp);
}
