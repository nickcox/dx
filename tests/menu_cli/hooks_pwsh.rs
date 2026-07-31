//! Generated PowerShell hook content: PSReadLine key handling, strict mode,
//! redraw helpers and `Set-DxLocation`.

use std::fs;
#[cfg(not(unix))]
use std::process::Command;
#[cfg(unix)]
use std::process::Command;

use super::common;
use super::support::*;

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
