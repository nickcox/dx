use std::process::Command;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

fn dx() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dx"))
}

fn make_temp_dir(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "dx-menu-cli-{label}-{nonce}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

// --- 4.2 Non-interactive / noop behavior ---

#[test]
fn menu_without_tty_outputs_noop_json() {
    // When run non-interactively (no TTY), dx menu should output {"action":"noop"}
    let output = dx()
        .args(["menu", "--buffer", "cd foo", "--cursor", "6"])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success(), "exit code should be 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed["action"], "noop");
}

#[test]
fn menu_unrecognized_command_outputs_noop() {
    let output = dx()
        .args(["menu", "--buffer", "ls -la", "--cursor", "5"])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(r#""action":"noop"#),
        "unrecognized command should produce noop"
    );
}

#[test]
fn menu_empty_buffer_outputs_noop() {
    let output = dx()
        .args(["menu", "--buffer", "", "--cursor", "0"])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed["action"], "noop");
}

// --- 4.2 Selection output contract ---

#[test]
fn menu_noop_json_has_only_action_field() {
    let output = dx()
        .args(["menu", "--buffer", "cd x", "--cursor", "4"])
        .output()
        .expect("dx menu should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed["action"], "noop");
    // noop should not have replaceStart/replaceEnd/value fields
    assert!(parsed.get("replaceStart").is_none());
    assert!(parsed.get("replaceEnd").is_none());
    assert!(parsed.get("value").is_none());
}

// --- 4.2 Shell hook invocation / action application contracts ---

#[test]
fn init_bash_with_menu_flag_includes_menu_code() {
    let output = dx()
        .args(["init", "bash", "--menu"])
        .output()
        .expect("dx init bash --menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("__dx_try_menu"),
        "bash with --menu should include __dx_try_menu"
    );
    assert!(
        stdout.contains("_dx_menu_wrapper"),
        "bash with --menu should include _dx_menu_wrapper"
    );
    assert!(
        stdout.contains("dx menu --buffer"),
        "bash with --menu should invoke dx menu"
    );
    assert!(
        stdout.contains("</dev/tty"),
        "bash menu should redirect stdin from /dev/tty"
    );
}

#[test]
fn init_zsh_with_menu_flag_includes_menu_widget() {
    let output = dx()
        .args(["init", "zsh", "--menu"])
        .output()
        .expect("dx init zsh --menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("__dx_menu_widget"),
        "zsh with --menu should include __dx_menu_widget"
    );
    assert!(
        stdout.contains("zle -N __dx_menu_widget"),
        "zsh with --menu should register the ZLE widget"
    );
    assert!(
        stdout.contains("bindkey '^I' __dx_menu_widget"),
        "zsh with --menu should bind Tab"
    );
    assert!(
        stdout.contains("</dev/tty"),
        "zsh menu should redirect stdin from /dev/tty"
    );
}

#[test]
fn init_fish_with_menu_flag_includes_menu_binding() {
    let output = dx()
        .args(["init", "fish", "--menu"])
        .output()
        .expect("dx init fish --menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("__dx_menu_complete"),
        "fish with --menu should include __dx_menu_complete"
    );
    assert!(
        stdout.contains(r"bind \t __dx_menu_complete"),
        "fish with --menu should bind Tab"
    );
    assert!(
        stdout.contains("</dev/tty"),
        "fish menu should redirect stdin from /dev/tty"
    );
}

#[test]
fn init_pwsh_with_menu_flag_includes_psreadline_handler() {
    let output = dx()
        .args(["init", "pwsh", "--menu"])
        .output()
        .expect("dx init pwsh --menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Set-PSReadLineKeyHandler -Key Tab"),
        "pwsh with --menu should include Tab key handler"
    );
    assert!(
        stdout.contains("ConvertFrom-Json"),
        "pwsh with --menu should parse JSON"
    );
    assert!(
        stdout.contains("TabCompleteNext"),
        "pwsh with --menu should fall back to TabCompleteNext"
    );
    assert!(
        stdout.contains("--psreadline-mode"),
        "pwsh with --menu should invoke dx menu with --psreadline-mode"
    );
}

// --- 4.3 Regression: menu disabled leaves existing behavior unchanged ---

#[test]
fn init_bash_without_menu_excludes_menu_code() {
    let output = dx()
        .args(["init", "bash"])
        .output()
        .expect("dx init bash should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("__dx_try_menu"),
        "bash without --menu should NOT include __dx_try_menu"
    );
    assert!(
        !stdout.contains("_dx_menu_wrapper"),
        "bash without --menu should NOT include _dx_menu_wrapper"
    );
    // Standard completions should still be present
    assert!(
        stdout.contains("_dx_complete_paths"),
        "standard completion functions should still exist"
    );
}

#[test]
fn init_zsh_without_menu_excludes_menu_widget() {
    let output = dx()
        .args(["init", "zsh"])
        .output()
        .expect("dx init zsh should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("__dx_menu_widget"),
        "zsh without --menu should NOT include __dx_menu_widget"
    );
    // Standard completions should still be present
    assert!(
        stdout.contains("compdef _dx_complete_paths cd"),
        "standard completions should still exist"
    );
}

#[test]
fn init_fish_without_menu_excludes_menu_binding() {
    let output = dx()
        .args(["init", "fish"])
        .output()
        .expect("dx init fish should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("__dx_menu_complete"),
        "fish without --menu should NOT include __dx_menu_complete"
    );
}

#[test]
fn init_pwsh_without_menu_excludes_tab_handler() {
    let output = dx()
        .args(["init", "pwsh"])
        .output()
        .expect("dx init pwsh should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("Set-PSReadLineKeyHandler -Key Tab"),
        "pwsh without --menu should NOT include Tab handler"
    );
    // Standard completions should still be present
    assert!(
        stdout.contains("Register-ArgumentCompleter"),
        "standard completions should still exist"
    );
}

// --- 5.3 Completion-context interactivity contracts ---
// Full PTY-based "stays open" tests require a pseudo-terminal and are deferred.
// These verify the structural contracts that enable correct interactive behavior.

#[test]
fn menu_with_valid_dx_command_without_tty_returns_noop() {
    // In a non-TTY context (CI/piped), dx menu for a valid command
    // should still return noop (since no interactive TUI is possible).
    // This proves the TTY gate is effective — without TTY the menu
    // does not attempt to open, and falls back cleanly.
    for cmd in [
        "cd foo", "up", "cdf proj", "z proj", "cdr", "back", "forward", "cd- ", "cd+ ",
    ] {
        let cursor = cmd.len().to_string();
        let output = dx()
            .args(["menu", "--buffer", cmd, "--cursor", &cursor])
            .output()
            .unwrap_or_else(|_| panic!("dx menu should run for buffer '{cmd}'"));

        assert!(output.status.success(), "should succeed for '{cmd}'");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
            .unwrap_or_else(|_| panic!("should be valid JSON for '{cmd}': {stdout}"));
        assert_eq!(
            parsed["action"], "noop",
            "non-TTY context should produce noop for '{cmd}'"
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

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.is_empty(),
        "stderr should be silent on noop, got: {stderr}"
    );
}

#[test]
fn menu_paths_mode_honors_explicit_cwd() {
    let process_cwd = make_temp_dir("process-cwd-empty");
    let explicit_cwd = make_temp_dir("explicit-cwd-with-child");
    let child_a = explicit_cwd.join("alpha");
    let child_b = explicit_cwd.join("beta");
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
                .to_str()
                .expect("explicit cwd path should be valid utf-8"),
        ])
        .current_dir(&process_cwd)
        .output()
        .expect("dx menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed["action"], "replace");
    assert_eq!(parsed["replaceStart"], 3);
    assert_eq!(parsed["replaceEnd"], 4);

    let value = parsed["value"]
        .as_str()
        .expect("replace action should include value");
    assert!(
        value.ends_with('/'),
        "paths mode replacement should drill in"
    );

    let replaced_path = value
        .strip_suffix('/')
        .expect("replacement should end with slash");
    let replaced_canon =
        fs::canonicalize(replaced_path).expect("replacement value path should exist");
    let expected_alpha =
        fs::canonicalize(&child_a).expect("expected child path should canonicalize");
    assert_eq!(
        replaced_canon, expected_alpha,
        "expected explicit cwd candidate identity to be selected"
    );

    let _ = fs::remove_dir_all(process_cwd);
    let _ = fs::remove_dir_all(explicit_cwd);
}

#[test]
fn menu_paths_mode_relative_query_uses_dot_slash_replacement() {
    let explicit_cwd = make_temp_dir("explicit-cwd-relative-rendering");
    let child = explicit_cwd.join("benches");
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

    let value = parsed["value"]
        .as_str()
        .expect("replace action should include value");
    assert_eq!(value, "./benches/");

    let _ = fs::remove_dir_all(explicit_cwd);
}

#[test]
fn menu_paths_mode_explicit_absolute_query_preserves_absolute_replacement() {
    let explicit_cwd = make_temp_dir("explicit-cwd-absolute-query");
    let child = explicit_cwd.join("benches");
    fs::create_dir_all(&child).expect("create benches child dir");

    let query = format!("{}/b", explicit_cwd.display());
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

    let value = parsed["value"]
        .as_str()
        .expect("replace action should include value");
    let expected = format!("{}/", child.display());
    assert_eq!(value, expected);

    let _ = fs::remove_dir_all(explicit_cwd);
}

#[test]
fn menu_paths_mode_parent_relative_query_preserves_parent_prefix_replacement() {
    let root = make_temp_dir("explicit-cwd-parent-relative");
    let explicit_cwd = root.join("work");
    let sibling = root.join("sibling");
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

    let value = parsed["value"]
        .as_str()
        .expect("replace action should include value");
    assert_eq!(value, "../sibling/");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn menu_flagged_cd_replace_span_starts_at_path_token() {
    let explicit_cwd = make_temp_dir("explicit-cwd-flagged-replace");
    let child = explicit_cwd.join("foo");
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
        .strip_suffix('/')
        .expect("replacement should end with slash");
    let replaced_canon =
        fs::canonicalize(replaced_path).expect("replacement value path should exist");
    let expected_child = fs::canonicalize(&child).expect("expected child path should canonicalize");
    assert_eq!(replaced_canon, expected_child);

    let _ = fs::remove_dir_all(explicit_cwd);
}

#[test]
fn menu_psreadline_mode_keeps_posix_flagged_cd_as_fallback() {
    let buffer = "cd -P foo";
    let output = dx()
        .args([
            "menu",
            "--buffer",
            buffer,
            "--cursor",
            &buffer.len().to_string(),
            "--psreadline-mode",
        ])
        .env("DX_MENU_DEBUG", "1")
        .output()
        .expect("dx menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed["action"], "noop");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("parse_buffer returned None -> noop"),
        "expected fallback parse path in psreadline mode; stderr: {stderr}"
    );
}

// --- 5.4 Terminal recovery contracts ---
// Full terminal-state recovery tests require PTY instrumentation (deferred).
// These verify the structural guarantee: hooks fall back cleanly on error/noop.

#[test]
fn hook_scripts_contain_fallback_on_noop() {
    // Verify each shell's menu code falls back to native completion on noop
    let bash = dx().args(["init", "bash", "--menu"]).output().unwrap();
    let bash_out = String::from_utf8_lossy(&bash.stdout);
    // Bash: _dx_menu_wrapper calls original completion when __dx_try_menu fails
    assert!(
        bash_out.contains("__dx_try_menu; then\n    return 0\n  fi"),
        "bash menu wrapper should fall back to original completion"
    );

    let zsh = dx().args(["init", "zsh", "--menu"]).output().unwrap();
    let zsh_out = String::from_utf8_lossy(&zsh.stdout);
    // Zsh: noop/error and invalid-action paths should fall back to expand-or-complete.
    assert!(
        zsh_out.contains("if [[ $__dx_exit -ne 0 ]]; then\n    zle expand-or-complete\n    return"),
        "zsh menu widget non-zero exit branch should fall back to expand-or-complete"
    );
    assert!(
        zsh_out.contains(
            "[[ \"$__dx_action\" == \"replace\" ]] || { zle expand-or-complete; return }"
        ),
        "zsh menu widget should fall back when action is not replace"
    );

    let fish = dx().args(["init", "fish", "--menu"]).output().unwrap();
    let fish_out = String::from_utf8_lossy(&fish.stdout);
    // Fish: __dx_menu_complete calls commandline -f complete on fallback
    assert!(
        fish_out.contains("commandline -f complete"),
        "fish menu should fall back to commandline -f complete"
    );

    let pwsh = dx().args(["init", "pwsh", "--menu"]).output().unwrap();
    let pwsh_out = String::from_utf8_lossy(&pwsh.stdout);
    // PowerShell: Tab handler calls TabCompleteNext on fallback
    assert!(
        pwsh_out.contains("TabCompleteNext"),
        "pwsh menu should fall back to TabCompleteNext"
    );
}

#[test]
fn hook_scripts_check_exit_status_before_applying() {
    // Verify hooks check for non-zero exit / failed commands before applying
    let bash = dx().args(["init", "bash", "--menu"]).output().unwrap();
    let bash_out = String::from_utf8_lossy(&bash.stdout);
    assert!(
        bash_out.contains(r#"|| return 1"#),
        "bash should check dx menu exit status"
    );

    let zsh = dx().args(["init", "zsh", "--menu"]).output().unwrap();
    let zsh_out = String::from_utf8_lossy(&zsh.stdout);
    assert!(
        zsh_out.contains("__dx_exit") && zsh_out.contains("-ne 0"),
        "zsh should check dx menu exit status"
    );

    let fish = dx().args(["init", "fish", "--menu"]).output().unwrap();
    let fish_out = String::from_utf8_lossy(&fish.stdout);
    assert!(
        fish_out.contains("test $status -ne 0"),
        "fish should check dx menu exit status"
    );

    let pwsh = dx().args(["init", "pwsh", "--menu"]).output().unwrap();
    let pwsh_out = String::from_utf8_lossy(&pwsh.stdout);
    assert!(
        pwsh_out.contains("$LASTEXITCODE -ne 0"),
        "pwsh should check dx menu exit status"
    );
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

#[test]
fn hook_scripts_apply_replace_action_contract() {
    let bash = dx().args(["init", "bash", "--menu"]).output().unwrap();
    let bash_out = String::from_utf8_lossy(&bash.stdout);
    assert!(bash_out.contains("__dx_json_extract_string()"));
    assert!(bash_out.contains("__dx_json_extract_uint()"));
    assert!(bash_out.contains("__dx_action=\"$(__dx_json_extract_string action \"$__dx_json\")\""));
    assert!(bash_out.contains("(( __dx_re >= __dx_rs )) || return 1"));

    let zsh = dx().args(["init", "zsh", "--menu"]).output().unwrap();
    let zsh_out = String::from_utf8_lossy(&zsh.stdout);
    assert!(zsh_out.contains("replaceStart"));
    assert!(zsh_out.contains("replaceEnd"));
    assert!(zsh_out.contains("__dx_value"));
    assert!(zsh_out
        .contains("[[ \"$__dx_action\" == \"replace\" ]] || { zle expand-or-complete; return }"));
    assert!(zsh_out.contains("(( __dx_re >= __dx_rs )) || { zle expand-or-complete; return }"));

    let fish = dx().args(["init", "fish", "--menu"]).output().unwrap();
    let fish_out = String::from_utf8_lossy(&fish.stdout);
    assert!(fish_out.contains("replaceStart"));
    assert!(fish_out.contains("replaceEnd"));
    assert!(fish_out.contains(r#"commandline -r -- "$prefix$value$suffix""#));
    assert!(fish_out.contains("string match -r '.*\"value\":\"((\\\\.|[^\"])*)\".*'"));
    assert!(fish_out.contains("if test $re -lt $rs"));

    let pwsh = dx().args(["init", "pwsh", "--menu"]).output().unwrap();
    let pwsh_out = String::from_utf8_lossy(&pwsh.stdout);
    assert!(pwsh_out.contains("$result.action -ne 'replace'"));
    assert!(pwsh_out.contains("PSConsoleReadLine]::Replace("));
}

#[test]
fn hook_scripts_do_not_perform_intermediate_menu_edits() {
    let bash = dx().args(["init", "bash", "--menu"]).output().unwrap();
    let bash_out = String::from_utf8_lossy(&bash.stdout);
    assert!(bash_out.contains("dx menu --buffer"));
    assert!(!bash_out.contains("dx menu --append"));

    let zsh = dx().args(["init", "zsh", "--menu"]).output().unwrap();
    let zsh_out = String::from_utf8_lossy(&zsh.stdout);
    assert!(zsh_out.matches("dx menu --buffer").count() >= 1);
}
