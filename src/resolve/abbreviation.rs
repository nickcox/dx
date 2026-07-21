use std::path::PathBuf;

use super::{path_query, traversal};

pub fn resolve_abbreviation(roots: &[PathBuf], query: &str, case_sensitive: bool) -> Vec<PathBuf> {
    if !path_query::has_separator(query) {
        return Vec::new();
    }

    let segments = parse_query_segments(query, case_sensitive);

    if segments.is_empty() {
        return Vec::new();
    }

    roots
        .iter()
        .filter(|root| root.is_dir())
        .flat_map(|root| {
            traversal::traverse_segment_paths(vec![root.clone()], &segments, |name, segment| {
                segment.matches(name)
            })
        })
        .collect()
}

pub fn resolve_abbreviation_exact(
    roots: &[PathBuf],
    query: &str,
    case_sensitive: bool,
) -> Result<Vec<PathBuf>, (PathBuf, std::io::Error)> {
    if !path_query::has_separator(query) {
        return Ok(Vec::new());
    }
    let segments = parse_query_segments(query, case_sensitive);
    if segments.is_empty() {
        return Ok(Vec::new());
    }

    let mut matches = Vec::new();
    for root in roots {
        // Configured roots are optional search locations, so an unavailable one
        // must not prevent another root from resolving the query.
        if !root.is_dir() {
            continue;
        }
        matches.extend(traversal::try_traverse_segment_paths(
            vec![root.clone()],
            &segments,
            |name, segment| segment.matches(name),
        )?);
    }
    Ok(matches)
}

fn parse_query_segments(query: &str, case_sensitive: bool) -> Vec<ParsedSegment> {
    query
        .split(std::path::is_separator)
        .filter(|segment| !segment.is_empty())
        .map(|segment| ParsedSegment::new(segment, case_sensitive))
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedSegment {
    normalized: String,
    tokens: Vec<SegmentToken>,
    operator_aware: bool,
    case_sensitive: bool,
}

impl ParsedSegment {
    fn new(segment: &str, case_sensitive: bool) -> Self {
        let normalized = normalize_for_matching(segment, case_sensitive);
        let operator_aware = contains_shortening_operator(&normalized);
        let tokens = if operator_aware {
            tokenize_segment(&normalized)
        } else {
            Vec::new()
        };

        Self {
            normalized,
            tokens,
            operator_aware,
            case_sensitive,
        }
    }

    fn matches(&self, name: &str) -> bool {
        if self.normalized.is_empty() {
            return false;
        }

        if !self.operator_aware {
            return if self.case_sensitive {
                name.starts_with(&self.normalized)
            } else {
                name.to_ascii_lowercase().starts_with(&self.normalized)
            };
        }

        if !self
            .tokens
            .iter()
            .any(|token| matches!(token, SegmentToken::Literal(_)))
        {
            return false;
        }

        let candidate = normalize_for_matching(name, self.case_sensitive);
        self.matches_operator_tokens(&candidate)
    }

    fn matches_operator_tokens(&self, candidate: &str) -> bool {
        let mut cursor = 0;
        let mut search_literal = false;

        for (idx, token) in self.tokens.iter().enumerate() {
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
                    if let Some(offset) = candidate.as_bytes()[cursor..]
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SegmentToken {
    Literal(String),
    Delimiter(char),
    Gap,
}

pub fn matches_segment(name: &str, segment: &str, case_sensitive: bool) -> bool {
    ParsedSegment::new(segment, case_sensitive).matches(name)
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

fn tokenize_segment(segment: &str) -> Vec<SegmentToken> {
    let bytes = segment.as_bytes();
    let mut tokens = Vec::new();
    let mut literal_start = 0;
    let mut idx = 0;

    while idx < bytes.len() {
        if bytes[idx] == b'.' && idx + 1 < bytes.len() && bytes[idx + 1] == b'.' {
            if literal_start < idx {
                tokens.push(SegmentToken::Literal(
                    segment[literal_start..idx].to_string(),
                ));
            }
            tokens.push(SegmentToken::Gap);
            idx += 2;
            literal_start = idx;
            continue;
        }

        if matches!(bytes[idx], b'.' | b'_' | b'-') {
            if literal_start < idx {
                tokens.push(SegmentToken::Literal(
                    segment[literal_start..idx].to_string(),
                ));
            }
            tokens.push(SegmentToken::Delimiter(bytes[idx] as char));
            idx += 1;
            literal_start = idx;
            continue;
        }

        idx += 1;
    }

    if literal_start < segment.len() {
        tokens.push(SegmentToken::Literal(segment[literal_start..].to_string()));
    }

    tokens
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::test_support::{TempDir, temp_dir};

    #[test]
    fn resolves_multisegment_abbreviation() {
        let temp: TempDir = temp_dir("abbrev");
        let root = temp.path().join("root");
        let target = root.join("project/src/components/button");
        fs::create_dir_all(&target).expect("create dirs");

        let matches = resolve_abbreviation(&[root], "pro/sr/com/bu", true);

        assert_eq!(matches, vec![target]);
    }

    #[test]
    fn ignores_query_without_slashes() {
        let temp: TempDir = temp_dir("abbrev-noslash");
        let root = temp.path().join("root");
        fs::create_dir_all(&root).expect("create dir");

        let matches = resolve_abbreviation(&[root], "project", true);
        assert!(matches.is_empty());
    }

    #[test]
    fn supports_case_insensitive_matching() {
        let temp: TempDir = temp_dir("abbrev-case");
        let root = temp.path().join("root");
        let target = root.join("Project/Source");
        fs::create_dir_all(&target).expect("create dirs");

        let matches = resolve_abbreviation(&[root], "pro/sou", false);

        assert_eq!(matches, vec![target]);
    }

    #[test]
    fn returns_empty_when_no_segment_path_matches() {
        let temp: TempDir = temp_dir("abbrev-no-match");
        let root = temp.path().join("root");
        fs::create_dir_all(root.join("project/src/components")).expect("create dirs");

        let matches = resolve_abbreviation(&[root], "pro/zzz", true);

        assert!(matches.is_empty());
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
                SegmentToken::Literal("a".to_string()),
                SegmentToken::Gap,
                SegmentToken::Literal("b".to_string()),
                SegmentToken::Delimiter('.'),
                SegmentToken::Literal("c".to_string()),
            ]
        );
    }

    #[test]
    fn parsed_segment_separates_prefix_and_operator_matching() {
        let prefix = ParsedSegment::new("pro", true);
        assert!(prefix.matches("project"));
        assert!(!prefix.matches("my-project"));

        let operator = ParsedSegment::new("p..shell", false);
        assert!(operator.matches("PowerShell"));

        let operator_only = ParsedSegment::new("..", true);
        assert!(!operator_only.matches("project"));
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
