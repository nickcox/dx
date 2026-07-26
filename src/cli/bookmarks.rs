use std::env;
use std::path::PathBuf;

use clap::{Subcommand, ValueHint};

use crate::bookmarks::storage;

use super::CliError;

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

pub fn run_bookmarks(command: Option<BookmarksCommand>, json: bool) -> Result<(), CliError> {
    match command {
        Some(BookmarksCommand::Add { name, path }) => run_add(&name, path.as_deref()),
        Some(BookmarksCommand::Remove { name }) => run_remove(&name),
        Some(BookmarksCommand::List { json: list_json }) => run_list(list_json),
        // bare `dx bookmarks` or `dx bookmarks --json`
        None => run_list(json),
    }
}

fn run_add(name: &str, path: Option<&str>) -> Result<(), CliError> {
    let mut store = storage::read_store()?;
    let resolved = resolve_bookmark_path(path)?;

    store.set(name, &resolved)?;
    storage::write_store(&store)?;

    Ok(())
}

fn run_remove(name: &str) -> Result<(), CliError> {
    let mut store = storage::read_store()?;

    store.remove(name)?;
    storage::write_store(&store)?;

    Ok(())
}

fn run_list(json: bool) -> Result<(), CliError> {
    let store = storage::read_store()?;

    if json {
        let output =
            serde_json::to_string(&store.to_serializable_map()).map_err(CliError::BookmarksJson)?;
        println!("{output}");
    } else {
        for (name, path) in store.list() {
            println!("{name} = {}", path.display());
        }
    }

    Ok(())
}

fn current_dir() -> Result<PathBuf, CliError> {
    env::current_dir().map_err(CliError::BookmarksCurrentDir)
}

fn resolve_bookmark_path(path: Option<&str>) -> Result<PathBuf, CliError> {
    match path {
        Some(value) => {
            let path = PathBuf::from(value);
            if path.is_absolute() {
                Ok(path)
            } else {
                current_dir().map(|cwd| cwd.join(path))
            }
        }
        None => current_dir(),
    }
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

        run_list(false).expect("listing a missing store succeeds with no bookmarks");
    }
}
