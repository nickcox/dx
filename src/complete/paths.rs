//! Directory candidates for a query, drawn from the same resolution pipeline
//! `dx resolve` uses so the two never disagree.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::resolve::Resolver;

pub fn complete(resolver: &Resolver, query: &str) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut output = Vec::new();

    for path in resolver.collect_completion_candidates(query) {
        if seen.insert(path.clone()) {
            output.push(path);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::bookmarks;
    use crate::config::AppConfig;
    use crate::test_support;

    use super::*;

    #[test]
    fn returns_multiple_abbreviation_candidates() {
        let temp = test_support::temp_dir("complete-paths-abbrev");
        let mut process = test_support::ScopedProcess::new();
        let root = temp.path().join("root");
        fs::create_dir_all(root.join("projects/alpha")).expect("create projects");
        fs::create_dir_all(root.join("presentations/alpha")).expect("create presentations");

        let resolver = Resolver::with_bookmark_lookup(
            AppConfig {
                search_roots: vec![root],
                ..AppConfig::default()
            },
            |_| None,
        );

        process.set_current_dir(temp.path());
        let output = complete(&resolver, "pr/al");

        assert_eq!(output.len(), 2);
        assert!(output.iter().any(|path| path.ends_with("projects/alpha")));
        assert!(
            output
                .iter()
                .any(|path| path.ends_with("presentations/alpha"))
        );
    }

    #[test]
    fn includes_bookmark_match_when_present() {
        let temp = test_support::temp_dir("complete-paths-bookmark");
        let mut process = test_support::ScopedProcess::new();
        let target = temp.path().join("work");
        fs::create_dir_all(&target).expect("create target");
        let target = fs::canonicalize(&target).expect("canonical target");

        let bookmarks_file = temp.path().join("bookmarks.toml");
        let toml = format!(
            "[bookmarks]\nwork = \"{}\"\n",
            target.display().to_string().replace('\\', "\\\\")
        );
        fs::write(&bookmarks_file, toml).expect("write bookmarks file");
        process.set("DX_BOOKMARKS_FILE", &bookmarks_file);

        let resolver = Resolver::with_bookmark_lookup(AppConfig::default(), bookmarks::lookup);

        process.set_current_dir(temp.path());
        let output = complete(&resolver, "work");

        assert!(output.contains(&target));
    }

    #[test]
    fn no_match_returns_empty() {
        let temp = test_support::temp_dir("complete-paths-none");
        let mut process = test_support::ScopedProcess::new();
        let resolver = Resolver::with_bookmark_lookup(AppConfig::default(), |_| None);

        process.set_current_dir(temp.path());
        let output = complete(&resolver, "zzz");

        assert!(output.is_empty());
    }
}
