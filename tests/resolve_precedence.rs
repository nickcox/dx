mod common;

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn direct_child_beats_fallback_root_match() {
    let cwd = common::temp_dir("precedence-direct");
    let local = cwd.path().join("src");
    fs::create_dir_all(&local).expect("create local src");

    let root = cwd.path().join("root");
    let fallback = root.join("src");
    fs::create_dir_all(&fallback).expect("create fallback src");

    let output = common::dx()
        .arg("resolve")
        .arg("src")
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .current_dir(cwd.path())
        .output()
        .expect("run dx");

    assert!(output.status.success());
    common::assert_same_path(String::from_utf8_lossy(&output.stdout).trim(), &local);
}

#[test]
fn step_up_alias_wins_over_search_root_name() {
    let workspace = common::temp_dir("precedence-step-up");
    let cwd = workspace.path().join("a/b/c");
    fs::create_dir_all(&cwd).expect("create nested cwd");
    let root = workspace.path().join("root");
    fs::create_dir_all(root.join("...")).expect("create literal dots directory");

    let output = common::dx()
        .arg("resolve")
        .arg("...")
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .current_dir(&cwd)
        .output()
        .expect("run dx");

    assert!(output.status.success());
    common::assert_same_path(
        String::from_utf8_lossy(&output.stdout).trim(),
        workspace.path().join("a"),
    );
}

#[test]
fn ambiguous_default_mode_fails_with_stderr_diagnostic() {
    let cwd = common::temp_dir("precedence-ambiguous");
    let root = cwd.path().join("root");
    fs::create_dir_all(root.join("proj/alpha")).expect("create proj alpha");
    fs::create_dir_all(root.join("prod/alpha")).expect("create prod alpha");

    let output = common::dx()
        .arg("resolve")
        .arg("pro/al")
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .current_dir(cwd.path())
        .output()
        .expect("run dx");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("ambiguous query"));
}

#[test]
fn ambiguous_json_mode_returns_candidates_and_non_zero_exit() {
    let cwd = common::temp_dir("precedence-json-ambiguous");
    let root = cwd.path().join("root");
    fs::create_dir_all(root.join("proj/alpha")).expect("create proj alpha");
    fs::create_dir_all(root.join("prod/alpha")).expect("create prod alpha");

    let output = common::dx()
        .arg("resolve")
        .arg("--json")
        .arg("pro/al")
        .env("DX_SEARCH_ROOTS", root.display().to_string())
        .current_dir(cwd.path())
        .output()
        .expect("run dx");

    // `--json` changes presentation, not success: an ambiguous query did not
    // resolve to one directory, so the exit code says so while stderr stays
    // empty and the detail goes to stdout.
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let json = serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("parse json");
    assert_eq!(json["status"], "error");
    assert_eq!(json["reason"], "ambiguous");

    let candidates = json["candidates"].as_array().expect("candidate array");
    assert_eq!(candidates.len(), 2);
    let actual = candidates
        .iter()
        .map(|candidate| {
            common::canonical(Path::new(
                candidate.as_str().expect("candidate path string"),
            ))
        })
        .collect::<BTreeSet<_>>();
    let expected = [root.join("proj/alpha"), root.join("prod/alpha")]
        .iter()
        .map(|candidate| common::canonical(candidate))
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
}
