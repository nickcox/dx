mod common;

use std::ffi::OsString;
use std::fs;
#[cfg(unix)]
use std::io::Write;
#[cfg(not(unix))]
use std::process::Command;
#[cfg(unix)]
use std::process::{Command, Stdio};

fn dx() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dx"))
}

fn pwsh_available() -> bool {
    common::optional_tool_available("pwsh")
}

fn path_with_dx_binary() -> OsString {
    let mut paths =
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect::<Vec<_>>();
    let dx_binary = std::path::PathBuf::from(env!("CARGO_BIN_EXE_dx"));
    if let Some(dx_dir) = dx_binary.parent()
        && !paths.iter().any(|existing| existing == dx_dir)
    {
        paths.insert(0, dx_dir.to_path_buf());
    }
    std::env::join_paths(paths).expect("join PATH entries")
}

#[cfg(unix)]
fn assert_hook_parses_with(shell: &str, command: &str, args: &[&str]) {
    if command == "bash" {
        assert!(
            common::tool_available(command),
            "bash is required for syntax checks"
        );
    } else if !common::optional_tool_available(command) {
        return;
    }

    let generated = dx()
        .args(["init", shell, "--menu"])
        .env(
            "DX_MENU_COMMAND_MAPPINGS",
            "Get-ChildItem=path,git.status=file",
        )
        .output()
        .expect("generate hook for syntax check");
    assert!(
        generated.status.success(),
        "failed to generate {shell} hook: {}",
        String::from_utf8_lossy(&generated.stderr)
    );

    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn shell syntax checker");
    child
        .stdin
        .take()
        .expect("syntax checker stdin")
        .write_all(&generated.stdout)
        .expect("write generated hook");
    let checked = child.wait_with_output().expect("wait for syntax checker");
    assert!(
        checked.status.success(),
        "generated {shell} hook failed syntax check: {}",
        String::from_utf8_lossy(&checked.stderr)
    );
}

// --- 4.2 Non-interactive / noop behavior ---

#[test]
fn menu_without_tty_outputs_noop_json() {
    // When run non-interactively (no TTY), dx menu should output {"action":"noop"}
    // unless the single-candidate fast path applies.
    let cwd = common::temp_dir("without-tty-noop");
    let output = dx()
        .args(["menu", "--buffer", "cd foo", "--cursor", "6"])
        .current_dir(cwd.path())
        .output()
        .expect("dx menu should run");

    assert!(output.status.success(), "exit code should be 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed, serde_json::json!({ "action": "noop" }));
}

#[test]
fn menu_unrecognized_command_outputs_noop() {
    let output = dx()
        .args(["menu", "--buffer", "ls -la", "--cursor", "5"])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success(), "exit code should be 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed, serde_json::json!({ "action": "noop" }));
}

#[test]
fn menu_empty_buffer_outputs_noop() {
    let output = dx()
        .args(["menu", "--buffer", "", "--cursor", "0"])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success(), "exit code should be 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed, serde_json::json!({ "action": "noop" }));
}

// --- 4.2 Selection output contract ---

#[test]
fn menu_noop_json_has_only_action_field() {
    let output = dx()
        .args(["menu", "--buffer", "cd x", "--cursor", "4"])
        .output()
        .expect("dx menu should run");

    assert!(output.status.success(), "exit code should be 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed, serde_json::json!({ "action": "noop" }));
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
fn init_pwsh_with_menu_flag_includes_psreadline_handler() {
    let output = dx()
        .args(["init", "pwsh", "--menu"])
        .output()
        .expect("dx init pwsh --menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Set-PSReadLineKeyHandler -Key 'Tab'"),
        "pwsh with --menu should include Tab key handler"
    );
    assert!(
        stdout.contains("ConvertFrom-Json"),
        "pwsh with --menu should parse JSON"
    );
    assert!(
        stdout.contains("__dx_pwsh_menu_fallback"),
        "pwsh with --menu should use fallback helper"
    );
    assert!(
        stdout.contains("dx menu --shell pwsh"),
        "pwsh with --menu should invoke dx menu in PowerShell mode"
    );
}

#[test]
fn init_pwsh_native_menu_uses_structured_argument_completers_without_key_handler() {
    let output = dx()
        .args(["init", "pwsh", "--native-menu"])
        .env("DX_MENU_COMMAND_MAPPINGS", "Get-Content=file")
        .env("DX_PWSH_MENU_KEY", "F12")
        .output()
        .expect("dx init pwsh --native-menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("function __dx_complete_json"));
    assert!(stdout.contains("function __dx_emit_native_completion"));
    assert!(stdout.contains("[System.Management.Automation.CompletionResult]::new("));
    assert!(stdout.contains(
        "Register-ArgumentCompleter -CommandName Set-DxLocation,cd,Set-Location -ParameterName Path"
    ));
    assert!(stdout.contains(
        "__dx_emit_native_completion (__dx_complete_json -Mode paths -Word $wordToComplete) -Directory"
    ));
    assert!(stdout.contains(
        "__dx_emit_native_completion (__dx_complete_json -Mode frecents -Word $wordToComplete)"
    ));
    assert!(!stdout.contains(
        "__dx_emit_native_completion (__dx_complete_json -Mode frecents -Word $wordToComplete) -Directory"
    ));
    assert!(stdout.contains("__dx_register_native_mapped_completions @('Get-Content=file')"));
    assert!(stdout.contains("-Mode filesystem"));
    assert!(stdout.contains("$dxArgs += @('--limit', [string]$probeLimit)"));
    assert!(stdout.contains("| showing first $($showingFirst.Value)"));
    assert!(stdout.contains("function __dx_truncate_native_label"));
    assert!(!stdout.contains("Test-Path -LiteralPath $value -PathType Container"));
    assert!(!stdout.contains("Set-PSReadLineKeyHandler"));
    // `dx menu` treats `--shell pwsh` as "driven by PSReadLine". The native
    // menu must therefore never invoke `dx menu` at all — it uses PowerShell's
    // own completion instead. This is the invariant that lets
    // `MenuCommand::psreadline_mode` be derived from the shell.
    assert!(!stdout.contains("dx menu --shell pwsh"));
    assert!(!stdout.contains("F12"));
}

#[test]
fn init_pwsh_native_menu_rejects_invalid_mappings() {
    let output = dx()
        .args(["init", "pwsh", "--native-menu"])
        .env("DX_MENU_COMMAND_MAPPINGS", "Get-Content=file,badentry")
        .output()
        .expect("dx init pwsh --native-menu should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid DX_MENU_COMMAND_MAPPINGS"));
}

#[test]
fn init_pwsh_native_menu_returns_structured_builtin_and_mapped_completions() {
    if !pwsh_available() {
        return;
    }

    let temp = common::temp_dir("pwsh-native-menu-completions");
    fs::create_dir(temp.path().join("alpha-dir")).expect("create directory");
    fs::create_dir(temp.path().join("alpha dir's")).expect("create quoted directory");
    fs::write(temp.path().join("alpha-file.txt"), "fixture").expect("create file");

    let script = r#"
$env:DX_MENU_COMMAND_MAPPINGS = 'Get-Content=file'
Set-StrictMode -Version Latest
Invoke-Expression ((& $env:CARGO_BIN_EXE_dx init pwsh --native-menu | Out-String))
$cd = @((TabExpansion2 'cd alpha' 8).CompletionMatches)
$quoted = @((TabExpansion2 "cd 'alpha d" 11).CompletionMatches)
$mapped = @((TabExpansion2 'gc alpha' 8).CompletionMatches)
$env:DX_MAX_MENU_RESULTS = '1'
$limited = @((TabExpansion2 'cd alpha' 8).CompletionMatches)
Remove-Item Env:DX_MAX_MENU_RESULTS
$env:DX_MENU_ITEM_MAX_LEN = '8'
$truncated = @((TabExpansion2 'cd alpha-' 9).CompletionMatches)
$env:DX_MENU_ITEM_MAX_LEN = '0'
$untruncated = @((TabExpansion2 'cd alpha-' 9).CompletionMatches)
Remove-Item Env:DX_MENU_ITEM_MAX_LEN
[PSCustomObject]@{
    cd = @($cd | ForEach-Object { [PSCustomObject]@{ text = $_.CompletionText; label = $_.ListItemText; tooltip = $_.ToolTip } })
    quoted = @($quoted | ForEach-Object { [PSCustomObject]@{ text = $_.CompletionText; label = $_.ListItemText; tooltip = $_.ToolTip } })
    mapped = @($mapped | ForEach-Object { [PSCustomObject]@{ text = $_.CompletionText; label = $_.ListItemText; tooltip = $_.ToolTip } })
    limitedCount = $limited.Count
    limitedTooltip = $limited[0].ToolTip
    truncated = $truncated[0].ListItemText
    untruncated = $untruncated[0].ListItemText
} | ConvertTo-Json -Compress -Depth 4
"#;
    let output = Command::new("pwsh")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .current_dir(temp.path())
        .env("PATH", path_with_dx_binary())
        .env("CARGO_BIN_EXE_dx", env!("CARGO_BIN_EXE_dx"))
        .output()
        .expect("pwsh should run");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("structured completion JSON");
    let cd = result["cd"].as_array().expect("cd completions");
    let cd_directory = cd
        .iter()
        .find(|candidate| {
            candidate["label"]
                .as_str()
                .is_some_and(|label| label.ends_with("/alpha-dir"))
        })
        .expect("alpha-dir completion");
    assert!(
        cd_directory["label"]
            .as_str()
            .expect("cd label")
            .ends_with("/alpha-dir")
    );
    assert!(
        cd_directory["tooltip"]
            .as_str()
            .expect("cd tooltip")
            .ends_with("alpha-dir")
    );
    let cd_text = cd_directory["text"].as_str().expect("cd completion text");
    assert!(!cd_text.starts_with('\''));
    assert!(cd_text.ends_with(&format!("alpha-dir{}", std::path::MAIN_SEPARATOR)));
    assert_eq!(
        result["quoted"]
            .as_array()
            .expect("quoted completions")
            .len(),
        1
    );
    assert!(
        result["quoted"][0]["text"]
            .as_str()
            .expect("quoted completion text")
            .contains("alpha dir''s")
    );
    assert_eq!(
        result["mapped"]
            .as_array()
            .expect("mapped completions")
            .len(),
        1,
        "unexpected mapped completions: {result}"
    );
    assert!(
        result["mapped"][0]["label"]
            .as_str()
            .expect("mapped label")
            .ends_with("/alpha-file.txt")
    );
    assert!(
        result["mapped"][0]["tooltip"]
            .as_str()
            .expect("mapped tooltip")
            .ends_with("alpha-file.txt")
    );
    assert_eq!(result["limitedCount"], 1);
    assert!(
        result["limitedTooltip"]
            .as_str()
            .expect("limited tooltip")
            .ends_with(" | showing first 1")
    );
    assert_eq!(result["truncated"], "…pha-dir");
    assert!(
        result["untruncated"]
            .as_str()
            .expect("untruncated label")
            .ends_with("/alpha-dir")
    );
}

#[test]
fn init_pwsh_menu_with_custom_key_emits_configured_key() {
    let output = dx()
        .args(["init", "pwsh", "--menu"])
        .env("DX_PWSH_MENU_KEY", "F12")
        .output()
        .expect("dx init pwsh --menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("$dxNewMenuKey = 'F12'"));
    assert!(stdout.contains("$Global:__dx_pwsh_menu_key = $dxNewMenuKey"));
    assert!(stdout.contains("Set-PSReadLineKeyHandler -Key 'F12'"));
    assert!(stdout.contains("-ScriptBlock"));
}

#[test]
fn init_pwsh_menu_with_empty_custom_key_defaults_to_tab() {
    let output = dx()
        .args(["init", "pwsh", "--menu"])
        .env("DX_PWSH_MENU_KEY", "   ")
        .output()
        .expect("dx init pwsh --menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("$dxNewMenuKey = 'Tab'"));
    assert!(stdout.contains("Set-PSReadLineKeyHandler -Key 'Tab'"));
    assert!(stdout.contains("-ScriptBlock"));
}

#[test]
fn init_pwsh_menu_with_unsafe_custom_key_fails() {
    let output = dx()
        .args(["init", "pwsh", "--menu"])
        .env("DX_PWSH_MENU_KEY", "Bad'Key")
        .output()
        .expect("dx init pwsh --menu should run");

    assert!(!output.status.success(), "unsafe key should fail init");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid DX_PWSH_MENU_KEY"));
}

#[test]
fn init_pwsh_menu_key_is_ignored_without_menu() {
    let output = dx()
        .args(["init", "pwsh"])
        .env("DX_PWSH_MENU_KEY", "F12")
        .output()
        .expect("dx init pwsh should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("Set-PSReadLineKeyHandler"));
    assert!(!stdout.contains("F12"));
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
fn init_pwsh_menu_emits_previous_function_fallback_and_custom_action_warning() {
    let output = dx()
        .args(["init", "pwsh", "--menu"])
        .output()
        .expect("dx init pwsh --menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Get-PSReadLineKeyHandler -Chord $Global:__dx_pwsh_menu_key"));
    assert!(stdout.contains("function global:__dx_pwsh_menu_fallback"));
    assert!(stdout.contains("$Global:__dx_pwsh_menu_handler_description = 'dx menu handler'"));
    assert!(stdout.contains("$dxPreviousMenuKeyVariable = Get-Variable -Name __dx_pwsh_menu_key -Scope Global -ErrorAction SilentlyContinue"));
    assert!(stdout.contains("Remove-PSReadLineKeyHandler -Chord $dxPreviousMenuKey"));
    assert!(
        stdout.contains(
            "$previousHandler.Description -eq $Global:__dx_pwsh_menu_handler_description"
        )
    );
    assert!(stdout.contains("Set-PSReadLineKeyHandler -Key 'Tab'"));
    assert!(stdout.contains(
        "-BriefDescription 'dx menu' -Description $Global:__dx_pwsh_menu_handler_description"
    ));
    assert!(stdout.contains("'MenuComplete' { [Microsoft.PowerShell.PSConsoleReadLine]::MenuComplete($key, $arg); return }"));
    assert!(stdout.contains("'TabCompleteNext' { [Microsoft.PowerShell.PSConsoleReadLine]::TabCompleteNext($key, $arg); return }"));
    assert!(stdout.contains("$Global:__dx_pwsh_menu_previous_function -eq 'CustomAction'"));
    assert!(stdout.contains(
        "dx init: warning: PSReadLine key '$Global:__dx_pwsh_menu_key' was bound to a CustomAction"
    ));
}

#[test]
fn init_pwsh_menu_loads_under_strict_mode() {
    if !pwsh_available() {
        return;
    }

    let output = Command::new("pwsh")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Import-Module PSReadLine; Set-StrictMode -Version Latest; Invoke-Expression ((& $env:CARGO_BIN_EXE_dx init pwsh --menu | Out-String))",
        ])
        .env("PATH", path_with_dx_binary())
        .env("CARGO_BIN_EXE_dx", env!("CARGO_BIN_EXE_dx"))
        .output()
        .expect("pwsh should run");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn init_pwsh_menu_warns_when_evaluated_over_custom_action() {
    if !pwsh_available() {
        return;
    }

    let output = Command::new("pwsh")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Set-PSReadLineKeyHandler -Key F12 -ScriptBlock { param($key, $arg) }; $env:DX_PWSH_MENU_KEY = 'F12'; Invoke-Expression ((& $env:CARGO_BIN_EXE_dx init pwsh --menu | Out-String))",
        ])
        .env("PATH", path_with_dx_binary())
        .env("CARGO_BIN_EXE_dx", env!("CARGO_BIN_EXE_dx"))
        .env("DX_PWSH_MENU_KEY", "F12")
        .output()
        .expect("pwsh should run");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("dx init: warning: PSReadLine key 'F12' was bound to a CustomAction"));
}

#[test]
fn init_pwsh_menu_does_not_warn_when_evaluated_over_menu_complete() {
    if !pwsh_available() {
        return;
    }

    let output = Command::new("pwsh")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Set-PSReadLineKeyHandler -Key F12 -Function MenuComplete; $env:DX_PWSH_MENU_KEY = 'F12'; Invoke-Expression ((& $env:CARGO_BIN_EXE_dx init pwsh --menu | Out-String))",
        ])
        .env("PATH", path_with_dx_binary())
        .env("CARGO_BIN_EXE_dx", env!("CARGO_BIN_EXE_dx"))
        .env("DX_PWSH_MENU_KEY", "F12")
        .output()
        .expect("pwsh should run");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("CustomAction"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn init_pwsh_menu_reload_does_not_warn_over_own_handler() {
    if !pwsh_available() {
        return;
    }

    let output = Command::new("pwsh")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Set-PSReadLineKeyHandler -Key F12 -Function MenuComplete; $env:DX_PWSH_MENU_KEY = 'F12'; $script = (& $env:CARGO_BIN_EXE_dx init pwsh --menu | Out-String); Invoke-Expression $script; Invoke-Expression $script; $h = Get-PSReadLineKeyHandler -Chord F12; \"function=$($Global:__dx_pwsh_menu_previous_function); description=$($h.Description)\"",
        ])
        .env("PATH", path_with_dx_binary())
        .env("CARGO_BIN_EXE_dx", env!("CARGO_BIN_EXE_dx"))
        .env("DX_PWSH_MENU_KEY", "F12")
        .output()
        .expect("pwsh should run");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("CustomAction"),
        "unexpected stderr: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("function=MenuComplete"));
    assert!(stdout.contains("description=dx menu handler"));
}

#[test]
fn init_pwsh_menu_key_change_removes_old_dx_binding() {
    if !pwsh_available() {
        return;
    }

    let output = Command::new("pwsh")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Set-PSReadLineKeyHandler -Key F11 -Function MenuComplete; $env:DX_PWSH_MENU_KEY = 'F11'; Invoke-Expression ((& $env:CARGO_BIN_EXE_dx init pwsh --menu | Out-String)); $env:DX_PWSH_MENU_KEY = 'F12'; Invoke-Expression ((& $env:CARGO_BIN_EXE_dx init pwsh --menu | Out-String)); $old = Get-PSReadLineKeyHandler -Chord F11 -ErrorAction SilentlyContinue; $new = Get-PSReadLineKeyHandler -Chord F12 -ErrorAction SilentlyContinue; \"old=$($old.Function)/$($old.Description); new=$($new.Function)/$($new.Description); previous=$Global:__dx_pwsh_menu_previous_function\"",
        ])
        .env("PATH", path_with_dx_binary())
        .env("CARGO_BIN_EXE_dx", env!("CARGO_BIN_EXE_dx"))
        .output()
        .expect("pwsh should run");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("CustomAction"),
        "unexpected stderr: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("old=MenuComplete/"));
    assert!(stdout.contains("new=dx menu/dx menu handler"));
}

#[test]
fn init_pwsh_uses_idiomatic_functions_and_restores_dot_dot_alias() {
    if !pwsh_available() {
        return;
    }

    let output = Command::new("pwsh")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Set-Alias -Name '..' -Value Get-Location; Invoke-Expression ((& $env:CARGO_BIN_EXE_dx init pwsh | Out-String)); $functions = @('Set-DxLocation', 'Step-Up', 'Undo-Location', 'Redo-Location', 'Set-FrecentLocation', 'Set-RecentLocation'); $functionNames = (Get-Command -Module dx -CommandType Function | Select-Object -ExpandProperty Name); \"functions=$($functionNames -join ',')\"; foreach ($name in @('up', '..', 'back', 'cd-', 'forward', 'cd+', 'cdf', 'z', 'cdr')) { \"$name=$((Get-Alias -Name $name).Definition)\" }; Remove-Module dx; \"dotdotAfterUnload=$((Get-Alias -Name '..').Definition)\"",
        ])
        .env("PATH", path_with_dx_binary())
        .env("CARGO_BIN_EXE_dx", env!("CARGO_BIN_EXE_dx"))
        .output()
        .expect("pwsh should run");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty(), "unexpected stderr: {stderr}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Set-DxLocation"));
    assert!(stdout.contains("Step-Up"));
    assert!(stdout.contains("Undo-Location"));
    assert!(stdout.contains("Redo-Location"));
    assert!(stdout.contains("Set-FrecentLocation"));
    assert!(stdout.contains("Set-RecentLocation"));
    assert!(stdout.contains("up=Step-Up"));
    assert!(stdout.contains("..=Step-Up"));
    assert!(stdout.contains("back=Undo-Location"));
    assert!(stdout.contains("cd-=Undo-Location"));
    assert!(stdout.contains("forward=Redo-Location"));
    assert!(stdout.contains("cd+=Redo-Location"));
    assert!(stdout.contains("cdf=Set-FrecentLocation"));
    assert!(stdout.contains("z=Set-FrecentLocation"));
    assert!(stdout.contains("cdr=Set-RecentLocation"));
    assert!(stdout.contains("dotdotAfterUnload=Get-Location"));
}

#[test]
fn pwsh_set_dx_location_preserves_native_binding_and_filesystem_stack_tracking() {
    if !pwsh_available() {
        return;
    }

    let temp = common::temp_dir("pwsh-location-wrapper");
    let start = temp.path().join("start");
    let destination = temp.path().join("destination");
    let wildcard_parent = temp.path().join("wildcard");
    let wildcard_destination = wildcard_parent.join("only-match");
    fs::create_dir_all(&start).expect("create start directory");
    fs::create_dir_all(&destination).expect("create destination directory");
    fs::create_dir_all(&wildcard_destination).expect("create wildcard destination directory");

    let quote = |path: &std::path::Path| path.display().to_string().replace('\'', "''");
    let missing = temp.path().join("missing");
    let script = format!(
        "Invoke-Expression ((& $env:CARGO_BIN_EXE_dx init pwsh | Out-String)); \
         $nativeHome = (Microsoft.PowerShell.Management\\Set-Location -PassThru).Path; \
         Set-DxLocation; \"home=$($PWD.Path)\"; \"nativeHome=$nativeHome\"; \
         Set-DxLocation -LiteralPath '{start}'; \
         $passThru = Set-DxLocation -LiteralPath '{destination}' -PassThru; \
         \"pass=$($passThru.Path)\"; \
         cd -; \"minus=$($PWD.Path)\"; \
         cd +; \"plus=$($PWD.Path)\"; \
         '{start}' | Set-DxLocation; \"pipe=$($PWD.Path)\"; \
         Set-DxLocation -Path '{wildcard_parent}/*'; \"wildcard=$($PWD.Path)\"; \
         Set-DxLocation -StackName dx-test -PassThru | Out-Null; \
         Set-DxLocation -LiteralPath Env:; \"provider=$((Get-Location).Provider.Name)\"; \
         Set-DxLocation -LiteralPath '{destination}'; \"return=$($PWD.Path)\"; \
         $before = $PWD.Path; $sessionFile = Join-Path $env:XDG_RUNTIME_DIR \"dx-sessions/$env:DX_SESSION.json\"; \
         $beforeStack = Get-Content -Raw $sessionFile; \
         Set-DxLocation -LiteralPath '{missing}' -ErrorAction SilentlyContinue; \
         \"failed=$($PWD.Path -eq $before)\"; \
         \"stackStable=$($beforeStack -eq (Get-Content -Raw $sessionFile))\"; \
         $savedPath = $env:PATH; $env:PATH = ''; \
         $stackFailure = Set-DxLocation -LiteralPath '{start}' -PassThru; \
         $env:PATH = $savedPath; \
         \"stackFailure=$($stackFailure.Path)\"",
        start = quote(&start),
        destination = quote(&destination),
        wildcard_parent = quote(&wildcard_parent),
        missing = quote(&missing),
    );
    let runtime = temp.path().join("runtime");
    fs::create_dir(&runtime).expect("create runtime directory");

    let output = Command::new("pwsh")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .env("PATH", path_with_dx_binary())
        .env("CARGO_BIN_EXE_dx", env!("CARGO_BIN_EXE_dx"))
        .env("DX_SESSION", "pwsh-location-wrapper")
        .env("XDG_RUNTIME_DIR", &runtime)
        .output()
        .expect("pwsh should run");

    assert!(
        output.status.success(),
        "pwsh failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    for (label, expected) in [
        ("pass", &destination),
        ("minus", &start),
        ("plus", &destination),
        ("pipe", &start),
        ("wildcard", &wildcard_destination),
        ("return", &destination),
        ("stackFailure", &start),
    ] {
        let value = stdout
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{label}=")))
            .unwrap_or_else(|| panic!("missing {label} output: {stdout}"));
        common::assert_same_path(value, expected);
    }
    let home = stdout
        .lines()
        .find_map(|line| line.strip_prefix("home="))
        .expect("missing home output");
    let native_home = stdout
        .lines()
        .find_map(|line| line.strip_prefix("nativeHome="))
        .expect("missing native home output");
    common::assert_same_path(home, native_home);
    assert!(
        stdout.contains("provider=Environment"),
        "unexpected stdout: {stdout}"
    );
    assert!(
        stdout.contains("failed=True"),
        "unexpected stdout: {stdout}"
    );
    assert!(
        stdout.contains("stackStable=True"),
        "unexpected stdout: {stdout}"
    );

    let session_file = runtime.join("dx-sessions/pwsh-location-wrapper.json");
    let session: serde_json::Value =
        serde_json::from_slice(&fs::read(&session_file).expect("read PowerShell session stack"))
            .expect("parse PowerShell session stack");
    let cwd = session["cwd"].as_str().expect("session cwd");
    common::assert_same_path(cwd, &destination);
    let entries = session["undo"]
        .as_array()
        .expect("session undo entries")
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert!(
        entries.iter().any(
            |entry| common::canonical(std::path::Path::new(entry)) == common::canonical(&start)
        ),
        "filesystem origin was not recorded: {entries:?}"
    );
    assert!(
        entries.iter().all(|entry| !entry.starts_with("Env:")),
        "provider location was recorded: {entries:?}"
    );
}

#[cfg(unix)]
#[test]
fn pwsh_set_dx_location_ignores_stack_push_spawn_errors() {
    if !pwsh_available() {
        return;
    }

    let temp = common::temp_dir("pwsh-location-wrapper-perms");
    let blocked = temp.path().join("blocked");
    let destination = temp.path().join("destination");
    fs::create_dir_all(&blocked).expect("create blocked directory");
    fs::create_dir_all(&destination).expect("create destination directory");

    let quote = |path: &std::path::Path| path.display().to_string().replace('\'', "''");
    let script = format!(
        "Invoke-Expression ((& $env:CARGO_BIN_EXE_dx init pwsh | Out-String)); \
         Set-DxLocation -LiteralPath '{blocked}'; \
         chmod 600 '{blocked}'; \
         Set-DxLocation -LiteralPath '{destination}'; \
         \"cwd=$($PWD.Path)\"",
        blocked = quote(&blocked),
        destination = quote(&destination),
    );

    let output = Command::new("pwsh")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .env("PATH", path_with_dx_binary())
        .env("CARGO_BIN_EXE_dx", env!("CARGO_BIN_EXE_dx"))
        .env("DX_SESSION", "pwsh-location-wrapper-perms")
        .output()
        .expect("pwsh should run");

    assert!(
        output.status.success(),
        "pwsh failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("ResourceUnavailable"),
        "unexpected stderr: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let cwd = stdout
        .lines()
        .find_map(|line| line.strip_prefix("cwd="))
        .expect("missing cwd output");
    common::assert_same_path(cwd, &destination);
}

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

#[test]
fn pwsh_menu_redraw_helpers_handle_wrapping_multiline_and_invalid_geometry() {
    if !pwsh_available() {
        return;
    }

    let script = r#"
Import-Module PSReadLine
Invoke-Expression ((& $env:CARGO_BIN_EXE_dx init pwsh --menu | Out-String))
Set-PSReadLineOption -PromptText @('> ') -ContinuationPrompt '>> ' -ExtraPromptLineCount 0

function New-TestRawUi([int]$x, [int]$y, [int]$width) {
    $rawUi = [pscustomobject]@{
        CursorPosition = [pscustomobject]@{ X = $x; Y = $y }
        WindowPosition = [pscustomobject]@{ X = 0; Y = 0 }
        WindowSize = [pscustomobject]@{ Width = $width; Height = 24 }
        BufferSize = [pscustomobject]@{ Width = $width; Height = 24 }
    }
    Add-Member -InputObject $rawUi -MemberType ScriptMethod -Name LengthInBufferCells -Value {
        param([string]$Text)
        $cells = 0
        foreach ($character in $Text.ToCharArray()) {
            if ([int]$character -eq 0x754c) { $cells += 2 } else { $cells += 1 }
        }
        return $cells
    }
    return $rawUi
}

function Write-Origin([string]$name, [string]$line, [int]$cursor, [int]$x, [int]$y) {
    $context = __dx_pwsh_capture_redraw_context -Line $line -Cursor $cursor -RawUi (New-TestRawUi $x $y 10)
    "$name=$($context.PromptTopY),$($context.RelativeCursorY)"
}

Write-Origin one 'abc' 3 5 20
Write-Origin wrapped 'abcdefghij' 10 2 20
Write-Origin middle 'abc' 1 3 20
Write-Origin wide '界a' 2 5 20
Write-Origin multiline "abc`nde" 6 5 20
Set-PSReadLineOption -ExtraPromptLineCount 1
Write-Origin extra "abc`nde" 6 5 20

$context = [pscustomobject]@{
    RelativeCursorY = 23
    PromptTopY = 23
    WindowY = 0
    WindowHeight = 24
    BufferHeight = 24
}
"valid=$(__dx_pwsh_resolve_redraw_y ([pscustomobject]@{ redrawRow = 13; scrollRows = 10 }) $context)"
"fraction=$(__dx_pwsh_resolve_redraw_y ([pscustomobject]@{ redrawRow = 13.5; scrollRows = 10 }) $context)"
"mismatch=$(__dx_pwsh_resolve_redraw_y ([pscustomobject]@{ redrawRow = 12; scrollRows = 10 }) $context)"
"negative=$(__dx_pwsh_resolve_redraw_y ([pscustomobject]@{ redrawRow = -1; scrollRows = 10 }) $context)"
"#;

    let output = Command::new("pwsh")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .env("PATH", path_with_dx_binary())
        .env("CARGO_BIN_EXE_dx", env!("CARGO_BIN_EXE_dx"))
        .output()
        .expect("pwsh should run");

    assert!(
        output.status.success(),
        "pwsh failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "one=20,20",
        "wrapped=19,20",
        "middle=20,20",
        "wide=20,20",
        "multiline=19,20",
        "extra=18,20",
        "valid=13",
        "fraction=",
        "mismatch=",
        "negative=",
    ] {
        assert!(
            stdout.lines().any(|line| line == expected),
            "missing {expected:?} in stdout: {stdout}"
        );
    }
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
fn init_pwsh_menu_with_mappings_emits_shared_handler_mode_routing() {
    let output = dx()
        .args(["init", "pwsh", "--menu"])
        .env("DX_MENU_COMMAND_MAPPINGS", "cat=file")
        .output()
        .expect("dx init pwsh --menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("$dxMappingSeeds = @('cat=file')"));
    assert!(stdout.contains(
        "foreach ($alias in Get-Alias -Definition $command -ErrorAction SilentlyContinue)"
    ));
    assert!(stdout.contains("$Global:__dx_pwsh_menu_mapped = @{}"));
    assert!(stdout.contains("$dxMapped = $Global:__dx_pwsh_menu_mapped"));
    assert!(stdout.contains("if ($dxMapped -and $dxMapped.ContainsKey($first))"));
    assert!(stdout.contains("($first -notin $dxCmds -and -not $dxMenuMode)"));
    assert!(
        stdout.contains("dx menu --shell pwsh --mode $dxMenuMode --buffer $line --cursor $cursor")
    );
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

#[test]
fn pwsh_shell_keeps_posix_flagged_cd_as_fallback() {
    let buffer = "cd -P foo";
    let output = dx()
        .args([
            "menu",
            "--buffer",
            buffer,
            "--cursor",
            &buffer.len().to_string(),
            "--shell",
            "pwsh",
        ])
        .env("DX_MENU_DEBUG", "1")
        .output()
        .expect("dx menu should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert_eq!(parsed, serde_json::json!({ "action": "noop" }));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("parse_buffer returned None -> noop"),
        "expected fallback parse path in psreadline mode; stderr: {stderr}"
    );
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
