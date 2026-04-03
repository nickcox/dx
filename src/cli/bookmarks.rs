use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

use clap::Subcommand;

use crate::bookmarks::{storage, BookmarkError};

#[derive(Debug, Subcommand)]
pub enum BookmarksCommand {
    /// Save a bookmark for a directory
    Add {
        /// Bookmark name (alphanumeric, hyphens, underscores)
        name: String,
        /// Directory path to bookmark (defaults to current directory)
        path: Option<String>,
    },
    /// Remove a saved bookmark
    Remove {
        /// Bookmark name to remove
        name: String,
    },
    /// List saved bookmarks (default when no subcommand given)
    List {
        #[arg(long)]
        json: bool,
    },
}

pub fn run_bookmarks(command: Option<BookmarksCommand>, json: bool) -> i32 {
    match command {
        Some(BookmarksCommand::Add { name, path }) => run_add(&name, path.as_deref()),
        Some(BookmarksCommand::Remove { name }) => run_remove(&name),
        Some(BookmarksCommand::List { json: list_json }) => run_list(list_json),
        // bare `dx bookmarks` or `dx bookmarks --json`
        None => run_list(json),
    }
}

fn run_add(name: &str, path: Option<&str>) -> i32 {
    let mut store = match storage::read_store() {
        Ok(value) => value,
        Err(err) => return storage_error(err),
    };

    let raw_path = match path {
        Some(value) => PathBuf::from(value),
        None => match env::current_dir() {
            Ok(value) => value,
            Err(err) => {
                eprintln!("dx bookmarks: failed to read current directory: {err}");
                return 1;
            }
        },
    };

    let resolved = if raw_path.is_absolute() {
        raw_path
    } else {
        match env::current_dir() {
            Ok(cwd) => cwd.join(raw_path),
            Err(err) => {
                eprintln!("dx bookmarks: failed to read current directory: {err}");
                return 1;
            }
        }
    };

    match store.set(name, &resolved) {
        Ok(_) => {}
        Err(err) => return bookmark_error(err),
    }

    if let Err(err) = storage::write_store(&store) {
        return storage_error(err);
    }

    0
}

fn run_remove(name: &str) -> i32 {
    let mut store = match storage::read_store() {
        Ok(value) => value,
        Err(err) => return storage_error(err),
    };

    if let Err(err) = store.remove(name) {
        return bookmark_error(err);
    }

    if let Err(err) = storage::write_store(&store) {
        return storage_error(err);
    }

    0
}

fn run_list(json: bool) -> i32 {
    let store = match storage::read_store() {
        Ok(value) => value,
        Err(err) => return storage_error(err),
    };

    if json {
        let payload = store
            .list()
            .into_iter()
            .map(|(name, path)| (name, path.display().to_string()))
            .collect::<BTreeMap<_, _>>();

        match serde_json::to_string(&payload) {
            Ok(output) => {
                println!("{output}");
                return 0;
            }
            Err(err) => {
                eprintln!("dx bookmarks: failed to serialize json: {err}");
                return 1;
            }
        }
    }

    for (name, path) in store.list() {
        println!("{name} = {}", path.display());
    }

    0
}

fn storage_error(err: storage::StorageError) -> i32 {
    eprintln!("dx bookmarks: {err}");
    1
}

fn bookmark_error(err: BookmarkError) -> i32 {
    eprintln!("dx bookmarks: {err}");
    1
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::test_support;

    use super::*;

    fn make_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "dx-cli-bookmarks-{label}-{nonce}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        test_support::env_lock()
    }

    #[test]
    fn empty_list_returns_zero() {
        let _guard = env_lock();
        let temp = make_temp_dir("empty-list");
        let file = temp.join("bookmarks.toml");
        unsafe { env::set_var("DX_BOOKMARKS_FILE", file.display().to_string()) };

        let code = run_list(false);
        assert_eq!(code, 0);

        unsafe { env::remove_var("DX_BOOKMARKS_FILE") };
        let _ = fs::remove_dir_all(temp);
    }
}
