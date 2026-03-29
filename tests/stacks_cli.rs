use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use dx::stacks::SessionStack;

fn make_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("dx-it-{label}-{nonce}-{}", std::process::id()));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

fn dx_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_dx"))
}

fn canonical(path: &Path) -> PathBuf {
    fs::canonicalize(path).expect("canonical path")
}

fn read_session(path: &Path) -> SessionStack {
    let raw = fs::read_to_string(path).expect("read session file");
    serde_json::from_str::<SessionStack>(&raw).expect("parse session json")
}

#[test]
fn full_push_undo_redo_push_cycle_updates_session_file() {
    let temp = make_temp_dir("stacks-cycle");
    let runtime = temp.join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let a = temp.join("a");
    let b = temp.join("b");
    let d = temp.join("d");
    fs::create_dir_all(&a).expect("create a");
    fs::create_dir_all(&b).expect("create b");
    fs::create_dir_all(&d).expect("create d");

    let a = canonical(&a);
    let b = canonical(&b);
    let d = canonical(&d);

    let push_a = Command::new(dx_bin())
        .args(["push", a.to_str().expect("utf8 path"), "--session", "s1"])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .current_dir(&temp)
        .output()
        .expect("push a");
    assert!(push_a.status.success());
    assert_eq!(
        String::from_utf8_lossy(&push_a.stdout).trim(),
        a.display().to_string()
    );

    let push_b = Command::new(dx_bin())
        .args(["push", b.to_str().expect("utf8 path"), "--session", "s1"])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .current_dir(&temp)
        .output()
        .expect("push b");
    assert!(push_b.status.success());
    assert_eq!(
        String::from_utf8_lossy(&push_b.stdout).trim(),
        b.display().to_string()
    );

    let undo = Command::new(dx_bin())
        .args(["undo", "--session", "s1"])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .current_dir(&temp)
        .output()
        .expect("undo");
    assert!(undo.status.success());
    assert_eq!(
        String::from_utf8_lossy(&undo.stdout).trim(),
        a.display().to_string()
    );

    let redo = Command::new(dx_bin())
        .args(["redo", "--session", "s1"])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .current_dir(&temp)
        .output()
        .expect("redo");
    assert!(redo.status.success());
    assert_eq!(
        String::from_utf8_lossy(&redo.stdout).trim(),
        b.display().to_string()
    );

    let push_d = Command::new(dx_bin())
        .args(["push", d.to_str().expect("utf8 path"), "--session", "s1"])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .current_dir(&temp)
        .output()
        .expect("push d");
    assert!(push_d.status.success());
    assert_eq!(
        String::from_utf8_lossy(&push_d.stdout).trim(),
        d.display().to_string()
    );

    let state = read_session(&runtime.join("dx-sessions").join("s1.json"));
    assert_eq!(state.cwd, Some(d));
    assert_eq!(state.undo, vec![a, b]);
    assert!(state.redo.is_empty());

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn pop_with_history_succeeds_then_empty_pop_fails() {
    let temp = make_temp_dir("stacks-pop");
    let runtime = temp.join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let a = temp.join("a");
    let b = temp.join("b");
    fs::create_dir_all(&a).expect("create a");
    fs::create_dir_all(&b).expect("create b");

    let a = canonical(&a);
    let b = canonical(&b);

    let _ = Command::new(dx_bin())
        .args(["push", a.to_str().expect("utf8 path"), "--session", "s2"])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .current_dir(&temp)
        .output()
        .expect("push a");
    let _ = Command::new(dx_bin())
        .args(["push", b.to_str().expect("utf8 path"), "--session", "s2"])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .current_dir(&temp)
        .output()
        .expect("push b");

    let pop_ok = Command::new(dx_bin())
        .args(["pop", "--session", "s2"])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .current_dir(&temp)
        .output()
        .expect("pop ok");
    assert!(pop_ok.status.success());
    assert_eq!(
        String::from_utf8_lossy(&pop_ok.stdout).trim(),
        a.display().to_string()
    );

    let pop_fail = Command::new(dx_bin())
        .args(["pop", "--session", "s2"])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .current_dir(&temp)
        .output()
        .expect("pop fail");
    assert!(!pop_fail.status.success());
    assert!(String::from_utf8_lossy(&pop_fail.stdout).trim().is_empty());
    assert!(String::from_utf8_lossy(&pop_fail.stderr).contains("nothing to pop"));

    let state = read_session(&runtime.join("dx-sessions").join("s2.json"));
    assert_eq!(state.cwd, Some(a));
    assert!(state.undo.is_empty());
    assert!(state.redo.is_empty());

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn missing_session_id_returns_error() {
    let temp = make_temp_dir("stacks-missing-session");
    let runtime = temp.join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let target = temp.join("target");
    fs::create_dir_all(&target).expect("create target");
    let target = canonical(&target);

    let output = Command::new(dx_bin())
        .args(["push", target.to_str().expect("utf8 path")])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .current_dir(&temp)
        .output()
        .expect("run dx push");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing session id"));

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn dx_session_env_is_used_and_cli_flag_overrides() {
    let temp = make_temp_dir("stacks-session-source");
    let runtime = temp.join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let env_dir = temp.join("env-dir");
    let cli_dir = temp.join("cli-dir");
    fs::create_dir_all(&env_dir).expect("create env dir");
    fs::create_dir_all(&cli_dir).expect("create cli dir");

    let env_dir = canonical(&env_dir);
    let cli_dir = canonical(&cli_dir);

    let by_env = Command::new(dx_bin())
        .args(["push", env_dir.to_str().expect("utf8 path")])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env("DX_SESSION", "env-session")
        .current_dir(&temp)
        .output()
        .expect("push by env session");
    assert!(by_env.status.success());

    let by_cli = Command::new(dx_bin())
        .args([
            "push",
            cli_dir.to_str().expect("utf8 path"),
            "--session",
            "cli-session",
        ])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env("DX_SESSION", "env-session")
        .current_dir(&temp)
        .output()
        .expect("push by cli session");
    assert!(by_cli.status.success());

    let env_state = read_session(&runtime.join("dx-sessions").join("env-session.json"));
    let cli_state = read_session(&runtime.join("dx-sessions").join("cli-session.json"));

    assert_eq!(env_state.cwd, Some(env_dir));
    assert_eq!(cli_state.cwd, Some(cli_dir));

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn session_directory_is_auto_created_with_temp_fallback() {
    let temp = make_temp_dir("stacks-temp-fallback");
    let temp_root = temp.join("temp-root");
    fs::create_dir_all(&temp_root).expect("create temp root");

    let target = temp.join("target");
    fs::create_dir_all(&target).expect("create target");
    let target = canonical(&target);

    let output = Command::new(dx_bin())
        .args([
            "push",
            target.to_str().expect("utf8 path"),
            "--session",
            "temp-fallback",
        ])
        .env_remove("XDG_RUNTIME_DIR")
        .env("TMPDIR", temp_root.display().to_string())
        .env("TEMP", temp_root.display().to_string())
        .env("TMP", temp_root.display().to_string())
        .env_remove("DX_SESSION")
        .current_dir(&temp)
        .output()
        .expect("push with temp fallback");

    assert!(output.status.success());

    let expected_dir = temp_root.join("dx-sessions");
    let expected_file = expected_dir.join("temp-fallback.json");
    assert!(expected_dir.exists());
    assert!(expected_file.exists());

    let expected_canon = canonical(&expected_dir);
    let actual_canon = canonical(expected_file.parent().expect("session file parent"));
    assert_eq!(actual_canon, expected_canon);

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn undo_with_target_jumps_multiple_entries() {
    let temp = make_temp_dir("stacks-undo-target");
    let runtime = temp.join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let a = temp.join("a");
    let b = temp.join("b");
    let c = temp.join("c");
    let d = temp.join("d");
    for dir in [&a, &b, &c, &d] {
        fs::create_dir_all(dir).expect("create dir");
    }
    let a = canonical(&a);
    let b = canonical(&b);
    let c = canonical(&c);
    let d = canonical(&d);

    // push a -> b -> c -> d
    for dir in [&a, &b, &c, &d] {
        let out = Command::new(dx_bin())
            .args(["push", dir.to_str().unwrap(), "--session", "target1"])
            .env("XDG_RUNTIME_DIR", runtime.display().to_string())
            .env_remove("DX_SESSION")
            .output()
            .unwrap();
        assert!(out.status.success());
    }

    // undo --target a (should consume c, b, reach a)
    let undo = Command::new(dx_bin())
        .args([
            "undo",
            "--session",
            "target1",
            "--target",
            a.to_str().unwrap(),
        ])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .output()
        .unwrap();
    assert!(undo.status.success());
    assert_eq!(
        String::from_utf8_lossy(&undo.stdout).trim(),
        a.display().to_string()
    );

    // verify session state: cwd=a, undo=[], redo=[d, c, b] (each undo pushes old cwd onto redo)
    let state = read_session(&runtime.join("dx-sessions").join("target1.json"));
    assert_eq!(state.cwd, Some(a));
    assert!(state.undo.is_empty());
    assert_eq!(state.redo, vec![d.clone(), c.clone(), b.clone()]);

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn redo_with_target_jumps_multiple_entries() {
    let temp = make_temp_dir("stacks-redo-target");
    let runtime = temp.join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let a = temp.join("a");
    let b = temp.join("b");
    let c = temp.join("c");
    for dir in [&a, &b, &c] {
        fs::create_dir_all(dir).expect("create dir");
    }
    let a = canonical(&a);
    let b = canonical(&b);
    let c = canonical(&c);

    // push a -> b -> c
    for dir in [&a, &b, &c] {
        let out = Command::new(dx_bin())
            .args(["push", dir.to_str().unwrap(), "--session", "target2"])
            .env("XDG_RUNTIME_DIR", runtime.display().to_string())
            .env_remove("DX_SESSION")
            .output()
            .unwrap();
        assert!(out.status.success());
    }

    // undo --target a (go back to beginning)
    let _ = Command::new(dx_bin())
        .args([
            "undo",
            "--session",
            "target2",
            "--target",
            a.to_str().unwrap(),
        ])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .output()
        .unwrap();

    // redo --target c (skip b, jump to c)
    let redo = Command::new(dx_bin())
        .args([
            "redo",
            "--session",
            "target2",
            "--target",
            c.to_str().unwrap(),
        ])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .output()
        .unwrap();
    assert!(redo.status.success());
    assert_eq!(
        String::from_utf8_lossy(&redo.stdout).trim(),
        c.display().to_string()
    );

    // verify: cwd=c, undo=[a, b], redo=[]
    let state = read_session(&runtime.join("dx-sessions").join("target2.json"));
    assert_eq!(state.cwd, Some(c));
    assert_eq!(state.undo, vec![a, b]);
    assert!(state.redo.is_empty());

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn undo_with_unreachable_target_fails() {
    let temp = make_temp_dir("stacks-undo-unreachable");
    let runtime = temp.join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let a = temp.join("a");
    let b = temp.join("b");
    for dir in [&a, &b] {
        fs::create_dir_all(dir).expect("create dir");
    }
    let a = canonical(&a);
    let b = canonical(&b);

    // push a -> b
    for dir in [&a, &b] {
        let out = Command::new(dx_bin())
            .args(["push", dir.to_str().unwrap(), "--session", "target3"])
            .env("XDG_RUNTIME_DIR", runtime.display().to_string())
            .env_remove("DX_SESSION")
            .output()
            .unwrap();
        assert!(out.status.success());
    }

    // undo --target /nonexistent should fail
    let undo = Command::new(dx_bin())
        .args([
            "undo",
            "--session",
            "target3",
            "--target",
            "/nonexistent/path",
        ])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .output()
        .unwrap();
    assert!(!undo.status.success());
    assert!(String::from_utf8_lossy(&undo.stderr).contains("target not reachable"));

    let _ = fs::remove_dir_all(temp);
}
