mod common;

use std::env;
use std::process::Command;

#[test]
fn init_bash_prints_non_empty_output() {
    let output = Command::new(common::dx_bin())
        .args(["init", "bash"])
        .output()
        .expect("run init bash");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty());
    assert!(stdout.contains("complete -F _dx -o bashdefault -o default dx"));
    assert!(stdout.contains("up()"));
    assert!(stdout.contains("back()"));
    assert!(stdout.contains("forward()"));
}

#[test]
fn init_zsh_prints_non_empty_output() {
    let output = Command::new(common::dx_bin())
        .args(["init", "zsh"])
        .output()
        .expect("run init zsh");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty());
    assert!(stdout.contains("#compdef dx"));
    assert!(stdout.contains("compdef _dx_complete_ancestors up"));
    assert!(stdout.contains("up()"));
    assert!(stdout.contains("back()"));
    assert!(stdout.contains("forward()"));
}

#[test]
fn init_fish_prints_non_empty_output() {
    let output = Command::new(common::dx_bin())
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
    let output = Command::new(common::dx_bin())
        .args(["init", "pwsh"])
        .output()
        .expect("run init pwsh");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty());
    assert!(stdout.contains("Register-ArgumentCompleter -Native -CommandName 'dx'"));
    assert!(stdout.contains("function Step-Up"));
    assert!(stdout.contains("function Undo-Location"));
    assert!(stdout.contains("function Redo-Location"));
    assert!(stdout.contains("__dx_set_alias up Step-Up"));
    assert!(stdout.contains("__dx_set_alias back Undo-Location"));
    assert!(stdout.contains("__dx_set_alias forward Redo-Location"));
}

#[test]
fn pwsh_completes_partial_dx_subcommands() {
    if !common::tool_available("pwsh") {
        return;
    }

    let binary_path = common::dx_bin();
    let bin_dir = binary_path.parent().expect("dx binary parent directory");
    let mut paths = vec![bin_dir.to_path_buf()];
    paths.extend(env::split_paths(&env::var_os("PATH").unwrap_or_default()));
    let path = env::join_paths(paths)
        .expect("join PATH")
        .to_string_lossy()
        .replace('\'', "''");
    let binary = binary_path.display().to_string().replace('\'', "''");
    let script = format!(
        "$env:PATH = '{path}'; Invoke-Expression ((& '{binary}' init pwsh | Out-String)); (TabExpansion2 -inputScript 'dx r' -cursorColumn 4).CompletionMatches | ForEach-Object {{ $_.CompletionText }}"
    );
    let output = Command::new("pwsh")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .expect("run PowerShell completion");

    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|value| value == "resolve")
    );
}

#[test]
fn init_unknown_shell_fails_with_diagnostic() {
    let output = Command::new(common::dx_bin())
        .args(["init", "unknown"])
        .output()
        .expect("run init unknown");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value"));
    assert!(stderr.contains("bash, zsh, fish, pwsh"));
}

#[test]
fn init_native_menu_is_power_shell_only() {
    let output = Command::new(common::dx_bin())
        .args(["init", "bash", "--native-menu"])
        .output()
        .expect("run init bash --native-menu");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("--native-menu is only supported for pwsh")
    );
}

#[test]
fn init_rejects_tui_and_native_menu_together() {
    let output = Command::new(common::dx_bin())
        .args(["init", "pwsh", "--menu", "--native-menu"])
        .output()
        .expect("run conflicting menu modes");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cannot be used with"));
}

#[test]
fn init_bash_with_command_not_found_flag_includes_handler() {
    let output = Command::new(common::dx_bin())
        .args(["init", "bash", "--command-not-found"])
        .output()
        .expect("run init bash with command-not-found");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("command_not_found_handle"));
}

#[test]
fn init_bash_without_command_not_found_flag_excludes_handler() {
    let output = Command::new(common::dx_bin())
        .args(["init", "bash"])
        .output()
        .expect("run init bash");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("command_not_found_handle"));
}

#[test]
fn init_zsh_with_command_not_found_flag_includes_handler() {
    let output = Command::new(common::dx_bin())
        .args(["init", "zsh", "--command-not-found"])
        .output()
        .expect("run init zsh with command-not-found");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("command_not_found_handler"));
}

/// `dx init` with a setting in the config file, and the environment cleared so
/// only the file can supply it.
fn init_with_config(body: &str, args: &[&str], env: &[(&str, &str)]) -> std::process::Output {
    let temp = common::temp_dir("init-config");
    let file = temp.path().join("dx.toml");
    std::fs::write(&file, body).expect("write config file");

    let mut command = Command::new(common::dx_bin());
    command.args(["init"]).args(args).env("DX_CONFIG", &file);
    for name in ["DX_MENU_COMMAND_MAPPINGS", "DX_PWSH_MENU_KEY"] {
        command.env_remove(name);
    }
    for (name, value) in env {
        command.env(name, value);
    }
    command.output().expect("run dx init")
}

#[test]
fn config_file_settings_produce_the_same_script_as_the_environment() {
    let from_file = init_with_config(
        "[menu]\ncommand_mappings = \"ls=path,cat=file\"\npwsh_key = \"F12\"\n",
        &["pwsh", "--menu"],
        &[],
    );

    let from_env = Command::new(common::dx_bin())
        .args(["init", "pwsh", "--menu"])
        .env_remove("DX_CONFIG")
        .env("DX_MENU_COMMAND_MAPPINGS", "ls=path,cat=file")
        .env("DX_PWSH_MENU_KEY", "F12")
        .output()
        .expect("run dx init");

    assert!(from_file.status.success());
    assert!(from_env.status.success());
    assert_eq!(from_file.stdout, from_env.stdout);
}

#[test]
fn the_environment_overrides_config_file_settings() {
    let output = init_with_config(
        "[menu]\npwsh_key = \"F12\"\n",
        &["pwsh", "--menu"],
        &[("DX_PWSH_MENU_KEY", "F5")],
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Set-PSReadLineKeyHandler -Key 'F5'"));
    assert!(!stdout.contains("Set-PSReadLineKeyHandler -Key 'F12'"));
}

#[test]
fn a_broken_config_file_still_emits_a_usable_hook() {
    // The output is evaluated by shell profiles, so failing here would break
    // shell startup — much worse than ignoring the file.
    let output = init_with_config("{this is not toml\n", &["bash", "--menu"], &[]);

    assert!(
        output.status.success(),
        "init must not fail on a bad config"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("cd()"),
        "a real hook should still be emitted"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("warning: ignoring config file"),
        "the problem should still be reported; stderr: {stderr}"
    );
}

#[test]
fn invalid_mappings_in_the_config_file_warn_rather_than_fail() {
    let output = init_with_config(
        "[menu]\ncommand_mappings = \"ls=path,badentry\"\n",
        &["bash", "--menu"],
        &[],
    );

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("warning: ignoring menu.command_mappings"),
        "stderr: {stderr}"
    );
}

#[test]
fn invalid_mappings_in_the_environment_still_fail() {
    // A value the user just typed is worth failing on; a stale config file is not.
    let output = init_with_config(
        "",
        &["bash", "--menu"],
        &[("DX_MENU_COMMAND_MAPPINGS", "ls=path,badentry")],
    );

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid DX_MENU_COMMAND_MAPPINGS"),
        "stderr: {stderr}"
    );
}
