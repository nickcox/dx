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
fn outputs_single_absolute_path_on_success() {
    let cwd = make_temp_dir("cli-success");
    let child = cwd.join("src");
    fs::create_dir_all(&child).expect("create child");

    let output = Command::new(dx_bin())
        .arg("resolve")
        .arg("src")
        .current_dir(&cwd)
        .output()
        .expect("run dx");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        canonical_string(&child)
    );
    assert!(String::from_utf8_lossy(&output.stderr).trim().is_empty());
    let _ = fs::remove_dir_all(cwd);
}

#[test]
fn returns_non_zero_with_empty_stdout_on_not_found() {
    let cwd = make_temp_dir("cli-not-found");

    let output = Command::new(dx_bin())
        .arg("resolve")
        .arg("missing")
        .current_dir(&cwd)
        .output()
        .expect("run dx");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unable to resolve query"));
    let _ = fs::remove_dir_all(cwd);
}

#[test]
fn list_mode_returns_candidates_for_ambiguity() {
    let cwd = make_temp_dir("cli-list");
    let root = cwd.join("root");
    fs::create_dir_all(root.join("proj/alpha")).expect("create proj alpha");
    fs::create_dir_all(root.join("prod/alpha")).expect("create prod alpha");

    let output = Command::new(dx_bin())
        .arg("resolve")
        .arg("--list")
        .arg("pro/al")
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .current_dir(&cwd)
        .output()
        .expect("run dx");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("proj/alpha"));
    assert!(stdout.contains("prod/alpha"));
    let _ = fs::remove_dir_all(cwd);
}

#[test]
fn json_mode_returns_structured_output() {
    let cwd = make_temp_dir("cli-json");
    let child = cwd.join("repo");
    fs::create_dir_all(&child).expect("create child");

    let output = Command::new(dx_bin())
        .arg("resolve")
        .arg("--json")
        .arg("repo")
        .current_dir(&cwd)
        .output()
        .expect("run dx");

    assert!(output.status.success());
    let json = serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("parse json");
    assert_eq!(json["status"], "ok");
    assert_eq!(json["path"], canonical_string(&child));
    let _ = fs::remove_dir_all(cwd);
}
