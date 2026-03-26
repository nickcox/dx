use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn make_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("dx-it-{label}-{nonce}-{}", std::process::id()));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

fn dx_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_dx"))
}

fn canonical_string(path: &PathBuf) -> String {
    fs::canonicalize(path)
        .expect("canonical path")
        .display()
        .to_string()
}

#[test]
fn direct_child_beats_fallback_root_match() {
    let cwd = make_temp_dir("precedence-direct");
    let local = cwd.join("src");
    fs::create_dir_all(&local).expect("create local src");

    let root = cwd.join("root");
    let fallback = root.join("src");
    fs::create_dir_all(&fallback).expect("create fallback src");

    let output = Command::new(dx_bin())
        .arg("resolve")
        .arg("src")
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .current_dir(&cwd)
        .output()
        .expect("run dx");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        canonical_string(&local)
    );
    let _ = fs::remove_dir_all(cwd);
}

#[test]
fn step_up_alias_wins_over_search_root_name() {
    let workspace = make_temp_dir("precedence-step-up");
    let cwd = workspace.join("a/b/c");
    fs::create_dir_all(&cwd).expect("create nested cwd");
    let root = workspace.join("root");
    fs::create_dir_all(root.join("...")).expect("create literal dots directory");

    let output = Command::new(dx_bin())
        .arg("resolve")
        .arg("...")
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .current_dir(&cwd)
        .output()
        .expect("run dx");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        canonical_string(&workspace.join("a"))
    );
    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn ambiguous_default_mode_fails_with_stderr_diagnostic() {
    let cwd = make_temp_dir("precedence-ambiguous");
    let root = cwd.join("root");
    fs::create_dir_all(root.join("proj/alpha")).expect("create proj alpha");
    fs::create_dir_all(root.join("prod/alpha")).expect("create prod alpha");

    let output = Command::new(dx_bin())
        .arg("resolve")
        .arg("pro/al")
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .current_dir(&cwd)
        .output()
        .expect("run dx");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("ambiguous query"));
    let _ = fs::remove_dir_all(cwd);
}

#[test]
fn ambiguous_json_mode_returns_candidates_and_zero_exit() {
    let cwd = make_temp_dir("precedence-json-ambiguous");
    let root = cwd.join("root");
    fs::create_dir_all(root.join("proj/alpha")).expect("create proj alpha");
    fs::create_dir_all(root.join("prod/alpha")).expect("create prod alpha");

    let output = Command::new(dx_bin())
        .arg("resolve")
        .arg("--json")
        .arg("pro/al")
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .current_dir(&cwd)
        .output()
        .expect("run dx");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"status\":\"error\""));
    assert!(stdout.contains("\"reason\":\"ambiguous\""));
    assert!(stdout.contains("proj/alpha"));
    assert!(stdout.contains("prod/alpha"));
    let _ = fs::remove_dir_all(cwd);
}
