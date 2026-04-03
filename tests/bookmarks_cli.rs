use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn make_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "dx-it-bookmarks-{label}-{nonce}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

fn dx_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_dx"))
}

fn canonical(path: &Path) -> PathBuf {
    fs::canonicalize(path).expect("canonical path")
}

#[test]
fn bookmarks_add_then_list_shows_entry() {
    let temp = make_temp_dir("add-list");
    let target = temp.join("proj");
    fs::create_dir_all(&target).expect("create target");
    let target = canonical(&target);

    let store = temp.join("bookmarks.toml");

    let add = Command::new(dx_bin())
        .args([
            "bookmarks",
            "add",
            "proj",
            target.to_str().expect("utf8 path"),
        ])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(&temp)
        .output()
        .expect("run bookmarks add");
    assert!(add.status.success());

    let list = Command::new(dx_bin())
        .arg("bookmarks")
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(&temp)
        .output()
        .expect("run list");

    assert!(list.status.success());
    assert_eq!(
        String::from_utf8_lossy(&list.stdout).trim(),
        format!("proj = {}", target.display())
    );
    assert!(String::from_utf8_lossy(&list.stderr).trim().is_empty());

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn bookmarks_add_then_remove_then_list_is_empty() {
    let temp = make_temp_dir("add-remove");
    let target = temp.join("proj");
    fs::create_dir_all(&target).expect("create target");
    let target = canonical(&target);
    let store = temp.join("bookmarks.toml");

    let add = Command::new(dx_bin())
        .args([
            "bookmarks",
            "add",
            "proj",
            target.to_str().expect("utf8 path"),
        ])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(&temp)
        .output()
        .expect("run bookmarks add");
    assert!(add.status.success());

    let remove = Command::new(dx_bin())
        .args(["bookmarks", "remove", "proj"])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(&temp)
        .output()
        .expect("run bookmarks remove");
    assert!(remove.status.success());

    let list = Command::new(dx_bin())
        .arg("bookmarks")
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(&temp)
        .output()
        .expect("run list");

    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).trim().is_empty());

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn bookmarks_add_then_resolve_returns_bookmarked_path() {
    let temp = make_temp_dir("add-resolve");
    let target = temp.join("proj");
    fs::create_dir_all(&target).expect("create target");
    let target = canonical(&target);
    let store = temp.join("bookmarks.toml");

    let add = Command::new(dx_bin())
        .args([
            "bookmarks",
            "add",
            "proj",
            target.to_str().expect("utf8 path"),
        ])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(&temp)
        .output()
        .expect("run bookmarks add");
    assert!(add.status.success());

    let resolve = Command::new(dx_bin())
        .args(["resolve", "proj"])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(&temp)
        .output()
        .expect("run resolve");

    assert!(resolve.status.success());
    let actual = canonical(Path::new(String::from_utf8_lossy(&resolve.stdout).trim()));
    assert_eq!(actual, target);

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn bookmarks_remove_nonexistent_and_invalid_name_fail() {
    let temp = make_temp_dir("errors");
    let store = temp.join("bookmarks.toml");

    let remove = Command::new(dx_bin())
        .args(["bookmarks", "remove", "missing"])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(&temp)
        .output()
        .expect("run bookmarks remove missing");
    assert!(!remove.status.success());
    assert!(String::from_utf8_lossy(&remove.stderr).contains("bookmark not found"));

    let invalid = Command::new(dx_bin())
        .args(["bookmarks", "add", "bad/name"])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(&temp)
        .output()
        .expect("run bookmarks add invalid");
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("invalid bookmark name"));

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn bookmarks_json_and_env_override_work() {
    let temp = make_temp_dir("json-env");
    let target = temp.join("proj");
    fs::create_dir_all(&target).expect("create target");
    let target = canonical(&target);
    let store = temp.join("custom/store.toml");

    let add = Command::new(dx_bin())
        .args([
            "bookmarks",
            "add",
            "proj",
            target.to_str().expect("utf8 path"),
        ])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(&temp)
        .output()
        .expect("run bookmarks add");
    assert!(add.status.success());
    assert!(store.exists());

    let list = Command::new(dx_bin())
        .args(["bookmarks", "--json"])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(&temp)
        .output()
        .expect("run list json");
    assert!(list.status.success());

    let json = serde_json::from_slice::<serde_json::Value>(&list.stdout).expect("parse json");
    assert_eq!(json["proj"], target.display().to_string());

    let _ = fs::remove_dir_all(temp);
}
