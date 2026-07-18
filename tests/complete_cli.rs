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
fn complete_ancestors_lists_nearest_first() {
    let temp = common::temp_dir("complete-ancestors");
    let cwd = temp.path().join("a/b/c");
    fs::create_dir_all(&cwd).expect("create nested");

    let output = Command::new(dx_bin())
        .args(["complete", "ancestors"])
        .current_dir(&cwd)
        .output()
        .expect("run complete ancestors");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout.lines().collect::<Vec<_>>();
    common::assert_same_path(lines[0], cwd.parent().expect("parent"));
    common::assert_same_path(
        lines
            .last()
            .expect("ancestor output includes filesystem root"),
        cwd.ancestors().last().expect("filesystem root"),
    );
}

#[test]
fn complete_ancestors_filter_returns_matching_entry() {
    let temp = common::temp_dir("complete-ancestors-filter");
    let cwd = temp.path().join("code/projects/dx");
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
    common::assert_same_path(lines[0], temp.path().join("code"));
    assert!(lines.iter().any(|line| {
        common::canonical(Path::new(line)) == common::canonical(&temp.path().join("code/projects"))
    }));
}

#[test]
fn complete_ancestors_at_root_returns_empty() {
    let root = PathBuf::from(std::path::MAIN_SEPARATOR.to_string());
    let output = Command::new(dx_bin())
        .args(["complete", "ancestors"])
        .current_dir(root)
        .output()
        .expect("run complete ancestors root");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
}

#[test]
fn complete_limit_and_list_alias_cap_results() {
    let temp = common::temp_dir("complete-ancestors-limit");
    let cwd = temp.path().join("a/b/c/d");
    fs::create_dir_all(&cwd).expect("create nested");

    let limited = Command::new(dx_bin())
        .args(["complete", "ancestors", "--limit", "1"])
        .current_dir(&cwd)
        .output()
        .expect("run complete ancestors --limit");
    assert!(limited.status.success());
    let limited_stdout = String::from_utf8_lossy(&limited.stdout);
    let limited_lines = limited_stdout.lines().collect::<Vec<_>>();
    assert_eq!(limited_lines.len(), 1);

    let alias = Command::new(dx_bin())
        .args(["complete", "ancestors", "--list", "1"])
        .current_dir(&cwd)
        .output()
        .expect("run complete ancestors --list");
    assert!(alias.status.success());
    let alias_stdout = String::from_utf8_lossy(&alias.stdout);
    let alias_lines = alias_stdout.lines().collect::<Vec<_>>();
    assert_eq!(alias_lines.len(), 1);
}

#[test]
fn complete_paths_returns_abbreviation_matches() {
    let temp = common::temp_dir("complete-paths");
    let root = temp.path().join("root");
    fs::create_dir_all(root.join("projects/alpha")).expect("create projects");
    fs::create_dir_all(root.join("presentations/alpha")).expect("create presentations");

    let output = Command::new(dx_bin())
        .args(["complete", "paths", "pr/al"])
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run complete paths");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let paths = stdout.lines().map(Path::new).collect::<Vec<_>>();
    assert!(
        paths
            .iter()
            .any(|path| common::canonical(path) == common::canonical(&root.join("projects/alpha")))
    );
    assert!(paths.iter().any(|path| {
        common::canonical(path) == common::canonical(&root.join("presentations/alpha"))
    }));
}

#[test]
fn complete_paths_no_match_returns_empty() {
    let temp = common::temp_dir("complete-paths-empty");
    let root = temp.path().join("root");
    fs::create_dir_all(&root).expect("create root");

    let output = Command::new(dx_bin())
        .args(["complete", "paths", "zzz"])
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run complete paths empty");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
}

#[test]
fn complete_recents_and_stack_use_session_state() {
    let temp = common::temp_dir("complete-recents-stack");
    let runtime = temp.path().join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");
    let now = temp.path().join("now");
    let a = temp.path().join("a");
    let b = temp.path().join("b");
    let x = temp.path().join("x");
    for path in [&now, &a, &b, &x] {
        fs::create_dir_all(path).expect("create session path");
    }

    let state = SessionStack {
        cwd: Some(now),
        undo: vec![a.clone(), b.clone()],
        redo: vec![x.clone()],
    };
    write_session(&runtime, "s1", &state);

    let recents = Command::new(dx_bin())
        .args(["complete", "recents", "--session", "s1"])
        .env("XDG_RUNTIME_DIR", &runtime)
        .current_dir(temp.path())
        .output()
        .expect("run recents");
    assert!(recents.status.success());
    let recents_lines = String::from_utf8_lossy(&recents.stdout)
        .lines()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert_eq!(
        recents_lines,
        vec![b.display().to_string(), a.display().to_string()]
    );

    let stack_back = Command::new(dx_bin())
        .args([
            "complete",
            "stack",
            "--direction",
            "back",
            "--session",
            "s1",
        ])
        .env("XDG_RUNTIME_DIR", &runtime)
        .current_dir(temp.path())
        .output()
        .expect("run stack back");
    assert!(stack_back.status.success());
    let back_lines = String::from_utf8_lossy(&stack_back.stdout)
        .lines()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert_eq!(
        back_lines,
        vec![b.display().to_string(), a.display().to_string()]
    );

    let stack_forward = Command::new(dx_bin())
        .args([
            "complete",
            "stack",
            "--direction",
            "forward",
            "--session",
            "s1",
        ])
        .env("XDG_RUNTIME_DIR", &runtime)
        .current_dir(temp.path())
        .output()
        .expect("run stack forward");
    assert!(stack_forward.status.success());
    let forward_lines = String::from_utf8_lossy(&stack_forward.stdout)
        .lines()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert_eq!(forward_lines, vec![x.display().to_string()]);
}

#[test]
fn complete_recents_missing_session_returns_empty_and_zero() {
    let temp = common::temp_dir("complete-recents-missing");

    let output = Command::new(dx_bin())
        .args(["complete", "recents"])
        .env_remove("DX_SESSION")
        .current_dir(temp.path())
        .output()
        .expect("run recents missing");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
}

#[test]
fn complete_frecents_without_zoxide_returns_empty() {
    let temp = common::temp_dir("complete-frecents-empty");
    let empty_path = common::temp_dir("complete-empty-path");

    let output = Command::new(dx_bin())
        .args(["complete", "frecents", "proj"])
        .env("PATH", empty_path.path())
        .current_dir(temp.path())
        .output()
        .expect("run frecents");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
}

#[test]
fn complete_json_output_has_path_label_rank() {
    let temp = common::temp_dir("complete-json");
    let cwd = temp.path().join("home/user/code");
    fs::create_dir_all(&cwd).expect("create cwd");

    let output = Command::new(dx_bin())
        .args(["complete", "ancestors", "--json"])
        .current_dir(&cwd)
        .output()
        .expect("run complete json");

    assert!(output.status.success());
    let json = serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("parse json");
    let entries = json.as_array().expect("JSON output is an array");
    assert!(!entries.is_empty());
    for (index, entry) in entries.iter().enumerate() {
        let object = entry.as_object().expect("completion is an object");
        let mut keys = object.keys().map(String::as_str).collect::<Vec<_>>();
        keys.sort_unstable();
        assert_eq!(keys, ["label", "path", "rank"]);
        assert!(object["path"].is_string());
        assert!(object["label"].is_string());
        assert_eq!(object["rank"].as_u64(), Some(index as u64 + 1));
    }
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

#[test]
fn complete_paths_uses_cwd_as_implicit_root_when_unset() {
    let temp = common::temp_dir("complete-paths-implicit-cwd-root");
    let cwd = temp.path().join("work");
    let target = cwd.join("workspace/project");
    fs::create_dir_all(&target).expect("create target");

    let output = Command::new(dx_bin())
        .args(["complete", "paths", "wo/pr"])
        .env_remove("DX_SEARCH_ROOTS")
        .current_dir(&cwd)
        .output()
        .expect("run complete paths implicit cwd root");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout
            .lines()
            .map(Path::new)
            .any(|path| common::canonical(path) == common::canonical(&target)),
        "expected implicit cwd-root abbreviation candidate, got: {stdout}"
    );
}

#[test]
fn complete_paths_returns_delimiter_aware_matches() {
    let temp = common::temp_dir("complete-paths-delimiter-aware");
    let root = temp.path().join("root");
    fs::create_dir_all(root.join("cd-extras")).expect("create cd-extras");
    fs::create_dir_all(root.join("cd-editor")).expect("create cd-editor");

    let output = Command::new(dx_bin())
        .args(["complete", "paths", "cd-e"])
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run complete paths delimiter-aware");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cd-extras"));
    assert!(stdout.contains("cd-editor"));
}

#[test]
fn complete_paths_returns_doubled_period_matches() {
    let temp = common::temp_dir("complete-paths-gap-aware");
    let root = temp.path().join("root");
    fs::create_dir_all(root.join("PowerShell")).expect("create powershell");

    let output = Command::new(dx_bin())
        .args(["complete", "paths", "p..shell"])
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .env("DX_CASE_SENSITIVE", "false")
        .current_dir(temp.path())
        .output()
        .expect("run complete paths gap-aware");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PowerShell"));
}
