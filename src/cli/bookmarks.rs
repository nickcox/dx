//! `dx bookmarks` — add, remove and list bookmarks.

use std::env;
use std::path::PathBuf;

use clap::{Subcommand, ValueHint};

use crate::bookmarks::{self, storage};

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
    /// Remove bookmarks whose target directory no longer exists
    Prune {
        #[arg(long)]
        json: bool,
    },
}

pub fn run_bookmarks(command: Option<BookmarksCommand>, json: bool) -> Result<(), CliError> {
    match command {
        Some(BookmarksCommand::Add { name, path }) => run_add(&name, path.as_deref()),
        Some(BookmarksCommand::Remove { name }) => run_remove(&name),
        Some(BookmarksCommand::List { json: list_json }) => run_list(list_json),
        Some(BookmarksCommand::Prune { json: prune_json }) => run_prune(prune_json),
        // bare `dx bookmarks` or `dx bookmarks --json`
        None => run_list(json),
    }
}

fn run_add(name: &str, path: Option<&str>) -> Result<(), CliError> {
    let mut store = storage::read_store()?;
    let resolved = resolve_bookmark_path(path)?;

    // Echoing the canonical path is the only way the user sees that a symlink
    // was resolved, or which directory a bare `add` actually captured.
    let canonical = store.set(name, &resolved)?;
    storage::write_store(&store)?;
    println!("{}", canonical.display());

    Ok(())
}

fn run_remove(name: &str) -> Result<(), CliError> {
    let mut store = storage::read_store()?;

    let removed = store.remove(name)?;
    storage::write_store(&store)?;
    println!("{}", removed.display());

    Ok(())
}

fn run_list(json: bool) -> Result<(), CliError> {
    let store = storage::read_store()?;
    print_entries(&store.entries(), json)
}

fn run_prune(json: bool) -> Result<(), CliError> {
    let mut store = storage::read_store()?;
    let removed = store.prune_stale();

    // Only rewrite when something actually changed, so the common no-op does
    // not churn the store file.
    if !removed.is_empty() {
        storage::write_store(&store)?;
    }

    print_entries(&removed, json)
}

fn print_entries(entries: &[bookmarks::BookmarkEntry], json: bool) -> Result<(), CliError> {
    if json {
        let output = serde_json::to_string(&bookmarks::to_serializable_entries(entries)?)
            .map_err(CliError::BookmarksJson)?;
        println!("{output}");
    } else {
        for entry in entries {
            let marker = if entry.exists { "" } else { " (missing)" };
            println!("{} = {}{marker}", entry.name, entry.path.display());
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
