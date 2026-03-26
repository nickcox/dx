use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn make_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("dx-it-{label}-{nonce}-{}", std::process::id()));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

#[test]
fn bash_hook_guard_prevents_recursive_resolve_calls() {
    let temp = make_temp_dir("hook-guard");
    let marker = temp.join("dx-called");
    let script = format!(
        "source \"{hook}\"; dx() {{ : > \"{marker}\"; return 0; }}; DX_RESOLVE_GUARD=1; command_not_found_handle \"./foo\" >/dev/null 2>&1; status=$?; printf '%s' \"$status\"",
        hook = "/Users/nick/code/personal/dx/scripts/hooks/dx.bash",
        marker = marker.display()
    );

    let output = Command::new("bash")
        .arg("-lc")
        .arg(script)
        .current_dir(&temp)
        .output()
        .expect("run bash");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "127");
    assert!(!marker.exists());
    let _ = fs::remove_dir_all(temp);
}

#[test]
fn bash_hook_resolves_path_like_command_once() {
    let temp = make_temp_dir("hook-resolve");
    let target = temp.join("target");
    let marker = temp.join("dx-called");
    fs::create_dir_all(&target).expect("create target");

    let script = format!(
        "source \"{hook}\"; dx() {{ : > \"{marker}\"; if [[ \"$1\" == resolve ]]; then printf '%s\\n' \"{target}\"; return 0; fi; return 1; }}; command_not_found_handle \"./target\" >/dev/null 2>&1; status=$?; printf '%s:%s' \"$status\" \"$PWD\"",
        hook = "/Users/nick/code/personal/dx/scripts/hooks/dx.bash",
        target = target.display(),
        marker = marker.display()
    );

    let output = Command::new("bash")
        .arg("-lc")
        .arg(script)
        .current_dir(&temp)
        .output()
        .expect("run bash");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    let mut parts = trimmed.splitn(2, ':');
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
    assert!(marker.exists());
    let _ = fs::remove_dir_all(temp);
}

#[test]
fn bash_cd_wrapper_invokes_dx_once_and_changes_directory() {
    let temp = make_temp_dir("hook-cd");
    let target = temp.join("project");
    let marker = temp.join("dx-called");
    fs::create_dir_all(&target).expect("create target");

    let script = format!(
        "source \"{hook}\"; dx() {{ : > \"{marker}\"; if [[ \"$1\" == resolve ]]; then printf '%s\\n' \"{target}\"; return 0; fi; return 1; }}; cd \"project\" >/dev/null 2>&1; status=$?; printf '%s:%s' \"$status\" \"$PWD\"",
        hook = "/Users/nick/code/personal/dx/scripts/hooks/dx.bash",
        target = target.display(),
        marker = marker.display()
    );

    let output = Command::new("bash")
        .arg("-lc")
        .arg(script)
        .current_dir(&temp)
        .output()
        .expect("run bash");

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
    assert!(marker.exists());
    let _ = fs::remove_dir_all(temp);
}
