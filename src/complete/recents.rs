use std::path::PathBuf;

use crate::stacks::storage;

use super::filter::filter_candidates;

pub fn complete(session: Option<&str>, query: Option<&str>) -> Vec<PathBuf> {
    let session = match session.filter(|value| !value.trim().is_empty()) {
        Some(value) => value,
        None => return Vec::new(),
    };

    let dir = match storage::ensure_session_dir() {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    let stack = match storage::read_session(&dir, session) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    let mut output = stack.undo;
    output.reverse();

    match query.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => filter_candidates(&output, value),
        None => output,
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::stacks::{storage, SessionStack};
    use crate::test_support;

    use super::complete;

    fn make_temp_dir(label: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "dx-complete-recents-{label}-{nonce}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        test_support::env_lock()
    }

    #[test]
    fn recents_history_is_returned_most_recent_first() {
        let _guard = env_lock();
        let temp = make_temp_dir("history");
        let runtime = temp.join("runtime");
        fs::create_dir_all(&runtime).expect("create runtime");
        env::set_var("XDG_RUNTIME_DIR", runtime.display().to_string());

        let dir = storage::ensure_session_dir().expect("session dir");
        let stack = SessionStack {
            cwd: Some(std::path::PathBuf::from("/now")),
            undo: vec![
                std::path::PathBuf::from("/a"),
                std::path::PathBuf::from("/b"),
                std::path::PathBuf::from("/c"),
            ],
            redo: Vec::new(),
        };
        storage::write_session(&dir, "s1", &stack).expect("write session");

        let output = complete(Some("s1"), None);
        assert_eq!(
            output,
            vec![
                std::path::PathBuf::from("/c"),
                std::path::PathBuf::from("/b"),
                std::path::PathBuf::from("/a")
            ]
        );

        env::remove_var("XDG_RUNTIME_DIR");
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn empty_session_returns_empty() {
        let _guard = env_lock();
        let temp = make_temp_dir("empty");
        let runtime = temp.join("runtime");
        fs::create_dir_all(&runtime).expect("create runtime");
        env::set_var("XDG_RUNTIME_DIR", runtime.display().to_string());

        let dir = storage::ensure_session_dir().expect("session dir");
        let stack = SessionStack::default();
        storage::write_session(&dir, "s1", &stack).expect("write session");

        let output = complete(Some("s1"), None);
        assert!(output.is_empty());

        env::remove_var("XDG_RUNTIME_DIR");
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn query_filter_is_applied() {
        let _guard = env_lock();
        let temp = make_temp_dir("filter");
        let runtime = temp.join("runtime");
        fs::create_dir_all(&runtime).expect("create runtime");
        env::set_var("XDG_RUNTIME_DIR", runtime.display().to_string());

        let dir = storage::ensure_session_dir().expect("session dir");
        let stack = SessionStack {
            cwd: Some(std::path::PathBuf::from("/now")),
            undo: vec![
                std::path::PathBuf::from("/tmp/scratch"),
                std::path::PathBuf::from("/home/user/projects/dx"),
            ],
            redo: Vec::new(),
        };
        storage::write_session(&dir, "s1", &stack).expect("write session");

        let output = complete(Some("s1"), Some("proj"));
        assert_eq!(
            output,
            vec![std::path::PathBuf::from("/home/user/projects/dx")]
        );

        env::remove_var("XDG_RUNTIME_DIR");
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn missing_session_returns_empty() {
        assert!(complete(None, None).is_empty());
    }
}
