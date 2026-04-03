use std::path::PathBuf;

/// Normalise a query string to a canonical form for directory matching.
///
/// Applied rules (in order):
/// 1. Trim surrounding whitespace.
/// 2. Strip a trailing `/` — `/foo/bar/` and `/foo/bar` refer to the same directory.
/// 3. Expand a leading `~/` (or bare `~`) to the user's home directory, so that
///    `~/projects` matches `/Users/nick/projects`.
/// 4. Strip a leading `./` — `./src` is treated the same as the bare name `src`
///    for substring/prefix matching purposes (we have no cwd here to make it absolute).
///
/// The result is lowercased after normalisation so all comparisons are case-insensitive.
fn normalize_query(query: &str) -> String {
    let s = query.trim();

    // Strip trailing slash.
    let s = s.trim_end_matches('/');

    // Expand `~` / `~/...`.
    let s: std::borrow::Cow<str> = if s == "~" {
        if let Some(home) = dirs::home_dir() {
            home.display().to_string().into()
        } else {
            s.into()
        }
    } else if let Some(rest) = s.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            format!("{}/{}", home.display(), rest).into()
        } else {
            s.into()
        }
    } else {
        s.into()
    };

    // Strip leading `./`.
    let s = s.strip_prefix("./").unwrap_or(&s);

    s.to_ascii_lowercase()
}

pub fn filter_candidates(candidates: &[PathBuf], query: &str) -> Vec<PathBuf> {
    let query = normalize_query(query);
    if query.is_empty() {
        return candidates.to_vec();
    }

    let mut exact_path = Vec::new();
    let mut exact_basename = Vec::new();
    let mut path_prefix = Vec::new();
    let mut basename_prefix = Vec::new();
    let mut substring = Vec::new();

    for candidate in candidates {
        let full = candidate.display().to_string();
        let full_lower = full.to_ascii_lowercase();
        let basename = candidate
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        let basename_lower = basename.to_ascii_lowercase();

        if full_lower == query {
            exact_path.push(candidate.clone());
            continue;
        }

        if !basename_lower.is_empty() && basename_lower == query {
            exact_basename.push(candidate.clone());
            continue;
        }

        if full_lower.starts_with(&*query) {
            path_prefix.push(candidate.clone());
            continue;
        }

        if !basename_lower.is_empty() && basename_lower.starts_with(&*query) {
            basename_prefix.push(candidate.clone());
            continue;
        }

        if full_lower.contains(&*query) {
            substring.push(candidate.clone());
        }
    }

    exact_path
        .into_iter()
        .chain(exact_basename)
        .chain(path_prefix)
        .chain(basename_prefix)
        .chain(substring)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{filter_candidates, normalize_query};

    // --- normalize_query ---

    #[test]
    fn normalize_strips_trailing_slash() {
        assert_eq!(normalize_query("/foo/bar/"), "/foo/bar");
    }

    #[test]
    fn normalize_expands_tilde() {
        if let Some(home) = dirs::home_dir() {
            let expected = format!("{}/projects", home.display()).to_ascii_lowercase();
            assert_eq!(normalize_query("~/projects"), expected);
        }
    }

    #[test]
    fn normalize_bare_tilde() {
        if let Some(home) = dirs::home_dir() {
            let expected = home.display().to_string().to_ascii_lowercase();
            assert_eq!(normalize_query("~"), expected);
        }
    }

    #[test]
    fn normalize_strips_dot_slash_prefix() {
        assert_eq!(normalize_query("./src"), "src");
    }

    #[test]
    fn normalize_trailing_slash_and_tilde() {
        if let Some(home) = dirs::home_dir() {
            let expected = format!("{}/projects", home.display()).to_ascii_lowercase();
            assert_eq!(normalize_query("~/projects/"), expected);
        }
    }

    // --- filter_candidates ---

    #[test]
    fn exact_basename_ranks_before_prefix_match() {
        let candidates = vec![
            PathBuf::from("/home/user/code-review"),
            PathBuf::from("/home/user/code"),
        ];

        let filtered = filter_candidates(&candidates, "code");
        assert_eq!(
            filtered,
            vec![
                PathBuf::from("/home/user/code"),
                PathBuf::from("/home/user/code-review")
            ]
        );
    }

    #[test]
    fn path_prefix_matches_are_included() {
        let candidates = vec![
            PathBuf::from("/home/user/projects/dx"),
            PathBuf::from("/tmp/scratch"),
        ];

        let filtered = filter_candidates(&candidates, "/home/user/pro");
        assert_eq!(filtered, vec![PathBuf::from("/home/user/projects/dx")]);
    }

    #[test]
    fn substring_matches_are_case_insensitive() {
        let candidates = vec![
            PathBuf::from("/home/user/projects/dx"),
            PathBuf::from("/tmp/scratch"),
        ];

        let filtered = filter_candidates(&candidates, "ProJ");
        assert_eq!(filtered, vec![PathBuf::from("/home/user/projects/dx")]);
    }

    #[test]
    fn preserves_input_order_within_same_match_tier() {
        let candidates = vec![
            PathBuf::from("/home/user/projects/alpha"),
            PathBuf::from("/home/user/projects/alpine"),
            PathBuf::from("/home/user/projects/algebra"),
        ];

        let filtered = filter_candidates(&candidates, "al");
        assert_eq!(filtered, candidates);
    }

    #[test]
    fn no_match_returns_empty() {
        let candidates = vec![PathBuf::from("/home/user/projects/dx")];
        let filtered = filter_candidates(&candidates, "zzz");
        assert!(filtered.is_empty());
    }

    #[test]
    fn trailing_slash_matches_candidate_without_slash() {
        let candidates = vec![PathBuf::from("/Users/nick/code/personal/dx")];
        let filtered = filter_candidates(&candidates, "/Users/nick/code/personal/dx/");
        assert_eq!(filtered, candidates);
    }

    #[test]
    fn tilde_matches_absolute_candidate() {
        if let Some(home) = dirs::home_dir() {
            let candidate = home.join("projects");
            let filtered = filter_candidates(&[candidate.clone()], "~/projects");
            assert_eq!(filtered, vec![candidate]);
        }
    }

    #[test]
    fn dot_slash_prefix_matches_by_basename() {
        let candidates = vec![PathBuf::from("/some/deep/path/src")];
        let filtered = filter_candidates(&candidates, "./src");
        assert_eq!(filtered, candidates);
    }
}
