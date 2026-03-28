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
        "dx-it-complete-{label}-{nonce}-{}",
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
fn complete_ancestors_lists_nearest_first() {
    let temp = make_temp_dir("ancestors");
    let cwd = temp.join("a/b/c");
    fs::create_dir_all(&cwd).expect("create nested");

    let output = Command::new(dx_bin())
        .args(["complete", "ancestors"])
        .current_dir(&cwd)
        .output()
        .expect("run complete ancestors");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout.lines().collect::<Vec<_>>();
    let expected = canonical(&cwd.parent().expect("parent").to_path_buf());
    let actual = fs::canonicalize(lines[0]).expect("canonical first ancestor");
    assert_eq!(actual, expected);
    assert!(lines.contains(&"/"));

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn complete_ancestors_filter_returns_matching_entry() {
    let temp = make_temp_dir("ancestors-filter");
    let cwd = temp.join("code/projects/dx");
    fs::create_dir_all(&cwd).expect("create nested");

    let output = Command::new(dx_bin())
        .args(["complete", "ancestors", "code"])
        .current_dir(&cwd)
        .output()
        .expect("run complete ancestors filter");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout.lines().collect::<Vec<_>>();
    assert!(!lines.is_empty());
    assert!(lines[0].ends_with("/code"));
    assert!(lines.iter().any(|line| line.ends_with("/code/projects")));

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn complete_ancestors_at_root_returns_empty() {
    let output = Command::new(dx_bin())
        .args(["complete", "ancestors"])
        .current_dir("/")
        .output()
        .expect("run complete ancestors root");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
}

#[test]
fn complete_paths_returns_abbreviation_matches() {
    let temp = make_temp_dir("paths");
    let root = temp.join("root");
    fs::create_dir_all(root.join("projects/alpha")).expect("create projects");
    fs::create_dir_all(root.join("presentations/alpha")).expect("create presentations");

    let output = Command::new(dx_bin())
        .args(["complete", "paths", "pr/al"])
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .current_dir(&temp)
        .output()
        .expect("run complete paths");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("projects/alpha"));
    assert!(stdout.contains("presentations/alpha"));

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn complete_paths_no_match_returns_empty() {
    let temp = make_temp_dir("paths-empty");
    let root = temp.join("root");
    fs::create_dir_all(&root).expect("create root");

    let output = Command::new(dx_bin())
        .args(["complete", "paths", "zzz"])
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .current_dir(&temp)
        .output()
        .expect("run complete paths empty");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn complete_recents_and_stack_use_session_state() {
    let _guard = env_lock();
    let temp = make_temp_dir("recents-stack");
    let runtime = temp.join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");
    env::set_var("XDG_RUNTIME_DIR", runtime.display().to_string());

    let dir = storage::ensure_session_dir().expect("session dir");
    let state = SessionStack {
        cwd: Some(PathBuf::from("/now")),
        undo: vec![PathBuf::from("/a"), PathBuf::from("/b")],
        redo: vec![PathBuf::from("/x")],
    };
    storage::write_session(&dir, "s1", &state).expect("write session");

    let recents = Command::new(dx_bin())
        .args(["complete", "recents", "--session", "s1"])
        .current_dir(&temp)
        .output()
        .expect("run recents");
    assert!(recents.status.success());
    let recents_lines = String::from_utf8_lossy(&recents.stdout)
        .lines()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert_eq!(recents_lines, vec!["/b".to_string(), "/a".to_string()]);

    let stack_back = Command::new(dx_bin())
        .args([
            "complete",
            "stack",
            "--direction",
            "back",
            "--session",
            "s1",
        ])
        .current_dir(&temp)
        .output()
        .expect("run stack back");
    assert!(stack_back.status.success());
    let back_lines = String::from_utf8_lossy(&stack_back.stdout)
        .lines()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert_eq!(back_lines, vec!["/b".to_string(), "/a".to_string()]);

    let stack_forward = Command::new(dx_bin())
        .args([
            "complete",
            "stack",
            "--direction",
            "forward",
            "--session",
            "s1",
        ])
        .current_dir(&temp)
        .output()
        .expect("run stack forward");
    assert!(stack_forward.status.success());
    let forward_lines = String::from_utf8_lossy(&stack_forward.stdout)
        .lines()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert_eq!(forward_lines, vec!["/x".to_string()]);

    env::remove_var("XDG_RUNTIME_DIR");
    let _ = fs::remove_dir_all(temp);
}

#[test]
fn complete_recents_missing_session_returns_empty_and_zero() {
    let temp = make_temp_dir("recents-missing");

    let output = Command::new(dx_bin())
        .args(["complete", "recents"])
        .env_remove("DX_SESSION")
        .current_dir(&temp)
        .output()
        .expect("run recents missing");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn complete_frecents_without_zoxide_returns_empty() {
    let temp = make_temp_dir("frecents-empty");

    let output = Command::new(dx_bin())
        .args(["complete", "frecents", "proj"])
        .env("PATH", "/usr/bin:/bin")
        .current_dir(&temp)
        .output()
        .expect("run frecents");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn complete_json_output_has_path_label_rank() {
    let temp = make_temp_dir("json");
    let cwd = temp.join("home/user/code");
    fs::create_dir_all(&cwd).expect("create cwd");

    let output = Command::new(dx_bin())
        .args(["complete", "ancestors", "--json"])
        .current_dir(&cwd)
        .output()
        .expect("run complete json");

    assert!(output.status.success());
    let json = serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("parse json");
    assert!(json.is_array());
    let first = &json[0];
    assert!(first.get("path").is_some());
    assert!(first.get("label").is_some());
    assert!(first.get("rank").is_some());

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn complete_error_cases_return_non_zero() {
    let missing_mode = Command::new(dx_bin())
        .args(["complete"])
        .output()
        .expect("run complete missing mode");
    assert!(!missing_mode.status.success());

    let invalid_mode = Command::new(dx_bin())
        .args(["complete", "bogus"])
        .output()
        .expect("run complete invalid mode");
    assert!(!invalid_mode.status.success());

    let stack_missing_direction = Command::new(dx_bin())
        .args(["complete", "stack"])
        .output()
        .expect("run complete stack missing direction");
    assert!(!stack_missing_direction.status.success());
}
