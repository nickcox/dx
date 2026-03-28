use std::path::PathBuf;

pub fn filter_candidates(candidates: &[PathBuf], query: &str) -> Vec<PathBuf> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return candidates.to_vec();
    }

    let needle = trimmed.to_ascii_lowercase();
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

        if full_lower == needle {
            exact_path.push(candidate.clone());
            continue;
        }

        if !basename_lower.is_empty() && basename_lower == needle {
            exact_basename.push(candidate.clone());
            continue;
        }

        if full_lower.starts_with(&needle) {
            path_prefix.push(candidate.clone());
            continue;
        }

        if !basename_lower.is_empty() && basename_lower.starts_with(&needle) {
            basename_prefix.push(candidate.clone());
            continue;
        }

        if full_lower.contains(&needle) {
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

    use super::filter_candidates;

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
}
