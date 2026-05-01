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
                matches_segment(name, segment, case_sensitive)
            });

        matches.extend(current);
    }

    matches
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SegmentToken<'a> {
    Literal(&'a str),
    Delimiter(char),
    Gap,
}

pub fn matches_segment(name: &str, segment: &str, case_sensitive: bool) -> bool {
    if segment.is_empty() {
        return false;
    }

    if !contains_shortening_operator(segment) {
        return if case_sensitive {
            name.starts_with(segment)
        } else {
            name.to_ascii_lowercase()
                .starts_with(&segment.to_ascii_lowercase())
        };
    }

    let candidate = normalize_for_matching(name, case_sensitive);
    let query = normalize_for_matching(segment, case_sensitive);
    let tokens = tokenize_segment(&query);
    if !tokens.iter().any(|token| matches!(token, SegmentToken::Literal(_))) {
        return false;
    }

    let mut cursor = 0;
    let mut search_literal = false;

    for (idx, token) in tokens.iter().enumerate() {
        match token {
            SegmentToken::Literal(fragment) => {
                if idx == 0 && !search_literal {
                    if !candidate[cursor..].starts_with(fragment) {
                        return false;
                    }
                    cursor += fragment.len();
                } else if let Some(offset) = candidate[cursor..].find(fragment) {
                    cursor += offset + fragment.len();
                } else {
                    return false;
                }
                search_literal = false;
            }
            SegmentToken::Delimiter(delimiter) => {
                if let Some(offset) = candidate[cursor..]
                    .as_bytes()
                    .iter()
                    .position(|byte| *byte == *delimiter as u8)
                {
                    cursor += offset + 1;
                    search_literal = true;
                } else {
                    return false;
                }
            }
            SegmentToken::Gap => {
                search_literal = true;
            }
        }
    }

    true
}

fn contains_shortening_operator(segment: &str) -> bool {
    segment
        .bytes()
        .any(|byte| matches!(byte, b'.' | b'_' | b'-'))
}

fn normalize_for_matching(input: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        input.to_string()
    } else {
        input.to_ascii_lowercase()
    }
}

fn tokenize_segment(segment: &str) -> Vec<SegmentToken<'_>> {
    let bytes = segment.as_bytes();
    let mut tokens = Vec::new();
    let mut literal_start = 0;
    let mut idx = 0;

    while idx < bytes.len() {
        if bytes[idx] == b'.' && idx + 1 < bytes.len() && bytes[idx + 1] == b'.' {
            if literal_start < idx {
                tokens.push(SegmentToken::Literal(&segment[literal_start..idx]));
            }
            tokens.push(SegmentToken::Gap);
            idx += 2;
            literal_start = idx;
            continue;
        }

        if matches!(bytes[idx], b'.' | b'_' | b'-') {
            if literal_start < idx {
                tokens.push(SegmentToken::Literal(&segment[literal_start..idx]));
            }
            tokens.push(SegmentToken::Delimiter(bytes[idx] as char));
            idx += 1;
            literal_start = idx;
            continue;
        }

        idx += 1;
    }

    if literal_start < segment.len() {
        tokens.push(SegmentToken::Literal(&segment[literal_start..]));
    }

    tokens
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

    #[test]
    fn matches_plain_prefix_without_shortening_operators() {
        assert!(matches_segment("project", "pro", true));
        assert!(!matches_segment("project", "roj", true));
    }

    #[test]
    fn matches_hyphen_delimited_fragment() {
        assert!(matches_segment("cd-extras", "cd-e", true));
        assert!(!matches_segment("editor-cd-extras", "cd-e", true));
    }

    #[test]
    fn matches_leading_dot_fragment_against_interior_suffix() {
        assert!(matches_segment("Microsoft.PowerShell.SDK", ".sdk", false));
    }

    #[test]
    fn preserves_delimiter_identity() {
        assert!(matches_segment("foo_bar", "foo_bar", true));
        assert!(!matches_segment("foo-bar", "foo_bar", true));
    }

    #[test]
    fn matches_doubled_period_gap_queries() {
        assert!(matches_segment("PowerShell", "p..shell", false));
        assert!(matches_segment("System32", "s..32", false));
        assert!(matches_segment("foo-bar", "f..bar", true));
    }

    #[test]
    fn tokenizes_doubled_period_before_single_dot_delimiter() {
        assert_eq!(
            tokenize_segment("a..b.c"),
            vec![
                SegmentToken::Literal("a"),
                SegmentToken::Gap,
                SegmentToken::Literal("b"),
                SegmentToken::Delimiter('.'),
                SegmentToken::Literal("c"),
            ]
        );
    }

    #[test]
    fn supports_case_insensitive_operator_matching() {
        assert!(matches_segment("Project_Source", "pro_sou", false));
        assert!(!matches_segment("Project_Source", "pro_sou", true));
    }

    #[test]
    fn gap_only_segment_does_not_match_everything() {
        assert!(!matches_segment("project", "..", true));
        assert!(!matches_segment("project", ".", true));
    }
}
