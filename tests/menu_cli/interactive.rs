//! The replacement `dx menu` produces for a given buffer, cursor and mode.

use std::fs;
#[cfg(not(unix))]
use std::process::Command;

use super::common;
use super::support::*;

// --- 5.3 Completion-context interactivity contracts ---
// These verify the structural contracts behind interactive behaviour. Driving the
// menu through a real pty is possible — answer the cursor-position query
// (`ESC [ 6 n`) and set a window size, or it refuses to draw — but is not done here.

#[test]
fn menu_with_valid_dx_command_without_tty_returns_noop() {
    // In a non-TTY context (CI/piped), dx menu for a valid command
    // should return noop when the single-candidate fast path does not apply.
    // This proves the TTY gate is effective — without TTY the menu
    // does not attempt to open, and falls back cleanly.
    let cwd = common::temp_dir("valid-without-tty-noop");
    let miss = format!(
        "__dx_no_candidate_{}",
        cwd.path()
            .file_name()
            .expect("temp cwd should have a file name")
            .to_string_lossy()
    );
    let session = format!("test-{miss}");
    let commands = [
        format!("cd {miss}"),
        format!("up {miss}"),
        format!("cdf {miss}"),
        format!("z {miss}"),
        format!("cdr {miss}"),
        format!("back {miss}"),
        format!("forward {miss}"),
        format!("cd- {miss}"),
        format!("cd+ {miss}"),
    ];
    for cmd in commands {
        let cursor = cmd.len().to_string();
        let output = dx()
            .args([
                "menu",
                "--buffer",
                &cmd,
                "--cursor",
                &cursor,
                "--session",
                &session,
            ])
            .current_dir(cwd.path())
            .output()
            .unwrap_or_else(|_| panic!("dx menu should run for buffer '{}'", cmd));

        assert!(output.status.success(), "should succeed for '{}'", cmd);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
            .unwrap_or_else(|_| panic!("should be valid JSON for '{}': {stdout}", cmd));
        assert_eq!(
            parsed,
            serde_json::json!({ "action": "noop" }),
            "non-TTY context should produce noop for '{}'",
            cmd
        );
    }
}

#[test]
fn menu_stderr_is_silent_on_noop() {
    // When menu falls back to noop, stderr should be empty (no diagnostic noise).
    let output = dx()
        .args(["menu", "--buffer", "cd foo", "--cursor", "6"])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success(), "exit code should be 0");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.is_empty(),
        "stderr should be silent on noop, got: {stderr}"
    );
}

#[cfg(unix)]
#[test]
fn menu_paths_mode_honors_explicit_cwd() {
    let process_cwd = common::temp_dir("process-cwd-empty");
    let explicit_cwd = common::temp_dir("explicit-cwd-with-child");
    let child_a = explicit_cwd.path().join("alpha");
    let child_b = explicit_cwd.path().join("beta");
    fs::create_dir_all(&child_a).expect("create alpha child dir in explicit cwd");
    fs::create_dir_all(&child_b).expect("create beta child dir in explicit cwd");

    let output = dx()
        .args([
            "menu",
            "--buffer",
            "cd a",
            "--cursor",
            "4",
            "--cwd",
            explicit_cwd
                .path()
                .to_str()
                .expect("explicit cwd path should be valid utf-8"),
        ])
        .current_dir(process_cwd.path())
        .output()
        .expect("dx menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed["action"], "replace");
    assert_eq!(parsed["replaceStart"], 3);
    assert_eq!(parsed["replaceEnd"], 4);
    assert_eq!(
        parsed["terminal"], "clean",
        "single-candidate fast path should emit terminal=clean"
    );

    let value = parsed["value"]
        .as_str()
        .expect("replace action should include value");
    assert!(
        value.ends_with(std::path::MAIN_SEPARATOR),
        "paths mode replacement should drill in"
    );

    let replaced_path = value
        .strip_suffix(std::path::MAIN_SEPARATOR)
        .expect("replacement should end with the native separator");
    let replaced_abs = if std::path::Path::new(replaced_path).is_relative() {
        explicit_cwd.path().join(replaced_path)
    } else {
        std::path::PathBuf::from(replaced_path)
    };
    let replaced_canon =
        fs::canonicalize(replaced_abs).expect("replacement value path should exist");
    let expected_alpha =
        fs::canonicalize(&child_a).expect("expected child path should canonicalize");
    assert_eq!(
        replaced_canon, expected_alpha,
        "expected explicit cwd candidate identity to be selected"
    );
}

#[cfg(unix)]
#[test]
fn menu_paths_mode_bare_query_uses_bare_relative_replacement() {
    let explicit_cwd = common::temp_dir("explicit-cwd-relative-rendering");
    let child = explicit_cwd.path().join("benches");
    fs::create_dir_all(&child).expect("create benches child dir");

    let output = dx()
        .args([
            "menu",
            "--buffer",
            "cd b",
            "--cursor",
            "4",
            "--cwd",
            explicit_cwd
                .path()
                .to_str()
                .expect("explicit cwd path should be valid utf-8"),
        ])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed["action"], "replace");
    assert_eq!(parsed["replaceStart"], 3);
    assert_eq!(parsed["replaceEnd"], 4);
    assert_eq!(parsed["terminal"], "clean");

    let value = parsed["value"]
        .as_str()
        .expect("replace action should include value");
    assert_eq!(value, format!("benches{}", std::path::MAIN_SEPARATOR));
}

#[cfg(unix)]
#[test]
fn menu_paths_mode_explicit_absolute_query_preserves_absolute_replacement() {
    let explicit_cwd = common::temp_dir("explicit-cwd-absolute-query");
    let child = explicit_cwd.path().join("benches");
    fs::create_dir_all(&child).expect("create benches child dir");

    let query = explicit_cwd.path().join("b").display().to_string();
    let buffer = format!("cd {query}");
    let output = dx()
        .args([
            "menu",
            "--buffer",
            &buffer,
            "--cursor",
            &buffer.len().to_string(),
            "--cwd",
            explicit_cwd
                .path()
                .to_str()
                .expect("explicit cwd path should be valid utf-8"),
        ])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed["action"], "replace");
    assert_eq!(parsed["replaceStart"], 3);
    assert_eq!(parsed["replaceEnd"], buffer.len());
    assert_eq!(parsed["terminal"], "clean");

    let value = parsed["value"]
        .as_str()
        .expect("replace action should include value");
    let expected = format!("{}{}", child.display(), std::path::MAIN_SEPARATOR);
    assert_eq!(value, expected);
}

#[cfg(unix)]
#[test]
fn menu_paths_mode_home_query_preserves_home_replacement() {
    let root = common::temp_dir("menu-home-query-replacement");
    let home = root.path().join("home");
    let cwd = root.path().join("work/nested/project");
    let child = home.join("code");
    fs::create_dir_all(&child).expect("create home child directory");
    fs::create_dir_all(&cwd).expect("create nested working directory");

    let buffer = "cd ~/c";
    let output = dx()
        .args([
            "menu",
            "--buffer",
            buffer,
            "--cursor",
            &buffer.len().to_string(),
            "--cwd",
            cwd.to_str().expect("cwd should be valid utf-8"),
        ])
        .env("HOME", &home)
        .output()
        .expect("dx menu should run");

    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be a valid menu action");
    assert_eq!(parsed["action"], "replace");
    assert_eq!(parsed["value"], "~/code/");
}

#[cfg(unix)]
#[test]
fn menu_paths_mode_parent_relative_query_preserves_parent_prefix_replacement() {
    let root = common::temp_dir("explicit-cwd-parent-relative");
    let explicit_cwd = root.path().join("work");
    let sibling = root.path().join("sibling");
    fs::create_dir_all(&explicit_cwd).expect("create explicit cwd dir");
    fs::create_dir_all(&sibling).expect("create sibling dir");

    let buffer = "cd ../s";
    let output = dx()
        .args([
            "menu",
            "--buffer",
            buffer,
            "--cursor",
            &buffer.len().to_string(),
            "--cwd",
            explicit_cwd
                .to_str()
                .expect("explicit cwd path should be valid utf-8"),
        ])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed["action"], "replace");
    assert_eq!(parsed["replaceStart"], 3);
    assert_eq!(parsed["replaceEnd"], buffer.len());
    assert_eq!(parsed["terminal"], "clean");

    let value = parsed["value"]
        .as_str()
        .expect("replace action should include value");
    assert_eq!(
        value,
        format!(
            "..{}sibling{}",
            std::path::MAIN_SEPARATOR,
            std::path::MAIN_SEPARATOR
        )
    );
}

#[cfg(unix)]
#[test]
fn menu_paths_mode_parent_relative_query_keeps_anchor_for_cwd_candidate() {
    let root = common::temp_dir("menu-parent-query-cwd-candidate");
    let cwd = root.path().join("work");
    fs::create_dir_all(&cwd).expect("create working directory");

    let buffer = "cd ../w";
    let output = dx()
        .args([
            "menu",
            "--buffer",
            buffer,
            "--cursor",
            &buffer.len().to_string(),
            "--cwd",
            cwd.to_str().expect("cwd should be valid utf-8"),
        ])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be a valid menu action");
    assert_eq!(parsed["action"], "replace");
    assert_eq!(parsed["value"], "../work/");
}

#[cfg(unix)]
#[test]
fn mapped_path_mode_returns_single_file_candidate_replace() {
    let explicit_cwd = common::temp_dir("mapped-path-file");
    let file = explicit_cwd.path().join("alpha.txt");
    fs::write(&file, "hello").expect("create file candidate");

    let output = dx()
        .args([
            "menu",
            "--mode",
            "path",
            "--buffer",
            "cat a",
            "--cursor",
            "5",
            "--cwd",
            explicit_cwd.path().to_str().expect("cwd utf-8"),
        ])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(parsed["action"], "replace");
    assert_eq!(parsed["value"], "alpha.txt");
    assert_eq!(parsed["terminal"], "clean");
}

#[cfg(unix)]
#[test]
fn mapped_directory_mode_excludes_files() {
    let explicit_cwd = common::temp_dir("mapped-directory-filter");
    let dir = explicit_cwd.path().join("alpha-dir");
    let file = explicit_cwd.path().join("alpha.txt");
    fs::create_dir_all(&dir).expect("create dir candidate");
    fs::write(&file, "hello").expect("create file candidate");

    let output = dx()
        .args([
            "menu",
            "--mode",
            "directory",
            "--buffer",
            "open alpha",
            "--cursor",
            "10",
            "--cwd",
            explicit_cwd.path().to_str().expect("cwd utf-8"),
        ])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(parsed["action"], "replace");
    assert_eq!(
        parsed["value"],
        format!("alpha-dir{}", std::path::MAIN_SEPARATOR)
    );
    assert_eq!(parsed["terminal"], "clean");
}

#[cfg(unix)]
#[test]
fn mapped_file_mode_excludes_directories() {
    let explicit_cwd = common::temp_dir("mapped-file-filter");
    let dir = explicit_cwd.path().join("alpha-dir");
    let file = explicit_cwd.path().join("alpha.txt");
    fs::create_dir_all(&dir).expect("create dir candidate");
    fs::write(&file, "hello").expect("create file candidate");

    let output = dx()
        .args([
            "menu",
            "--mode",
            "file",
            "--buffer",
            "cat alpha",
            "--cursor",
            "9",
            "--cwd",
            explicit_cwd.path().to_str().expect("cwd utf-8"),
        ])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(parsed["action"], "replace");
    assert_eq!(parsed["value"], "alpha.txt");
    assert_eq!(parsed["terminal"], "clean");
}

#[cfg(unix)]
#[test]
fn menu_flagged_cd_replace_span_starts_at_path_token() {
    let explicit_cwd = common::temp_dir("explicit-cwd-flagged-replace");
    let child = explicit_cwd.path().join("foo");
    fs::create_dir_all(&child).expect("create child dir in explicit cwd");

    let buffer = "cd -P f";
    let output = dx()
        .args([
            "menu",
            "--buffer",
            buffer,
            "--cursor",
            &buffer.len().to_string(),
            "--cwd",
            explicit_cwd
                .path()
                .to_str()
                .expect("explicit cwd path should be valid utf-8"),
        ])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed["action"], "replace");
    assert_eq!(parsed["replaceStart"], 6);
    assert_eq!(parsed["replaceEnd"], 7);
    assert_eq!(parsed["terminal"], "clean");

    let value = parsed["value"]
        .as_str()
        .expect("replace action should include value");
    let replace_start = parsed["replaceStart"]
        .as_u64()
        .expect("replaceStart should be u64") as usize;
    let replace_end = parsed["replaceEnd"]
        .as_u64()
        .expect("replaceEnd should be u64") as usize;
    let rebuilt = format!(
        "{}{}{}",
        &buffer[..replace_start],
        value,
        &buffer[replace_end..]
    );
    assert!(
        rebuilt.starts_with("cd -P "),
        "flag prefix should remain unchanged: {rebuilt}"
    );

    let replaced_path = value
        .strip_suffix(std::path::MAIN_SEPARATOR)
        .expect("replacement should end with the native separator");
    let replaced_abs = if std::path::Path::new(replaced_path).is_relative() {
        explicit_cwd.path().join(replaced_path)
    } else {
        std::path::PathBuf::from(replaced_path)
    };
    let replaced_canon =
        fs::canonicalize(replaced_abs).expect("replacement value path should exist");
    let expected_child = fs::canonicalize(&child).expect("expected child path should canonicalize");
    assert_eq!(replaced_canon, expected_child);
}

// --- 5.5 Debug instrumentation ---

#[test]
fn menu_debug_mode_emits_stderr_diagnostics() {
    let output = dx()
        .args(["menu", "--buffer", "cd foo", "--cursor", "6"])
        .env("DX_MENU_DEBUG", "1")
        .output()
        .expect("dx menu should run");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[dx-menu-debug]"),
        "DX_MENU_DEBUG=1 should emit debug output on stderr, got: {stderr}"
    );
    assert!(
        stderr.contains("buffer="),
        "debug output should include buffer info"
    );
    // stdout should still be valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should still be valid JSON");
}

#[test]
fn menu_debug_mode_off_by_default() {
    let output = dx()
        .args(["menu", "--buffer", "cd foo", "--cursor", "6"])
        .env_remove("DX_MENU_DEBUG")
        .output()
        .expect("dx menu should run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("[dx-menu-debug]"),
        "debug output should not appear without DX_MENU_DEBUG=1"
    );
}
