mod common;

use std::fs;

#[test]
fn outputs_single_absolute_path_on_success() {
    let cwd = common::temp_dir("cli-success");
    let child = cwd.path().join("src");
    fs::create_dir_all(&child).expect("create child");

    let output = common::dx()
        .arg("resolve")
        .arg("src")
        .current_dir(cwd.path())
        .output()
        .expect("run dx");

    assert!(output.status.success());
    common::assert_same_path(String::from_utf8_lossy(&output.stdout).trim(), &child);
    assert!(String::from_utf8_lossy(&output.stderr).trim().is_empty());
}

#[test]
fn returns_non_zero_with_empty_stdout_on_not_found() {
    let cwd = common::temp_dir("cli-not-found");

    let output = common::dx()
        .arg("resolve")
        .arg("missing")
        .current_dir(cwd.path())
        .output()
        .expect("run dx");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unable to resolve query"));
}

#[test]
fn list_mode_returns_candidates_for_ambiguity() {
    let cwd = common::temp_dir("cli-list");
    let root = cwd.path().join("root");
    fs::create_dir_all(root.join("proj/alpha")).expect("create proj alpha");
    fs::create_dir_all(root.join("prod/alpha")).expect("create prod alpha");

    let output = common::dx()
        .arg("resolve")
        .arg("--list")
        .arg("pro/al")
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .current_dir(cwd.path())
        .output()
        .expect("run dx");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("proj/alpha"));
    assert!(stdout.contains("prod/alpha"));
}

#[test]
fn json_mode_returns_structured_output() {
    let cwd = common::temp_dir("cli-json");
    let child = cwd.path().join("repo");
    fs::create_dir_all(&child).expect("create child");

    let output = common::dx()
        .arg("resolve")
        .arg("--json")
        .arg("repo")
        .current_dir(cwd.path())
        .output()
        .expect("run dx");

    assert!(output.status.success());
    let json = serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("parse json");
    assert_eq!(json["status"], "ok");
    common::assert_same_path(json["path"].as_str().expect("path string"), &child);
}

#[test]
fn resolve_uses_cwd_as_implicit_root_when_unset() {
    let cwd = common::temp_dir("cli-implicit-cwd-root");
    let target = cwd.path().join("workspace/project/src");
    fs::create_dir_all(&target).expect("create target");

    let output = common::dx()
        .arg("resolve")
        .arg("wo/pr/sr")
        .env_remove("DX_SEARCH_ROOTS")
        .current_dir(cwd.path())
        .output()
        .expect("run dx implicit cwd root");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    common::assert_same_path(String::from_utf8_lossy(&output.stdout).trim(), &target);
}

#[test]
fn resolves_delimiter_aware_segment_query() {
    let cwd = common::temp_dir("cli-delimiter-aware");
    let root = cwd.path().join("root");
    let target = root.join("cd-extras");
    fs::create_dir_all(&target).expect("create target");

    let output = common::dx()
        .arg("resolve")
        .arg("cd-e")
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .current_dir(cwd.path())
        .output()
        .expect("run dx delimiter-aware resolve");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    common::assert_same_path(String::from_utf8_lossy(&output.stdout).trim(), &target);
}

#[test]
fn resolves_doubled_period_segment_query() {
    let cwd = common::temp_dir("cli-gap-aware");
    let root = cwd.path().join("root");
    let target = root.join("PowerShell");
    fs::create_dir_all(&target).expect("create target");

    let output = common::dx()
        .arg("resolve")
        .arg("p..shell")
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .env("DX_CASE_SENSITIVE", "false")
        .current_dir(cwd.path())
        .output()
        .expect("run dx gap-aware resolve");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    common::assert_same_path(String::from_utf8_lossy(&output.stdout).trim(), &target);
}
