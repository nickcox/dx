use std::path::PathBuf;
use std::process::Command;

fn dx_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_dx"))
}

#[test]
fn init_bash_prints_non_empty_output() {
    let output = Command::new(dx_bin())
        .args(["init", "bash"])
        .output()
        .expect("run init bash");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty());
    assert!(stdout.contains("_dx_complete_dx dx"));
    assert!(stdout.contains("up()"));
    assert!(stdout.contains("back()"));
    assert!(stdout.contains("forward()"));
}

#[test]
fn init_zsh_prints_non_empty_output() {
    let output = Command::new(dx_bin())
        .args(["init", "zsh"])
        .output()
        .expect("run init zsh");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty());
    assert!(stdout.contains("compdef _dx_complete_dx dx"));
    assert!(stdout.contains("compdef _dx_complete_ancestors up"));
    assert!(stdout.contains("up()"));
    assert!(stdout.contains("back()"));
    assert!(stdout.contains("forward()"));
}

#[test]
fn init_fish_prints_non_empty_output() {
    let output = Command::new(dx_bin())
        .args(["init", "fish"])
        .output()
        .expect("run init fish");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty());
    assert!(stdout.contains("complete -c dx"));
    assert!(stdout.contains("function up"));
    assert!(stdout.contains("function back"));
    assert!(stdout.contains("function forward"));
}

#[test]
fn init_pwsh_prints_non_empty_output() {
    let output = Command::new(dx_bin())
        .args(["init", "pwsh"])
        .output()
        .expect("run init pwsh");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty());
    assert!(stdout.contains("Register-ArgumentCompleter -CommandName dx"));
    assert!(stdout.contains("function up"));
    assert!(stdout.contains("function back"));
    assert!(stdout.contains("function forward"));
}

#[test]
fn init_unknown_shell_fails_with_diagnostic() {
    let output = Command::new(dx_bin())
        .args(["init", "unknown"])
        .output()
        .expect("run init unknown");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unsupported shell"));
    assert!(stderr.contains("bash, zsh, fish, pwsh"));
}

#[test]
fn init_bash_with_command_not_found_flag_includes_handler() {
    let output = Command::new(dx_bin())
        .args(["init", "bash", "--command-not-found"])
        .output()
        .expect("run init bash with command-not-found");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("command_not_found_handle"));
}

#[test]
fn init_bash_without_command_not_found_flag_excludes_handler() {
    let output = Command::new(dx_bin())
        .args(["init", "bash"])
        .output()
        .expect("run init bash");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("command_not_found_handle"));
}
