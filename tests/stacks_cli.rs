use std::fs;
use std::path::{Path, PathBuf};

use dx::stacks::SessionStack;

mod common;

fn read_session(path: &Path) -> SessionStack {
    let raw = fs::read_to_string(path).expect("read session file");
    serde_json::from_str::<SessionStack>(&raw).expect("parse session json")
}

#[test]
fn full_push_undo_redo_push_cycle_updates_session_file() {
    let temp = common::temp_dir("stacks-cycle");
    let runtime = temp.path().join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let a = temp.path().join("a");
    let b = temp.path().join("b");
    let d = temp.path().join("d");
    fs::create_dir_all(&a).expect("create a");
    fs::create_dir_all(&b).expect("create b");
    fs::create_dir_all(&d).expect("create d");

    let a = common::canonical(&a);
    let b = common::canonical(&b);
    let d = common::canonical(&d);

    let push_a = common::dx()
        .args([
            "stack",
            "push",
            a.to_str().expect("utf8 path"),
            "--session",
            "s1",
        ])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .current_dir(temp.path())
        .output()
        .expect("push a");
    assert!(push_a.status.success());
    assert_eq!(
        String::from_utf8_lossy(&push_a.stdout).trim(),
        a.display().to_string()
    );

    let push_b = common::dx()
        .args([
            "stack",
            "push",
            b.to_str().expect("utf8 path"),
            "--session",
            "s1",
        ])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .current_dir(temp.path())
        .output()
        .expect("push b");
    assert!(push_b.status.success());
    assert_eq!(
        String::from_utf8_lossy(&push_b.stdout).trim(),
        b.display().to_string()
    );

    let undo = common::dx()
        .args(["stack", "undo", "--session", "s1"])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .current_dir(temp.path())
        .output()
        .expect("undo");
    assert!(undo.status.success());
    assert_eq!(
        String::from_utf8_lossy(&undo.stdout).trim(),
        a.display().to_string()
    );

    let redo = common::dx()
        .args(["stack", "redo", "--session", "s1"])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .current_dir(temp.path())
        .output()
        .expect("redo");
    assert!(redo.status.success());
    assert_eq!(
        String::from_utf8_lossy(&redo.stdout).trim(),
        b.display().to_string()
    );

    let push_d = common::dx()
        .args([
            "stack",
            "push",
            d.to_str().expect("utf8 path"),
            "--session",
            "s1",
        ])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .current_dir(temp.path())
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
}

#[test]
fn missing_session_id_returns_error() {
    let temp = common::temp_dir("stacks-missing-session");
    let runtime = temp.path().join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let target = temp.path().join("target");
    fs::create_dir_all(&target).expect("create target");
    let target = common::canonical(&target);

    let output = common::dx()
        .args(["stack", "push", target.to_str().expect("utf8 path")])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .current_dir(temp.path())
        .output()
        .expect("run dx stack push");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing session id"));
}

#[test]
fn dx_session_env_is_used_and_cli_flag_overrides() {
    let temp = common::temp_dir("stacks-session-source");
    let runtime = temp.path().join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let env_dir = temp.path().join("env-dir");
    let cli_dir = temp.path().join("cli-dir");
    fs::create_dir_all(&env_dir).expect("create env dir");
    fs::create_dir_all(&cli_dir).expect("create cli dir");

    let env_dir = common::canonical(&env_dir);
    let cli_dir = common::canonical(&cli_dir);

    let by_env = common::dx()
        .args(["stack", "push", env_dir.to_str().expect("utf8 path")])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env("DX_SESSION", "env-session")
        .current_dir(temp.path())
        .output()
        .expect("push by env session");
    assert!(by_env.status.success());

    let by_cli = common::dx()
        .args([
            "stack",
            "push",
            cli_dir.to_str().expect("utf8 path"),
            "--session",
            "cli-session",
        ])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env("DX_SESSION", "env-session")
        .current_dir(temp.path())
        .output()
        .expect("push by cli session");
    assert!(by_cli.status.success());

    let env_state = read_session(&runtime.join("dx-sessions").join("env-session.json"));
    let cli_state = read_session(&runtime.join("dx-sessions").join("cli-session.json"));

    assert_eq!(env_state.cwd, Some(env_dir));
    assert_eq!(cli_state.cwd, Some(cli_dir));
}

#[test]
fn session_directory_is_auto_created_with_temp_fallback() {
    let temp = common::temp_dir("stacks-temp-fallback");
    let temp_root = temp.path().join("temp-root");
    fs::create_dir_all(&temp_root).expect("create temp root");

    let target = temp.path().join("target");
    fs::create_dir_all(&target).expect("create target");
    let target = common::canonical(&target);

    let output = common::dx()
        .args([
            "stack",
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
        .current_dir(temp.path())
        .output()
        .expect("push with temp fallback");

    assert!(output.status.success());

    let expected_dir = temp_root.join("dx-sessions");
    let expected_file = expected_dir.join("temp-fallback.json");
    assert!(expected_dir.exists());
    assert!(expected_file.exists());

    let expected_canon = common::canonical(&expected_dir);
    let actual_canon = common::canonical(expected_file.parent().expect("session file parent"));
    assert_eq!(actual_canon, expected_canon);
}

#[test]
fn undo_with_target_jumps_multiple_entries() {
    let temp = common::temp_dir("stacks-undo-target");
    let runtime = temp.path().join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let a = temp.path().join("a");
    let b = temp.path().join("b");
    let c = temp.path().join("c");
    let d = temp.path().join("d");
    for dir in [&a, &b, &c, &d] {
        fs::create_dir_all(dir).expect("create dir");
    }
    let a = common::canonical(&a);
    let b = common::canonical(&b);
    let c = common::canonical(&c);
    let d = common::canonical(&d);

    // push a -> b -> c -> d
    for dir in [&a, &b, &c, &d] {
        let out = common::dx()
            .args([
                "stack",
                "push",
                dir.to_str().unwrap(),
                "--session",
                "target1",
            ])
            .env("XDG_RUNTIME_DIR", runtime.display().to_string())
            .env_remove("DX_SESSION")
            .output()
            .unwrap();
        assert!(out.status.success());
    }

    // undo --target a (should consume c, b, reach a)
    let undo = common::dx()
        .args([
            "stack",
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
}

#[test]
fn redo_with_target_jumps_multiple_entries() {
    let temp = common::temp_dir("stacks-redo-target");
    let runtime = temp.path().join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let a = temp.path().join("a");
    let b = temp.path().join("b");
    let c = temp.path().join("c");
    for dir in [&a, &b, &c] {
        fs::create_dir_all(dir).expect("create dir");
    }
    let a = common::canonical(&a);
    let b = common::canonical(&b);
    let c = common::canonical(&c);

    // push a -> b -> c
    for dir in [&a, &b, &c] {
        let out = common::dx()
            .args([
                "stack",
                "push",
                dir.to_str().unwrap(),
                "--session",
                "target2",
            ])
            .env("XDG_RUNTIME_DIR", runtime.display().to_string())
            .env_remove("DX_SESSION")
            .output()
            .unwrap();
        assert!(out.status.success());
    }

    // undo --target a (go back to beginning)
    let undo = common::dx()
        .args([
            "stack",
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
    assert!(
        undo.status.success(),
        "undo setup failed: {}",
        String::from_utf8_lossy(&undo.stderr)
    );

    // redo --target c (skip b, jump to c)
    let redo = common::dx()
        .args([
            "stack",
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
}

#[test]
fn undo_with_unreachable_target_fails() {
    let temp = common::temp_dir("stacks-undo-unreachable");
    let runtime = temp.path().join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let a = temp.path().join("a");
    let b = temp.path().join("b");
    for dir in [&a, &b] {
        fs::create_dir_all(dir).expect("create dir");
    }
    let a = common::canonical(&a);
    let b = common::canonical(&b);

    // push a -> b
    for dir in [&a, &b] {
        let out = common::dx()
            .args([
                "stack",
                "push",
                dir.to_str().unwrap(),
                "--session",
                "target3",
            ])
            .env("XDG_RUNTIME_DIR", runtime.display().to_string())
            .env_remove("DX_SESSION")
            .output()
            .unwrap();
        assert!(out.status.success());
    }

    // undo --target /nonexistent should fail
    let undo = common::dx()
        .args([
            "stack",
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
}

#[test]
fn stack_list_plain_supports_directions_and_ordering() {
    let temp = common::temp_dir("stacks-list-plain");
    let runtime = temp.path().join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let state = SessionStack {
        cwd: Some(PathBuf::from("/x")),
        undo: vec![PathBuf::from("/a"), PathBuf::from("/b")],
        redo: vec![PathBuf::from("/c"), PathBuf::from("/d")],
    };
    let sessions_dir = runtime.join("dx-sessions");
    fs::create_dir_all(&sessions_dir).expect("create sessions dir");
    let session_file = sessions_dir.join("list1.json");
    fs::write(
        &session_file,
        serde_json::to_string(&state).expect("serialize session"),
    )
    .expect("write session file");

    let both = common::dx()
        .args([
            "stack",
            "--list",
            "--direction",
            "both",
            "--session",
            "list1",
        ])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .output()
        .expect("list both");
    assert!(both.status.success());
    assert_eq!(String::from_utf8_lossy(&both.stdout), "/b\n/a\n/d\n/c\n");

    let undo = common::dx()
        .args([
            "stack",
            "--list",
            "--direction",
            "undo",
            "--session",
            "list1",
        ])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .output()
        .expect("list undo");
    assert!(undo.status.success());
    assert_eq!(String::from_utf8_lossy(&undo.stdout), "/b\n/a\n");

    let redo = common::dx()
        .args([
            "stack",
            "--list",
            "--direction",
            "redo",
            "--session",
            "list1",
        ])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .output()
        .expect("list redo");
    assert!(redo.status.success());
    assert_eq!(String::from_utf8_lossy(&redo.stdout), "/d\n/c\n");
}

#[test]
fn stack_list_json_and_read_only_contract() {
    let temp = common::temp_dir("stacks-list-json");
    let runtime = temp.path().join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let state = SessionStack {
        cwd: Some(PathBuf::from("/x")),
        undo: vec![PathBuf::from("/a"), PathBuf::from("/b")],
        redo: Vec::new(),
    };
    let sessions_dir = runtime.join("dx-sessions");
    fs::create_dir_all(&sessions_dir).expect("create sessions dir");
    let session_file = sessions_dir.join("list2.json");
    fs::write(
        &session_file,
        serde_json::to_string(&state).expect("serialize session"),
    )
    .expect("write session file");

    let before = fs::read(&session_file).expect("read before bytes");

    let out = common::dx()
        .args([
            "stack",
            "--list",
            "--direction",
            "undo",
            "--json",
            "--session",
            "list2",
        ])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .output()
        .expect("list json");
    assert!(out.status.success());

    let parsed: serde_json::Value = serde_json::from_slice(&out.stdout).expect("parse json output");
    assert_eq!(
        parsed,
        serde_json::json!([
            {"path": "/b", "label": "b", "rank": 1},
            {"path": "/a", "label": "a", "rank": 2}
        ])
    );
    let items = parsed.as_array().expect("json array");
    for (item, (path, label, rank)) in items.iter().zip([("/b", "b", 1_u64), ("/a", "a", 2_u64)]) {
        let object = item.as_object().expect("stack item object");
        assert_eq!(object.len(), 3, "stack item must have exactly three keys");
        assert_eq!(
            object.get("path").and_then(serde_json::Value::as_str),
            Some(path)
        );
        assert_eq!(
            object.get("label").and_then(serde_json::Value::as_str),
            Some(label)
        );
        assert_eq!(
            object.get("rank").and_then(serde_json::Value::as_u64),
            Some(rank)
        );
    }

    let after = fs::read(&session_file).expect("read after bytes");
    assert_eq!(before, after, "stack --list must not mutate session file");
}

#[test]
fn stack_clear_scope_idempotent_and_preserves_cwd() {
    let temp = common::temp_dir("stacks-clear");
    let runtime = temp.path().join("runtime");
    fs::create_dir_all(&runtime).expect("create runtime");

    let state = SessionStack {
        cwd: Some(PathBuf::from("/x")),
        undo: vec![PathBuf::from("/a"), PathBuf::from("/b")],
        redo: vec![PathBuf::from("/c")],
    };
    let sessions_dir = runtime.join("dx-sessions");
    fs::create_dir_all(&sessions_dir).expect("create sessions dir");
    let session_file = sessions_dir.join("clear1.json");
    fs::write(
        &session_file,
        serde_json::to_string(&state).expect("serialize session"),
    )
    .expect("write session file");

    let clear_undo = common::dx()
        .args([
            "stack",
            "--clear",
            "--direction",
            "undo",
            "--session",
            "clear1",
        ])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .output()
        .expect("clear undo");
    assert!(clear_undo.status.success());
    assert!(String::from_utf8_lossy(&clear_undo.stdout).is_empty());

    let mid = read_session(&session_file);
    assert_eq!(mid.cwd, Some(PathBuf::from("/x")));
    assert!(mid.undo.is_empty());
    assert_eq!(mid.redo, vec![PathBuf::from("/c")]);

    let clear_undo_again = common::dx()
        .args([
            "stack",
            "--clear",
            "--direction",
            "undo",
            "--session",
            "clear1",
        ])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .output()
        .expect("clear undo again");
    assert!(clear_undo_again.status.success());

    let clear_both = common::dx()
        .args(["stack", "--clear", "--session", "clear1"])
        .env("XDG_RUNTIME_DIR", runtime.display().to_string())
        .env_remove("DX_SESSION")
        .output()
        .expect("clear both");
    assert!(clear_both.status.success());

    let end = read_session(&session_file);
    assert_eq!(end.cwd, Some(PathBuf::from("/x")));
    assert!(end.undo.is_empty());
    assert!(end.redo.is_empty());
}
