use std::fs;
use std::path::Path;

mod common;

#[test]
fn bookmarks_add_then_list_shows_entry() {
    let temp = common::temp_dir("bookmarks-add-list");
    let target = temp.path().join("proj");
    fs::create_dir_all(&target).expect("create target");
    let target = common::canonical(&target);

    let store = temp.path().join("bookmarks.toml");

    let add = common::dx()
        .args([
            "bookmarks",
            "add",
            "proj",
            target.to_str().expect("utf8 path"),
        ])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run bookmarks add");
    assert!(add.status.success());

    let list = common::dx()
        .arg("bookmarks")
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run list");

    assert!(list.status.success());
    assert_eq!(
        String::from_utf8_lossy(&list.stdout).trim(),
        format!("proj = {}", target.display())
    );
    assert!(String::from_utf8_lossy(&list.stderr).trim().is_empty());
}

#[test]
fn bookmarks_add_then_remove_then_list_is_empty() {
    let temp = common::temp_dir("bookmarks-add-remove");
    let target = temp.path().join("proj");
    fs::create_dir_all(&target).expect("create target");
    let target = common::canonical(&target);
    let store = temp.path().join("bookmarks.toml");

    let add = common::dx()
        .args([
            "bookmarks",
            "add",
            "proj",
            target.to_str().expect("utf8 path"),
        ])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run bookmarks add");
    assert!(add.status.success());

    let remove = common::dx()
        .args(["bookmarks", "remove", "proj"])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run bookmarks remove");
    assert!(remove.status.success());

    let list = common::dx()
        .arg("bookmarks")
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run list");

    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).trim().is_empty());
}

#[test]
fn bookmarks_add_then_resolve_returns_bookmarked_path() {
    let temp = common::temp_dir("bookmarks-add-resolve");
    let target = temp.path().join("proj");
    fs::create_dir_all(&target).expect("create target");
    let target = common::canonical(&target);
    let store = temp.path().join("bookmarks.toml");

    let add = common::dx()
        .args([
            "bookmarks",
            "add",
            "proj",
            target.to_str().expect("utf8 path"),
        ])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run bookmarks add");
    assert!(add.status.success());

    let resolve = common::dx()
        .args(["resolve", "proj"])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run resolve");

    assert!(resolve.status.success());
    let actual = common::canonical(Path::new(String::from_utf8_lossy(&resolve.stdout).trim()));
    assert_eq!(actual, target);
}

#[test]
fn bookmarks_remove_nonexistent_and_invalid_name_fail() {
    let temp = common::temp_dir("bookmarks-errors");
    let store = temp.path().join("bookmarks.toml");

    let remove = common::dx()
        .args(["bookmarks", "remove", "missing"])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run bookmarks remove missing");
    assert!(!remove.status.success());
    assert!(String::from_utf8_lossy(&remove.stderr).contains("bookmark not found"));

    let invalid = common::dx()
        .args(["bookmarks", "add", "bad/name"])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run bookmarks add invalid");
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("invalid bookmark name"));
}

#[test]
fn bookmarks_json_and_env_override_work() {
    let temp = common::temp_dir("bookmarks-json-env");
    let target = temp.path().join("proj");
    fs::create_dir_all(&target).expect("create target");
    let target = common::canonical(&target);
    let store = temp.path().join("custom/store.toml");

    let add = common::dx()
        .args([
            "bookmarks",
            "add",
            "proj",
            target.to_str().expect("utf8 path"),
        ])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run bookmarks add");
    assert!(add.status.success());
    assert!(store.exists());

    let list = common::dx()
        .args(["bookmarks", "--json"])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run list json");
    assert!(list.status.success());

    let json = serde_json::from_slice::<serde_json::Value>(&list.stdout).expect("parse json");
    assert_eq!(json["proj"], target.display().to_string());
}
