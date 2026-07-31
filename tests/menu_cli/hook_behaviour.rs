//! Runs the generated hooks and checks how they apply or reject an action.

use std::fs;
#[cfg(not(unix))]
use std::process::Command;
#[cfg(unix)]
use std::process::Command;

use super::common;
use super::support::*;

#[cfg(unix)]
#[test]
fn zsh_stack_navigation_works_when_current_directory_blocks_process_spawn() {
    if !common::optional_tool_available("zsh") {
        return;
    }

    let temp = common::temp_dir("zsh-stack-restricted-cwd");
    let start = temp.path().join("start");
    let blocked = temp.path().join("blocked");
    let runtime = temp.path().join("runtime");
    fs::create_dir_all(&start).expect("create start directory");
    fs::create_dir_all(&blocked).expect("create blocked directory");
    fs::create_dir_all(&runtime).expect("create runtime directory");

    let quote = |path: &std::path::Path| path.display().to_string().replace('\'', "'\\''");
    let script = format!(
        "eval \"$($CARGO_BIN_EXE_dx init zsh)\"; \
         builtin cd '{start}'; \
         dx stack push '{start}' >/dev/null; \
         cd '{blocked}'; \
         chmod 600 '{blocked}'; \
         cd-; \
         __dx_status=$?; \
         chmod 700 '{blocked}'; \
         print -r -- \"status=$__dx_status\"; \
         print -r -- \"cwd=$PWD\"",
        start = quote(&start),
        blocked = quote(&blocked),
    );

    let output = Command::new("zsh")
        .args(["-f", "-c", &script])
        .env("PATH", path_with_dx_binary())
        .env("CARGO_BIN_EXE_dx", env!("CARGO_BIN_EXE_dx"))
        .env("DX_SESSION", "zsh-stack-restricted-cwd")
        .env("XDG_RUNTIME_DIR", &runtime)
        .output()
        .expect("zsh should run");

    assert!(
        output.status.success(),
        "zsh failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("permission denied"),
        "unexpected stderr: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.lines().any(|line| line == "status=0"),
        "unexpected stdout: {stdout}"
    );
    let cwd = stdout
        .lines()
        .find_map(|line| line.strip_prefix("cwd="))
        .expect("missing cwd output");
    common::assert_same_path(cwd, &start);
}

// --- 5.4 Terminal recovery contracts ---
// Full terminal-state recovery needs pty instrumentation, which is not set up here.
// These verify the structural guarantee: hooks fall back cleanly on error/noop.

#[test]
fn hook_scripts_contain_fallback_on_noop() {
    // Verify each shell's menu code falls back to native completion on noop
    let bash = dx().args(["init", "bash", "--menu"]).output().unwrap();
    let bash_out = String::from_utf8_lossy(&bash.stdout);
    // Bash: _dx_menu_wrapper calls original completion when __dx_try_menu fails
    assert!(
        bash_out.contains("if __dx_try_menu; then\n    [[ \"$__dx_menu_terminal\" == \"dirty\" && -t 1 ]] && printf '\\r' >/dev/tty\n    return 0\n  fi"),
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
    assert!(
        zsh_out.contains("if [[ \"$__dx_action\" == \"cancel\" ]]; then"),
        "zsh menu widget should treat cancel as handled without native fallback"
    );
    assert!(
        zsh_out.contains("CURSOR=${#BUFFER}"),
        "zsh cancel path should restore cursor to end of buffer"
    );

    let fish = dx().args(["init", "fish", "--menu"]).output().unwrap();
    let fish_out = String::from_utf8_lossy(&fish.stdout);
    // Fish: __dx_menu_complete calls commandline -f complete on fallback
    assert!(
        fish_out.contains("commandline -f complete"),
        "fish menu should fall back to commandline -f complete"
    );
    assert!(
        fish_out.contains("if test \"$action\" = \"cancel\""),
        "fish menu should treat cancel as handled without native fallback"
    );
    assert!(
        fish_out.contains("commandline -C (string length -- \"$buf\")"),
        "fish cancel path should restore cursor to end of buffer"
    );

    let pwsh = dx().args(["init", "pwsh", "--menu"]).output().unwrap();
    let pwsh_out = String::from_utf8_lossy(&pwsh.stdout);
    // PowerShell: Tab handler calls TabCompleteNext on fallback
    assert!(
        pwsh_out.contains("TabCompleteNext"),
        "pwsh menu should fall back to TabCompleteNext"
    );
    assert!(
        pwsh_out.contains("if ($result -and $result.action -eq 'cancel')"),
        "pwsh menu should treat cancel as handled without native fallback"
    );
    assert!(
        pwsh_out.contains("[Microsoft.PowerShell.PSConsoleReadLine]::SetCursorPosition($cursor)"),
        "pwsh cancel path should preserve the logical cursor"
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

#[test]
fn hook_scripts_apply_replace_action_contract() {
    let bash = dx().args(["init", "bash", "--menu"]).output().unwrap();
    let bash_out = String::from_utf8_lossy(&bash.stdout);
    assert!(bash_out.contains("__dx_json_extract_string()"));
    assert!(bash_out.contains("__dx_json_extract_uint()"));
    assert!(bash_out.contains("__dx_action=\"$(__dx_json_extract_string action \"$__dx_json\")\""));
    assert!(
        bash_out.contains("__dx_terminal=\"$(__dx_json_extract_string terminal \"$__dx_json\")\"")
    );
    assert!(bash_out.contains(
        "[[ \"$__dx_terminal\" == \"clean\" || \"$__dx_terminal\" == \"dirty\" ]] || return 1"
    ));
    assert!(bash_out.contains("[[ \"$__dx_action\" == \"cancel\" ]] && return 0"));
    assert!(bash_out.contains("(( __dx_re >= __dx_rs )) || return 1"));
    assert!(
        bash_out.contains(
            "[[ \"$__dx_menu_terminal\" == \"dirty\" && -t 1 ]] && printf '\\r' >/dev/tty"
        )
    );

    let zsh = dx().args(["init", "zsh", "--menu"]).output().unwrap();
    let zsh_out = String::from_utf8_lossy(&zsh.stdout);
    assert!(zsh_out.contains("replaceStart"));
    assert!(zsh_out.contains("replaceEnd"));
    assert!(zsh_out.contains("__dx_value"));
    assert!(zsh_out.contains("__dx_terminal_marker=\"\\\"terminal\\\":\\\"\""));
    assert!(zsh_out.contains("[[ \"$__dx_terminal\" == \"clean\" || \"$__dx_terminal\" == \"dirty\" ]] || { zle expand-or-complete; return }"));
    assert!(zsh_out.contains("[[ \"$__dx_terminal\" == \"dirty\" ]] && zle reset-prompt"));
    assert!(zsh_out.contains("if [[ \"$__dx_action\" == \"cancel\" ]]; then"));
    assert!(zsh_out.contains("CURSOR=${#BUFFER}"));
    assert!(
        zsh_out.contains(
            "[[ \"$__dx_action\" == \"replace\" ]] || { zle expand-or-complete; return }"
        )
    );
    assert!(zsh_out.contains("(( __dx_re >= __dx_rs )) || { zle expand-or-complete; return }"));

    let fish = dx().args(["init", "fish", "--menu"]).output().unwrap();
    let fish_out = String::from_utf8_lossy(&fish.stdout);
    assert!(fish_out.contains("replaceStart"));
    assert!(fish_out.contains("replaceEnd"));
    assert!(fish_out.contains("set -l terminal (string replace -r '.*\\\"terminal\\\":\\\"([^\\\"[:space:]]+)\\\".*' '$1' -- \"$json\")"));
    assert!(fish_out.contains("if test \"$terminal\" != \"clean\" -a \"$terminal\" != \"dirty\""));
    assert!(fish_out.contains("if test \"$terminal\" = \"dirty\""));
    assert!(fish_out.contains("if test \"$action\" = \"cancel\""));
    assert!(fish_out.contains("commandline -C (string length -- \"$buf\")"));
    assert!(fish_out.contains(r#"commandline -r -- "$prefix$value$suffix""#));
    assert!(fish_out.contains("string match -r '.*\"value\":\"((\\\\.|[^\"])*)\".*'"));
    assert!(fish_out.contains("if test $re -lt $rs"));

    let pwsh = dx().args(["init", "pwsh", "--menu"]).output().unwrap();
    let pwsh_out = String::from_utf8_lossy(&pwsh.stdout);
    assert!(pwsh_out.contains("if ($result -and $result.action -eq 'cancel')"));
    assert!(
        pwsh_out.contains("[Microsoft.PowerShell.PSConsoleReadLine]::SetCursorPosition($cursor)")
    );
    assert!(pwsh_out.contains("$result.action -ne 'replace'"));
    assert!(pwsh_out.contains("$result.terminal -ne 'clean' -and $result.terminal -ne 'dirty'"));
    assert!(pwsh_out.contains("if ($result.terminal -eq 'dirty')"));
    assert!(pwsh_out.contains("PSConsoleReadLine]::Replace("));
}

#[test]
fn hook_scripts_do_not_perform_intermediate_menu_edits() {
    let bash = dx().args(["init", "bash", "--menu"]).output().unwrap();
    let bash_out = String::from_utf8_lossy(&bash.stdout);
    assert!(bash_out.contains("dx menu --shell bash --buffer"));
    assert!(!bash_out.contains("dx menu --append"));

    let zsh = dx().args(["init", "zsh", "--menu"]).output().unwrap();
    let zsh_out = String::from_utf8_lossy(&zsh.stdout);
    assert!(zsh_out.matches("dx menu --shell zsh --buffer").count() >= 1);
}
