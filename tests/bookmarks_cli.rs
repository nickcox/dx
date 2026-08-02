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
    let entries = json.as_array().expect("json is an array of entries");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["name"], "proj");
    assert_eq!(entries[0]["path"], target.display().to_string());
    assert_eq!(entries[0]["exists"], true);
}

/// Adds a bookmark through the CLI, returning what `add` printed.
fn add_bookmark(store: &Path, cwd: &Path, name: &str, target: &Path) -> String {
    let add = common::dx()
        .args(["bookmarks", "add", name, target.to_str().expect("utf8 path")])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(cwd)
        .output()
        .expect("run bookmarks add");
    assert!(add.status.success());
    String::from_utf8_lossy(&add.stdout).trim().to_string()
}

#[test]
fn bookmarks_add_echoes_canonical_path() {
    let temp = common::temp_dir("bookmarks-add-echo");
    let target = temp.path().join("proj");
    fs::create_dir_all(&target).expect("create target");
    let target = common::canonical(&target);
    let store = temp.path().join("bookmarks.toml");

    let echoed = add_bookmark(&store, temp.path(), "proj", &target);
    assert_eq!(common::canonical(Path::new(&echoed)), target);
}

#[cfg(unix)]
#[test]
fn bookmarks_add_echoes_the_target_a_symlink_points_at() {
    use std::os::unix::fs::symlink;

    let temp = common::temp_dir("bookmarks-add-echo-symlink");
    let target = temp.path().join("real");
    fs::create_dir_all(&target).expect("create target");
    let target = common::canonical(&target);
    let link = temp.path().join("link");
    symlink(&target, &link).expect("create symlink");
    let store = temp.path().join("bookmarks.toml");

    // Canonicalization is invisible until it surprises you, so `add` says where
    // the bookmark actually points.
    let echoed = add_bookmark(&store, temp.path(), "proj", &link);
    assert_eq!(common::canonical(Path::new(&echoed)), target);
}

#[test]
fn bookmarks_remove_echoes_removed_path() {
    let temp = common::temp_dir("bookmarks-remove-echo");
    let target = temp.path().join("proj");
    fs::create_dir_all(&target).expect("create target");
    let target = common::canonical(&target);
    let store = temp.path().join("bookmarks.toml");
    let _ = add_bookmark(&store, temp.path(), "proj", &target);

    let remove = common::dx()
        .args(["bookmarks", "remove", "proj"])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run bookmarks remove");

    assert!(remove.status.success());
    let echoed = String::from_utf8_lossy(&remove.stdout).trim().to_string();
    assert_eq!(common::canonical(Path::new(&echoed)), target);
}

#[test]
fn bookmarks_list_marks_stale_entries() {
    let temp = common::temp_dir("bookmarks-list-stale");
    let target = temp.path().join("proj");
    fs::create_dir_all(&target).expect("create target");
    let target = common::canonical(&target);
    let store = temp.path().join("bookmarks.toml");
    let _ = add_bookmark(&store, temp.path(), "proj", &target);
    fs::remove_dir_all(&target).expect("remove target");

    let list = common::dx()
        .arg("bookmarks")
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run list");

    assert!(list.status.success());
    assert_eq!(
        String::from_utf8_lossy(&list.stdout).trim(),
        format!("proj = {} (missing)", target.display())
    );
}

#[test]
fn bookmarks_prune_reports_and_removes_stale_entries() {
    let temp = common::temp_dir("bookmarks-prune");
    let live = temp.path().join("live");
    let stale = temp.path().join("stale");
    for dir in [&live, &stale] {
        fs::create_dir_all(dir).expect("create dir");
    }
    let live = common::canonical(&live);
    let stale = common::canonical(&stale);
    let store = temp.path().join("bookmarks.toml");

    let _ = add_bookmark(&store, temp.path(), "live", &live);
    let _ = add_bookmark(&store, temp.path(), "stale", &stale);
    fs::remove_dir_all(&stale).expect("remove stale target");

    let prune = common::dx()
        .args(["bookmarks", "prune"])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run prune");

    assert!(prune.status.success());
    assert_eq!(
        String::from_utf8_lossy(&prune.stdout).trim(),
        format!("stale = {} (missing)", stale.display())
    );

    let list = common::dx()
        .arg("bookmarks")
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run list");

    assert!(list.status.success());
    assert_eq!(
        String::from_utf8_lossy(&list.stdout).trim(),
        format!("live = {}", live.display())
    );
}

#[test]
fn bookmarks_prune_with_nothing_stale_is_silent_and_succeeds() {
    let temp = common::temp_dir("bookmarks-prune-noop");
    let target = temp.path().join("proj");
    fs::create_dir_all(&target).expect("create target");
    let target = common::canonical(&target);
    let store = temp.path().join("bookmarks.toml");
    let _ = add_bookmark(&store, temp.path(), "proj", &target);

    let prune = common::dx()
        .args(["bookmarks", "prune"])
        .env("DX_BOOKMARKS_FILE", store.display().to_string())
        .current_dir(temp.path())
        .output()
        .expect("run prune");

    assert!(prune.status.success());
    assert!(String::from_utf8_lossy(&prune.stdout).trim().is_empty());
    assert!(String::from_utf8_lossy(&prune.stderr).trim().is_empty());
}
