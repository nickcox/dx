//! Candidates from one half of the session stack, for `dx back` and
//! `dx forward`.

use std::path::PathBuf;

use super::{StackDirection, complete_session_paths};

pub fn complete(
    session: Option<&str>,
    direction: StackDirection,
    query: Option<&str>,
) -> Vec<PathBuf> {
    complete_session_paths(session, query, |stack| match direction {
        StackDirection::Back => stack.undo,
        StackDirection::Forward => stack.redo,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::stacks::{SessionStack, storage};
    use crate::test_support;

    use super::{StackDirection, complete};

    #[test]
    fn back_direction_returns_undo_entries_top_first() {
        let temp = test_support::temp_dir("complete-stack-back");
        let mut process = test_support::ScopedProcess::new();
        let runtime = temp.path().join("runtime");
        fs::create_dir_all(&runtime).expect("create runtime");
        process.set("XDG_RUNTIME_DIR", &runtime);

        let dir = storage::ensure_session_dir().expect("session dir");
        let stack = SessionStack {
            cwd: Some(temp.path().join("now")),
            undo: vec![temp.path().join("a"), temp.path().join("b")],
            redo: vec![temp.path().join("x")],
        };
        storage::write_session(&dir, "s1", &stack).expect("write session");

        let output = complete(Some("s1"), StackDirection::Back, None);
        assert_eq!(output, vec![temp.path().join("b"), temp.path().join("a")]);
    }

    #[test]
    fn forward_direction_returns_redo_entries_top_first() {
        let temp = test_support::temp_dir("complete-stack-forward");
        let mut process = test_support::ScopedProcess::new();
        let runtime = temp.path().join("runtime");
        fs::create_dir_all(&runtime).expect("create runtime");
        process.set("XDG_RUNTIME_DIR", &runtime);

        let dir = storage::ensure_session_dir().expect("session dir");
        let stack = SessionStack {
            cwd: Some(temp.path().join("now")),
            undo: vec![temp.path().join("a")],
            redo: vec![temp.path().join("x"), temp.path().join("y")],
        };
        storage::write_session(&dir, "s1", &stack).expect("write session");

        let output = complete(Some("s1"), StackDirection::Forward, None);
        assert_eq!(output, vec![temp.path().join("y"), temp.path().join("x")]);
    }

    #[test]
    fn empty_stack_direction_returns_empty() {
        let temp = test_support::temp_dir("complete-stack-empty");
        let mut process = test_support::ScopedProcess::new();
        let runtime = temp.path().join("runtime");
        fs::create_dir_all(&runtime).expect("create runtime");
        process.set("XDG_RUNTIME_DIR", &runtime);

        let dir = storage::ensure_session_dir().expect("session dir");
        storage::write_session(&dir, "s1", &SessionStack::default()).expect("write session");

        let output = complete(Some("s1"), StackDirection::Back, None);
        assert!(output.is_empty());
    }

    #[test]
    fn query_filter_is_applied() {
        let temp = test_support::temp_dir("complete-stack-filter");
        let mut process = test_support::ScopedProcess::new();
        let runtime = temp.path().join("runtime");
        fs::create_dir_all(&runtime).expect("create runtime");
        process.set("XDG_RUNTIME_DIR", &runtime);

        let dir = storage::ensure_session_dir().expect("session dir");
        let stack = SessionStack {
            cwd: Some(temp.path().join("now")),
            undo: Vec::new(),
            redo: vec![temp.path().join("scratch"), temp.path().join("projects/dx")],
        };
        storage::write_session(&dir, "s1", &stack).expect("write session");

        let output = complete(Some("s1"), StackDirection::Forward, Some("proj"));
        assert_eq!(output, vec![temp.path().join("projects/dx")]);
    }
}
