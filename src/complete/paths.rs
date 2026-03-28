use std::collections::HashSet;
use std::path::PathBuf;

use crate::resolve::Resolver;

pub fn complete(resolver: &Resolver, query: &str) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut output = Vec::new();

    for path in resolver.collect_completion_candidates(query) {
        let key = path.display().to_string();
        if seen.insert(key) {
            output.push(path);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::bookmarks;
    use crate::config::AppConfig;
    use crate::test_support;

    use super::*;

    fn make_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "dx-complete-paths-{label}-{nonce}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        test_support::env_lock()
    }

    #[test]
    fn returns_multiple_abbreviation_candidates() {
        let _guard = env_lock();
        let temp = make_temp_dir("abbrev");
        let root = temp.join("root");
        fs::create_dir_all(root.join("projects/alpha")).expect("create projects");
        fs::create_dir_all(root.join("presentations/alpha")).expect("create presentations");

        let resolver = Resolver::with_bookmark_lookup(
            AppConfig {
                search_roots: vec![root],
                ..AppConfig::default()
            },
            |_| None,
        );

        let old_cwd = env::current_dir().expect("current dir");
        env::set_current_dir(&temp).expect("set current dir");
        let output = complete(&resolver, "pr/al");
        env::set_current_dir(old_cwd).expect("restore current dir");

        assert_eq!(output.len(), 2);
        assert!(output.iter().any(|path| path.ends_with("projects/alpha")));
        assert!(output
            .iter()
            .any(|path| path.ends_with("presentations/alpha")));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn includes_bookmark_match_when_present() {
        let _guard = env_lock();
        let temp = make_temp_dir("bookmark");
        let target = temp.join("work");
        fs::create_dir_all(&target).expect("create target");
        let target = fs::canonicalize(&target).expect("canonical target");

        let bookmarks_file = temp.join("bookmarks.toml");
        let toml = format!(
            "[bookmarks]\nwork = \"{}\"\n",
            target.display().to_string().replace('\\', "\\\\")
        );
        fs::write(&bookmarks_file, toml).expect("write bookmarks file");
        env::set_var("DX_BOOKMARKS_FILE", bookmarks_file.display().to_string());

        let resolver = Resolver::with_bookmark_lookup(AppConfig::default(), bookmarks::lookup);

        let old_cwd = env::current_dir().expect("current dir");
        env::set_current_dir(&temp).expect("set current dir");
        let output = complete(&resolver, "work");
        env::set_current_dir(old_cwd).expect("restore current dir");

        assert!(output.contains(&target));

        env::remove_var("DX_BOOKMARKS_FILE");
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn no_match_returns_empty() {
        let _guard = env_lock();
        let temp = make_temp_dir("none");
        let resolver = Resolver::with_bookmark_lookup(AppConfig::default(), |_| None);

        let old_cwd = env::current_dir().expect("current dir");
        env::set_current_dir(&temp).expect("set current dir");
        let output = complete(&resolver, "zzz");
        env::set_current_dir(old_cwd).expect("restore current dir");

        assert!(output.is_empty());
        let _ = fs::remove_dir_all(temp);
    }
}
