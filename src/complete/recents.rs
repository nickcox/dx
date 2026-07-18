use std::path::PathBuf;

use super::complete_session_paths;

pub fn complete(session: Option<&str>, query: Option<&str>) -> Vec<PathBuf> {
    complete_session_paths(session, query, |stack| stack.undo)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::stacks::{SessionStack, storage};
    use crate::test_support;

    use super::complete;

    #[test]
    fn recents_history_is_returned_most_recent_first() {
        let temp = test_support::temp_dir("complete-recents-history");
        let mut process = test_support::ScopedProcess::new();
        let runtime = temp.path().join("runtime");
        fs::create_dir_all(&runtime).expect("create runtime");
        process.set("XDG_RUNTIME_DIR", &runtime);

        let dir = storage::ensure_session_dir().expect("session dir");
        let stack = SessionStack {
            cwd: Some(temp.path().join("now")),
            undo: vec![
                temp.path().join("a"),
                temp.path().join("b"),
                temp.path().join("c"),
            ],
            redo: Vec::new(),
        };
        storage::write_session(&dir, "s1", &stack).expect("write session");

        let output = complete(Some("s1"), None);
        assert_eq!(
            output,
            vec![
                temp.path().join("c"),
                temp.path().join("b"),
                temp.path().join("a")
            ]
        );
    }

    #[test]
    fn empty_session_returns_empty() {
        let temp = test_support::temp_dir("complete-recents-empty");
        let mut process = test_support::ScopedProcess::new();
        let runtime = temp.path().join("runtime");
        fs::create_dir_all(&runtime).expect("create runtime");
        process.set("XDG_RUNTIME_DIR", &runtime);

        let dir = storage::ensure_session_dir().expect("session dir");
        let stack = SessionStack::default();
        storage::write_session(&dir, "s1", &stack).expect("write session");

        let output = complete(Some("s1"), None);
        assert!(output.is_empty());
    }

    #[test]
    fn query_filter_is_applied() {
        let temp = test_support::temp_dir("complete-recents-filter");
        let mut process = test_support::ScopedProcess::new();
        let runtime = temp.path().join("runtime");
        fs::create_dir_all(&runtime).expect("create runtime");
        process.set("XDG_RUNTIME_DIR", &runtime);

        let dir = storage::ensure_session_dir().expect("session dir");
        let stack = SessionStack {
            cwd: Some(temp.path().join("now")),
            undo: vec![temp.path().join("scratch"), temp.path().join("projects/dx")],
            redo: Vec::new(),
        };
        storage::write_session(&dir, "s1", &stack).expect("write session");

        let output = complete(Some("s1"), Some("proj"));
        assert_eq!(output, vec![temp.path().join("projects/dx")]);
    }

    #[test]
    fn missing_session_returns_empty() {
        assert!(complete(None, None).is_empty());
    }
}
