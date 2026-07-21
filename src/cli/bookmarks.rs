use std::env;
use std::path::PathBuf;

use clap::{Subcommand, ValueHint};

use crate::bookmarks::{BookmarkError, BookmarkStore, storage};

#[derive(Debug, Subcommand)]
pub enum BookmarksCommand {
    /// Save a bookmark for a directory
    Add {
        /// Bookmark name (alphanumeric, hyphens, underscores)
        name: String,
        /// Directory path to bookmark (defaults to current directory)
        #[arg(value_hint = ValueHint::DirPath)]
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
    let mut store = match read_store_or_exit() {
        Ok(value) => value,
        Err(code) => return code,
    };

    let resolved = match resolve_bookmark_path(path) {
        Ok(value) => value,
        Err(code) => return code,
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
    let mut store = match read_store_or_exit() {
        Ok(value) => value,
        Err(code) => return code,
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
    let store = match read_store_or_exit() {
        Ok(value) => value,
        Err(code) => return code,
    };

    if json {
        return print_bookmarks_json(&store);
    }

    print_bookmarks_plain(&store)
}

fn read_store_or_exit() -> Result<BookmarkStore, i32> {
    storage::read_store().map_err(storage_error)
}

fn current_dir_or_exit() -> Result<PathBuf, i32> {
    env::current_dir().map_err(|err| {
        eprintln!("dx bookmarks: failed to read current directory: {err}");
        1
    })
}

fn resolve_bookmark_path(path: Option<&str>) -> Result<PathBuf, i32> {
    match path {
        Some(value) => {
            let path = PathBuf::from(value);
            if path.is_absolute() {
                Ok(path)
            } else {
                current_dir_or_exit().map(|cwd| cwd.join(path))
            }
        }
        None => current_dir_or_exit(),
    }
}

fn print_bookmarks_json(store: &BookmarkStore) -> i32 {
    match serde_json::to_string(&store.to_serializable_map()) {
        Ok(output) => {
            println!("{output}");
            0
        }
        Err(err) => {
            eprintln!("dx bookmarks: failed to serialize json: {err}");
            1
        }
    }
}

fn print_bookmarks_plain(store: &BookmarkStore) -> i32 {
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
    use crate::test_support::{ScopedProcess, temp_dir};

    use super::*;

    #[test]
    fn empty_list_returns_zero() {
        let mut process = ScopedProcess::new();
        let temp = temp_dir("cli-bookmarks-empty-list");
        let file = temp.path().join("bookmarks.toml");
        process.set("DX_BOOKMARKS_FILE", file.as_os_str());

        let code = run_list(false);
        assert_eq!(code, 0);
    }
}
