use std::path::PathBuf;

use super::traversal;

pub fn resolve_abbreviation(roots: &[PathBuf], query: &str, case_sensitive: bool) -> Vec<PathBuf> {
    if !query.contains('/') {
        return Vec::new();
    }

    let segments = query
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    if segments.is_empty() {
        return Vec::new();
    }

    let mut matches = Vec::new();

    for root in roots {
        if !root.is_dir() {
            continue;
        }

        let current =
            traversal::traverse_segment_paths(vec![root.clone()], &segments, |name, segment| {
                matches_prefix(name, segment, case_sensitive)
            });

        matches.extend(current);
    }

    matches
}

pub fn matches_prefix(name: &str, segment: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        name.starts_with(segment)
    } else {
        name.to_ascii_lowercase()
            .starts_with(&segment.to_ascii_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn make_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dx-{label}-{nonce}-{}", std::process::id()));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    #[test]
    fn resolves_multisegment_abbreviation() {
        let temp = make_temp_dir("abbrev");
        let root = temp.join("root");
        let target = root.join("project/src/components/button");
        fs::create_dir_all(&target).expect("create dirs");

        let matches = resolve_abbreviation(&[root], "pro/sr/com/bu", true);

        assert_eq!(matches, vec![target]);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn ignores_query_without_slashes() {
        let temp = make_temp_dir("abbrev-noslash");
        let root = temp.join("root");
        fs::create_dir_all(&root).expect("create dir");

        let matches = resolve_abbreviation(&[root], "project", true);
        assert!(matches.is_empty());
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn supports_case_insensitive_matching() {
        let temp = make_temp_dir("abbrev-case");
        let root = temp.join("root");
        let target = root.join("Project/Source");
        fs::create_dir_all(&target).expect("create dirs");

        let matches = resolve_abbreviation(&[root], "pro/sou", false);

        assert_eq!(matches, vec![target]);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn returns_empty_when_no_segment_path_matches() {
        let temp = make_temp_dir("abbrev-no-match");
        let root = temp.join("root");
        fs::create_dir_all(root.join("project/src/components")).expect("create dirs");

        let matches = resolve_abbreviation(&[root], "pro/zzz", true);

        assert!(matches.is_empty());
        let _ = fs::remove_dir_all(temp);
    }
}
