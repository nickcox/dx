use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn dx_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_dx"))
}

fn make_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("dx-it-{label}-{nonce}-{}", std::process::id()));
    fs::create_dir_all(&path).expect("create temp dir");
    path
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
            command.arg("-lc");
        }
        "zsh" => {
            command.arg("-fc");
        }
        _ => panic!("unsupported shell: {shell}"),
    }

    command
        .arg(script)
        .current_dir(cwd)
        .output()
        .expect("run shell script")
}

#[test]
fn bash_generated_hook_command_not_found_guard_prevents_recursive_resolve_calls() {
    let temp = make_temp_dir("hook-guard");
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
    let _ = fs::remove_dir_all(temp);
}

#[test]
fn bash_generated_hook_command_not_found_resolves_path_like_command_once() {
    let temp = make_temp_dir("hook-resolve");
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
    let _ = fs::remove_dir_all(temp);
}

#[test]
fn bash_generated_hook_cd_wrapper_invokes_dx_once_and_changes_directory() {
    let temp = make_temp_dir("hook-cd");
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
    let _ = fs::remove_dir_all(temp);
}

#[test]
fn zsh_generated_hook_command_not_found_guard_prevents_recursive_resolve_calls() {
    let temp = make_temp_dir("zsh-hook-guard");
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
    let _ = fs::remove_dir_all(temp);
}

#[test]
fn zsh_generated_hook_command_not_found_resolves_path_like_command_once() {
    let temp = make_temp_dir("zsh-hook-resolve");
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
    let _ = fs::remove_dir_all(temp);
}
