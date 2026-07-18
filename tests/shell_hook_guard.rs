mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn dx_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_dx"))
}

fn optional_tool_available(command: &str) -> bool {
    if common::tool_available(command) {
        return true;
    }

    let diagnostic =
        format!("{command} is required for this external-shell test but is unavailable");
    if std::env::var_os("CI").is_some() || std::env::var_os("DX_REQUIRE_EXTERNAL_TOOLS").is_some() {
        panic!("{diagnostic}");
    }

    eprintln!("skipping external-shell test: {diagnostic}");
    false
}

fn generated_hook_script(shell: &str) -> String {
    let output = Command::new(dx_bin())
        .args(["init", shell, "--command-not-found"])
        .output()
        .expect("run dx init with command-not-found");
    assert!(output.status.success());
    String::from_utf8(output.stdout).expect("generated hook output utf8")
}

fn write_generated_hook(temp: &Path, shell: &str) -> PathBuf {
    let hook_path = temp.join(format!("dx-generated-{shell}.sh"));
    fs::write(&hook_path, generated_hook_script(shell)).expect("write generated hook script");
    hook_path
}

fn run_shell(shell: &str, script: &str, cwd: &Path) -> Output {
    let mut command = Command::new(shell);
    match shell {
        "bash" => {
            command.args(["--noprofile", "--norc", "-c"]);
        }
        "zsh" => {
            command.args(["-f", "-c"]);
        }
        _ => panic!("unsupported shell: {shell}"),
    }

    command
        .arg(script)
        .current_dir(cwd)
        .env("HOME", cwd)
        .env("XDG_CONFIG_HOME", cwd)
        .env("ZDOTDIR", cwd)
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .env_remove("BASH_ENV")
        .env_remove("ENV")
        .output()
        .expect("run shell script")
}

#[test]
fn bash_generated_hook_command_not_found_guard_prevents_recursive_resolve_calls() {
    let temp_dir = common::temp_dir("hook-guard");
    let temp = temp_dir.path();
    let hook = write_generated_hook(&temp, "bash");
    let script = format!(
        "source \"{hook}\"; __dx_calls=0; dx() {{ __dx_calls=$((__dx_calls+1)); return 0; }}; DX_RESOLVE_GUARD=1; command_not_found_handle \"./foo\" >/dev/null 2>&1; status=$?; printf '%s:%s' \"$status\" \"$__dx_calls\"",
        hook = hook.display()
    );

    let output = run_shell("bash", &script, &temp);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut parts = stdout.trim().splitn(2, ':');
    let status = parts.next().expect("status part");
    let calls = parts.next().expect("calls part");
    assert_eq!(status, "127");
    assert_eq!(calls, "0");
}

#[test]
fn bash_generated_hook_command_not_found_resolves_path_like_command_once() {
    let temp_dir = common::temp_dir("hook-resolve");
    let temp = temp_dir.path();
    let target = temp.join("target");
    let hook = write_generated_hook(&temp, "bash");
    fs::create_dir_all(&target).expect("create target");

    let script = format!(
        "source \"{hook}\"; __dx_calls=0; dx() {{ __dx_calls=$((__dx_calls+1)); if [[ \"$1\" == resolve ]]; then printf '%s\\n' \"{target}\"; return 0; fi; return 1; }}; command_not_found_handle \"./target\" >/dev/null 2>&1; status=$?; printf '%s:%s:%s' \"$status\" \"$PWD\" \"$__dx_calls\"",
        hook = hook.display(),
        target = target.display(),
    );

    let output = run_shell("bash", &script, &temp);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    let mut parts = trimmed.splitn(3, ':');
    let status = parts.next().expect("status part");
    let pwd = parts.next().expect("pwd part");
    let calls = parts.next().expect("calls part");
    assert_eq!(status, "0");
    assert_eq!(calls, "1");

    let expected = fs::canonicalize(&target)
        .expect("canonical target")
        .display()
        .to_string();
    let actual = fs::canonicalize(pwd)
        .expect("canonical actual")
        .display()
        .to_string();
    assert_eq!(actual, expected);
}

#[test]
fn bash_generated_hook_cd_wrapper_invokes_dx_once_and_changes_directory() {
    let temp_dir = common::temp_dir("hook-cd");
    let temp = temp_dir.path();
    let target = temp.join("project");
    let marker = temp.join("dx-called.log");
    let hook = write_generated_hook(&temp, "bash");
    fs::create_dir_all(&target).expect("create target");

    let script = format!(
        "source \"{hook}\"; dx() {{ printf '%s\\n' \"$1\" >> \"{marker}\"; if [[ \"$1\" == resolve ]]; then printf '%s\\n' \"{target}\"; return 0; fi; return 0; }}; cd \"project\" >/dev/null 2>&1; status=$?; printf '%s:%s' \"$status\" \"$PWD\"",
        hook = hook.display(),
        marker = marker.display(),
        target = target.display(),
    );

    let output = run_shell("bash", &script, &temp);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut parts = stdout.trim().splitn(2, ':');
    let status = parts.next().expect("status part");
    let pwd = parts.next().expect("pwd part");
    assert_eq!(status, "0");

    let expected = fs::canonicalize(&target)
        .expect("canonical target")
        .display()
        .to_string();
    let actual = fs::canonicalize(pwd)
        .expect("canonical actual")
        .display()
        .to_string();
    assert_eq!(actual, expected);

    let log = fs::read_to_string(&marker).expect("read dx call log");
    let resolve_calls = log.lines().filter(|line| *line == "resolve").count();
    assert_eq!(resolve_calls, 1, "cd wrapper should resolve exactly once");
}

#[test]
fn zsh_generated_hook_command_not_found_guard_prevents_recursive_resolve_calls() {
    if !optional_tool_available("zsh") {
        return;
    }
    let temp_dir = common::temp_dir("zsh-hook-guard");
    let temp = temp_dir.path();
    let hook = write_generated_hook(&temp, "zsh");
    let script = format!(
        "function compdef() {{ :; }}; source \"{hook}\"; __dx_calls=0; function dx() {{ __dx_calls=$((__dx_calls+1)); return 0; }}; DX_RESOLVE_GUARD=1; command_not_found_handler \"./foo\" >/dev/null 2>&1; rc=$?; printf '%s:%s' \"$rc\" \"$__dx_calls\"",
        hook = hook.display()
    );

    let output = run_shell("zsh", &script, &temp);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut parts = stdout.trim().splitn(2, ':');
    let status = parts.next().expect("status part");
    let calls = parts.next().expect("calls part");
    assert_eq!(status, "127");
    assert_eq!(calls, "0");
}

#[test]
fn zsh_generated_hook_command_not_found_resolves_path_like_command_once() {
    if !optional_tool_available("zsh") {
        return;
    }
    let temp_dir = common::temp_dir("zsh-hook-resolve");
    let temp = temp_dir.path();
    let target = temp.join("target");
    let hook = write_generated_hook(&temp, "zsh");
    fs::create_dir_all(&target).expect("create target");

    let script = format!(
        "function compdef() {{ :; }}; source \"{hook}\"; __dx_calls=0; function dx() {{ __dx_calls=$((__dx_calls+1)); if [[ \"$1\" == \"resolve\" ]]; then print -r -- \"{target}\"; return 0; fi; return 1; }}; command_not_found_handler \"./target\" >/dev/null 2>&1; rc=$?; printf '%s:%s:%s' \"$rc\" \"$PWD\" \"$__dx_calls\"",
        hook = hook.display(),
        target = target.display(),
    );

    let output = run_shell("zsh", &script, &temp);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut parts = stdout.trim().splitn(3, ':');
    let status = parts.next().expect("status part");
    let pwd = parts.next().expect("pwd part");
    let calls = parts.next().expect("calls part");
    assert_eq!(status, "0");
    assert_eq!(calls, "1");

    let expected = fs::canonicalize(&target)
        .expect("canonical target")
        .display()
        .to_string();
    let actual = fs::canonicalize(pwd)
        .expect("canonical actual")
        .display()
        .to_string();
    assert_eq!(actual, expected);
}

#[test]
fn bash_generated_hook_command_not_found_resolves_delimiter_shortened_command_once() {
    let temp_dir = common::temp_dir("hook-resolve-delimiter");
    let temp = temp_dir.path();
    let target = temp.join("cd-extras");
    let hook = write_generated_hook(&temp, "bash");
    fs::create_dir_all(&target).expect("create target");

    let script = format!(
        "source \"{hook}\"; __dx_calls=0; dx() {{ __dx_calls=$((__dx_calls+1)); if [[ \"$1\" == resolve ]]; then printf '%s\\n' \"{target}\"; return 0; fi; return 1; }}; command_not_found_handle \"cd-e\" >/dev/null 2>&1; status=$?; printf '%s:%s:%s' \"$status\" \"$PWD\" \"$__dx_calls\"",
        hook = hook.display(),
        target = target.display(),
    );

    let output = run_shell("bash", &script, &temp);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut parts = stdout.trim().splitn(3, ':');
    let status = parts.next().expect("status part");
    let pwd = parts.next().expect("pwd part");
    let calls = parts.next().expect("calls part");
    assert_eq!(status, "0");
    assert_eq!(calls, "1");

    let expected = fs::canonicalize(&target)
        .expect("canonical target")
        .display()
        .to_string();
    let actual = fs::canonicalize(pwd)
        .expect("canonical actual")
        .display()
        .to_string();
    assert_eq!(actual, expected);
}

#[test]
fn bash_generated_hook_command_not_found_plain_word_still_falls_through() {
    let temp_dir = common::temp_dir("hook-plain-word-fallthrough");
    let temp = temp_dir.path();
    let hook = write_generated_hook(&temp, "bash");
    let script = format!(
        "source \"{hook}\"; __dx_calls=0; dx() {{ __dx_calls=$((__dx_calls+1)); return 0; }}; command_not_found_handle \"gti\" >/dev/null 2>&1; status=$?; printf '%s:%s' \"$status\" \"$__dx_calls\"",
        hook = hook.display(),
    );

    let output = run_shell("bash", &script, &temp);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut parts = stdout.trim().splitn(2, ':');
    let status = parts.next().expect("status part");
    let calls = parts.next().expect("calls part");
    assert_eq!(status, "127");
    assert_eq!(calls, "0");
}

#[test]
fn zsh_generated_hook_command_not_found_resolves_doubled_period_command_once() {
    if !optional_tool_available("zsh") {
        return;
    }
    let temp_dir = common::temp_dir("zsh-hook-resolve-gap");
    let temp = temp_dir.path();
    let target = temp.join("PowerShell");
    let hook = write_generated_hook(&temp, "zsh");
    fs::create_dir_all(&target).expect("create target");

    let script = format!(
        "function compdef() {{ :; }}; source \"{hook}\"; __dx_calls=0; function dx() {{ __dx_calls=$((__dx_calls+1)); if [[ \"$1\" == \"resolve\" ]]; then print -r -- \"{target}\"; return 0; fi; return 1; }}; command_not_found_handler \"p..shell\" >/dev/null 2>&1; rc=$?; printf '%s:%s:%s' \"$rc\" \"$PWD\" \"$__dx_calls\"",
        hook = hook.display(),
        target = target.display(),
    );

    let output = run_shell("zsh", &script, &temp);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut parts = stdout.trim().splitn(3, ':');
    let status = parts.next().expect("status part");
    let pwd = parts.next().expect("pwd part");
    let calls = parts.next().expect("calls part");
    assert_eq!(status, "0");
    assert_eq!(calls, "1");

    let expected = fs::canonicalize(&target)
        .expect("canonical target")
        .display()
        .to_string();
    let actual = fs::canonicalize(pwd)
        .expect("canonical actual")
        .display()
        .to_string();
    assert_eq!(actual, expected);
}

#[test]
fn fish_generated_hook_command_not_found_respects_literal_directory_auto_cd_behavior() {
    if !optional_tool_available("fish") {
        return;
    }
    let temp_dir = common::temp_dir("fish-hook-literal-dir");
    let temp = temp_dir.path();
    let target = temp.join("literal-dir");
    fs::create_dir_all(&target).expect("create target");

    let output = Command::new("fish")
        .arg("--no-config")
        .arg("-c")
        .arg("if test -d literal-dir; cd literal-dir; end; pwd")
        .current_dir(temp)
        .env("HOME", temp)
        .env("XDG_CONFIG_HOME", temp)
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .env_remove("BASH_ENV")
        .env_remove("ENV")
        .output()
        .expect("run fish literal dir script");

    assert!(output.status.success());
    let actual = fs::canonicalize(String::from_utf8_lossy(&output.stdout).trim())
        .expect("canonical actual")
        .display()
        .to_string();
    let expected = fs::canonicalize(&target)
        .expect("canonical target")
        .display()
        .to_string();
    assert_eq!(actual, expected);
}

#[test]
fn fish_generated_hook_command_not_found_resolves_delimiter_shortened_command_once() {
    if !optional_tool_available("fish") {
        return;
    }
    let temp_dir = common::temp_dir("fish-hook-resolve-delimiter");
    let temp = temp_dir.path();
    let target = temp.join("cd-extras");
    let hook = write_generated_hook(&temp, "fish");
    fs::create_dir_all(&target).expect("create target");

    let script = format!(
        "source \"{hook}\"; set -g __dx_resolve_calls 0; function dx; if test \"$argv[1]\" = resolve; set -g __dx_resolve_calls (math $__dx_resolve_calls + 1); printf '%s\\n' \"{target}\"; return 0; end; return 0; end; fish_command_not_found cd-e >/dev/null 2>&1; set status_value $status; printf '%s:%s:%s' $status_value $PWD $__dx_resolve_calls",
        hook = hook.display(),
        target = target.display(),
    );

    let output = Command::new("fish")
        .arg("--no-config")
        .arg("-c")
        .arg(&script)
        .current_dir(temp)
        .env("HOME", temp)
        .env("XDG_CONFIG_HOME", temp)
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .env_remove("BASH_ENV")
        .env_remove("ENV")
        .output()
        .expect("run fish delimiter-shortened hook script");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut parts = stdout.trim().splitn(3, ':');
    let status = parts.next().expect("status part");
    let pwd = parts.next().expect("pwd part");
    let calls = parts.next().expect("calls part");
    assert_eq!(status, "0");
    assert_eq!(calls, "1");

    let expected = fs::canonicalize(&target)
        .expect("canonical target")
        .display()
        .to_string();
    let actual = fs::canonicalize(pwd)
        .expect("canonical actual")
        .display()
        .to_string();
    assert_eq!(actual, expected);
}

#[test]
fn zsh_generated_hook_cd_permission_denied_error_does_not_leak_helper_name() {
    if !optional_tool_available("zsh") {
        return;
    }
    let temp_dir = common::temp_dir("zsh-hook-cd-perm-denied");
    let temp = temp_dir.path();
    let blocked = temp.join("blocked");
    let hook = write_generated_hook(&temp, "zsh");
    fs::create_dir_all(&blocked).expect("create blocked dir");
    #[cfg(unix)]
    let _permission_guard = common::PermissionGuard::new(&blocked);

    let mut perms = fs::metadata(&blocked)
        .expect("blocked metadata")
        .permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o000);
    }
    fs::set_permissions(&blocked, perms).expect("set blocked perms");

    if fs::read_dir(&blocked).is_ok() {
        eprintln!(
            "skipping permission-denied assertion: elevated privileges can access chmod 000 directory {}",
            blocked.display()
        );
        return;
    }

    let script = format!(
        "function compdef() {{ :; }}; source \"{hook}\"; cd \"{blocked}\" >/dev/null; printf '%s' \"$?\"",
        hook = hook.display(),
        blocked = blocked.display(),
    );

    let output = run_shell("zsh", &script, &temp);

    assert!(output.status.success());
    let status = String::from_utf8_lossy(&output.stdout);
    assert_ne!(status.trim(), "0");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cd:"),
        "expected native cd prefix in stderr, got: {stderr}"
    );
    assert!(
        stderr.contains("permission denied"),
        "expected permission denied in stderr, got: {stderr}"
    );
    assert!(
        !stderr.contains("__dx_cd_native"),
        "stderr should not leak helper name, got: {stderr}"
    );
}
