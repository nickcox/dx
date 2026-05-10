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
    use std::env;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::stacks::{SessionStack, storage};
    use crate::test_support;

    use super::{StackDirection, complete};

    fn make_temp_dir(label: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "dx-complete-stack-{label}-{nonce}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        test_support::env_lock()
    }

    #[test]
    fn back_direction_returns_undo_entries_top_first() {
        let _guard = env_lock();
        let temp = make_temp_dir("back");
        let runtime = temp.join("runtime");
        fs::create_dir_all(&runtime).expect("create runtime");
        unsafe { env::set_var("XDG_RUNTIME_DIR", runtime.display().to_string()) };

        let dir = storage::ensure_session_dir().expect("session dir");
        let stack = SessionStack {
            cwd: Some(std::path::PathBuf::from("/now")),
            undo: vec![
                std::path::PathBuf::from("/a"),
                std::path::PathBuf::from("/b"),
            ],
            redo: vec![std::path::PathBuf::from("/x")],
        };
        storage::write_session(&dir, "s1", &stack).expect("write session");

        let output = complete(Some("s1"), StackDirection::Back, None);
        assert_eq!(
            output,
            vec![
                std::path::PathBuf::from("/b"),
                std::path::PathBuf::from("/a")
            ]
        );

        unsafe { env::remove_var("XDG_RUNTIME_DIR") };
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn forward_direction_returns_redo_entries_top_first() {
        let _guard = env_lock();
        let temp = make_temp_dir("forward");
        let runtime = temp.join("runtime");
        fs::create_dir_all(&runtime).expect("create runtime");
        unsafe { env::set_var("XDG_RUNTIME_DIR", runtime.display().to_string()) };

        let dir = storage::ensure_session_dir().expect("session dir");
        let stack = SessionStack {
            cwd: Some(std::path::PathBuf::from("/now")),
            undo: vec![std::path::PathBuf::from("/a")],
            redo: vec![
                std::path::PathBuf::from("/x"),
                std::path::PathBuf::from("/y"),
            ],
        };
        storage::write_session(&dir, "s1", &stack).expect("write session");

        let output = complete(Some("s1"), StackDirection::Forward, None);
        assert_eq!(
            output,
            vec![
                std::path::PathBuf::from("/y"),
                std::path::PathBuf::from("/x")
            ]
        );

        unsafe { env::remove_var("XDG_RUNTIME_DIR") };
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn empty_stack_direction_returns_empty() {
        let _guard = env_lock();
        let temp = make_temp_dir("empty");
        let runtime = temp.join("runtime");
        fs::create_dir_all(&runtime).expect("create runtime");
        unsafe { env::set_var("XDG_RUNTIME_DIR", runtime.display().to_string()) };

        let dir = storage::ensure_session_dir().expect("session dir");
        storage::write_session(&dir, "s1", &SessionStack::default()).expect("write session");

        let output = complete(Some("s1"), StackDirection::Back, None);
        assert!(output.is_empty());

        unsafe { env::remove_var("XDG_RUNTIME_DIR") };
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn query_filter_is_applied() {
        let _guard = env_lock();
        let temp = make_temp_dir("filter");
        let runtime = temp.join("runtime");
        fs::create_dir_all(&runtime).expect("create runtime");
        unsafe { env::set_var("XDG_RUNTIME_DIR", runtime.display().to_string()) };

        let dir = storage::ensure_session_dir().expect("session dir");
        let stack = SessionStack {
            cwd: Some(std::path::PathBuf::from("/now")),
            undo: Vec::new(),
            redo: vec![
                std::path::PathBuf::from("/tmp/scratch"),
                std::path::PathBuf::from("/home/user/projects/dx"),
            ],
        };
        storage::write_session(&dir, "s1", &stack).expect("write session");

        let output = complete(Some("s1"), StackDirection::Forward, Some("proj"));
        assert_eq!(
            output,
            vec![std::path::PathBuf::from("/home/user/projects/dx")]
        );

        unsafe { env::remove_var("XDG_RUNTIME_DIR") };
        let _ = fs::remove_dir_all(temp);
    }
}
