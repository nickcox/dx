//! Generated hook content for bash, zsh and fish, plus the menu-disabled forms.

#[cfg(not(unix))]
use std::process::Command;
#[cfg(unix)]
use std::process::Command;

use super::support::*;

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
        stdout.contains("dx menu --shell bash --buffer"),
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
fn init_bash_menu_ignores_pwsh_menu_key() {
    let output = dx()
        .args(["init", "bash", "--menu"])
        .env("DX_PWSH_MENU_KEY", "F12")
        .output()
        .expect("dx init bash --menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("F12"));
}

#[test]
fn init_with_invalid_menu_mappings_fails_when_menu_enabled() {
    let output = dx()
        .args(["init", "bash", "--menu"])
        .env("DX_MENU_COMMAND_MAPPINGS", "ls=path,badentry")
        .output()
        .expect("dx init bash --menu should run");

    assert!(
        !output.status.success(),
        "invalid mappings should fail init"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid DX_MENU_COMMAND_MAPPINGS"));
}

#[test]
fn init_rejects_shell_injection_in_menu_mappings() {
    let cases = [
        ("bash", "bad); echo injected=path"),
        ("zsh", "bad); echo injected=path"),
        ("fish", "bad; echo injected=path"),
        ("pwsh", "bad'; Write-Output injected=path"),
    ];

    for (shell, mappings) in cases {
        let output = dx()
            .args(["init", shell, "--menu"])
            .env("DX_MENU_COMMAND_MAPPINGS", mappings)
            .output()
            .expect("dx init should run");

        assert!(
            !output.status.success(),
            "{shell} accepted unsafe mappings: {mappings:?}"
        );
        assert!(
            output.stdout.is_empty(),
            "{shell} emitted a script for unsafe mappings"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("invalid DX_MENU_COMMAND_MAPPINGS"),
            "{shell} did not explain the invalid mapping"
        );
    }
}

#[test]
fn generated_hooks_with_safe_mappings_pass_available_shell_parsers() {
    #[cfg(unix)]
    {
        assert_hook_parses_with("bash", "bash", &["-n"]);
        assert_hook_parses_with("zsh", "zsh", &["-n"]);
        assert_hook_parses_with("fish", "fish", &["--no-execute"]);
    }

    if pwsh_available() {
        let generated = dx()
            .args(["init", "pwsh", "--menu"])
            .env(
                "DX_MENU_COMMAND_MAPPINGS",
                "Get-ChildItem=path,git.status=file",
            )
            .output()
            .expect("generate PowerShell hook for syntax check");
        assert!(generated.status.success());

        let source = String::from_utf8(generated.stdout).expect("PowerShell hook should be UTF-8");
        let checked = Command::new("pwsh")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "$tokens = $null; $errors = $null; [System.Management.Automation.Language.Parser]::ParseInput($env:DX_HOOK_SOURCE, [ref]$tokens, [ref]$errors) > $null; if ($errors.Count -gt 0) { $errors | ForEach-Object { [Console]::Error.WriteLine($_) }; exit 1 }",
            ])
            .env("DX_HOOK_SOURCE", source)
            .output()
            .expect("run PowerShell parser");
        assert!(
            checked.status.success(),
            "generated PowerShell hook failed syntax check: {}",
            String::from_utf8_lossy(&checked.stderr)
        );
    }
}

#[test]
fn init_bash_menu_with_mappings_emits_explicit_mode_bindings() {
    let output = dx()
        .args(["init", "bash", "--menu"])
        .env("DX_MENU_COMMAND_MAPPINGS", "ls=path,cat=file")
        .output()
        .expect("dx init bash --menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("complete -F _dx_menu_wrapper ls"));
    assert!(stdout.contains("complete -F _dx_menu_wrapper cat"));
    assert!(stdout.contains("ls) __dx_menu_mode=\"path\" ;;"));
    assert!(stdout.contains("cat) __dx_menu_mode=\"file\" ;;"));
    assert!(
        stdout.contains(
            "dx menu --shell bash --mode \"$__dx_mode_override\" --buffer \"$COMP_LINE\""
        )
    );
}

#[test]
fn init_zsh_menu_with_mappings_emits_shared_widget_mode_routing() {
    let output = dx()
        .args(["init", "zsh", "--menu"])
        .env("DX_MENU_COMMAND_MAPPINGS", "open=path")
        .output()
        .expect("dx init zsh --menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("open) __dx_menu_mode=\"path\" ;;"));
    assert!(stdout.contains("dx menu --shell zsh --mode \"$__dx_menu_mode\" --buffer \"$BUFFER\""));
}

#[test]
fn init_fish_menu_with_mappings_emits_shared_helper_mode_routing() {
    let output = dx()
        .args(["init", "fish", "--menu"])
        .env("DX_MENU_COMMAND_MAPPINGS", "ls=path")
        .output()
        .expect("dx init fish --menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("case ls\n      set -l dx_menu_mode path"));
    assert!(stdout.contains("dx menu --shell fish --mode \"$dx_menu_mode\" --buffer \"$buf\""));
}

#[test]
fn init_non_pwsh_menu_mappings_remain_literal_command_registrations() {
    let bash = dx()
        .args(["init", "bash", "--menu"])
        .env("DX_MENU_COMMAND_MAPPINGS", "Get-ChildItem=path")
        .output()
        .expect("dx init bash --menu should run");
    let zsh = dx()
        .args(["init", "zsh", "--menu"])
        .env("DX_MENU_COMMAND_MAPPINGS", "Get-ChildItem=path")
        .output()
        .expect("dx init zsh --menu should run");
    let fish = dx()
        .args(["init", "fish", "--menu"])
        .env("DX_MENU_COMMAND_MAPPINGS", "Get-ChildItem=path")
        .output()
        .expect("dx init fish --menu should run");

    assert!(bash.status.success());
    assert!(zsh.status.success());
    assert!(fish.status.success());

    let bash_stdout = String::from_utf8_lossy(&bash.stdout);
    let zsh_stdout = String::from_utf8_lossy(&zsh.stdout);
    let fish_stdout = String::from_utf8_lossy(&fish.stdout);

    assert!(bash_stdout.contains("Get-ChildItem) __dx_menu_mode=\"path\" ;;"));
    assert!(zsh_stdout.contains("'Get-ChildItem') __dx_menu_mode=\"path\" ;;"));
    assert!(fish_stdout.contains("case 'Get-ChildItem'\n      set -l dx_menu_mode path"));
    assert!(!bash_stdout.contains("Get-Alias -Definition"));
    assert!(!zsh_stdout.contains("Get-Alias -Definition"));
    assert!(!fish_stdout.contains("Get-Alias -Definition"));
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
