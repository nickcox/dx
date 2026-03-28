pub mod ancestors;
pub mod filter;
pub mod paths;
pub mod recents;
pub mod stack;

use std::fmt;
use std::path::{Component, Path, PathBuf};

use serde::Serialize;

use crate::frecency::FrecencyProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionMode {
    Paths,
    Ancestors,
    Frecents,
    Recents,
    Stack(StackDirection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackDirection {
    Back,
    Forward,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub path: PathBuf,
    pub label: String,
    pub rank: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorError {
    EmptyCandidates,
    OutOfRange { index: usize, total: usize },
    NoMatch(String),
}

impl fmt::Display for SelectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectorError::EmptyCandidates => write!(f, "no candidates available"),
            SelectorError::OutOfRange { index, total } => {
                write!(f, "selector index {index} out of range (1..={total})")
            }
            SelectorError::NoMatch(selector) => {
                write!(f, "selector did not match any candidate: {selector}")
            }
        }
    }
}

#[derive(Debug, Serialize)]
struct JsonCandidate {
    path: String,
    label: String,
    rank: usize,
}

pub fn complete_frecents(provider: &dyn FrecencyProvider, query: Option<&str>) -> Vec<PathBuf> {
    if !provider.is_available() {
        return Vec::new();
    }

    provider.query(query.unwrap_or(""))
}

pub fn select_candidate(
    candidates: &[PathBuf],
    selector: Option<&str>,
) -> Result<PathBuf, SelectorError> {
    if candidates.is_empty() {
        return Err(SelectorError::EmptyCandidates);
    }

    let selector = selector.map(str::trim).filter(|value| !value.is_empty());
    let Some(selector) = selector else {
        return Ok(candidates[0].clone());
    };

    if selector
        .as_bytes()
        .iter()
        .all(|value| value.is_ascii_digit())
    {
        let index = selector.parse::<usize>().unwrap_or(0);
        if index == 0 || index > candidates.len() {
            return Err(SelectorError::OutOfRange {
                index,
                total: candidates.len(),
            });
        }
        return Ok(candidates[index - 1].clone());
    }

    let filtered = filter::filter_candidates(candidates, selector);
    filtered
        .into_iter()
        .next()
        .ok_or_else(|| SelectorError::NoMatch(selector.to_string()))
}

pub fn format_plain(paths: &[PathBuf]) -> String {
    if paths.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    for path in paths {
        output.push_str(&path.display().to_string());
        output.push('\n');
    }
    output
}

pub fn format_json(paths: &[PathBuf]) -> Result<String, serde_json::Error> {
    let payload = to_candidates(paths)
        .into_iter()
        .map(|candidate| JsonCandidate {
            path: candidate.path.display().to_string(),
            label: candidate.label,
            rank: candidate.rank,
        })
        .collect::<Vec<_>>();

    serde_json::to_string(&payload)
}

pub fn to_candidates(paths: &[PathBuf]) -> Vec<Candidate> {
    paths
        .iter()
        .enumerate()
        .map(|(index, path)| Candidate {
            path: path.clone(),
            label: label_for_path(path),
            rank: index + 1,
        })
        .collect()
}

pub fn label_for_path(path: &Path) -> String {
    if path == Path::new("/") {
        return "/".to_string();
    }

    let components = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();

    match components.len() {
        0 => path.display().to_string(),
        1 => components[0].clone(),
        _ => {
            let tail = &components[components.len() - 2..];
            format!("{}/{}", tail[0], tail[1])
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        complete_frecents, format_json, format_plain, label_for_path, select_candidate,
        SelectorError,
    };
    use crate::frecency::FrecencyProvider;

    #[derive(Debug)]
    struct MockProvider {
        available: bool,
        paths: Vec<PathBuf>,
    }

    impl FrecencyProvider for MockProvider {
        fn query(&self, _filter: &str) -> Vec<PathBuf> {
            self.paths.clone()
        }

        fn is_available(&self) -> bool {
            self.available
        }
    }

    #[test]
    fn format_plain_prints_one_path_per_line() {
        let output = format_plain(&[PathBuf::from("/a"), PathBuf::from("/b")]);
        assert_eq!(output, "/a\n/b\n");
    }

    #[test]
    fn format_json_contains_required_fields() {
        let output = format_json(&[PathBuf::from("/home/user/code")]).expect("serialize json");
        assert!(output.contains("\"path\":\"/home/user/code\""));
        assert!(output.contains("\"label\":\"user/code\""));
        assert!(output.contains("\"rank\":1"));
    }

    #[test]
    fn label_generation_uses_path_tail() {
        assert_eq!(
            label_for_path(PathBuf::from("/home/user").as_path()),
            "home/user"
        );
        assert_eq!(label_for_path(PathBuf::from("/home").as_path()), "home");
        assert_eq!(label_for_path(PathBuf::from("/").as_path()), "/");
    }

    #[test]
    fn selector_without_input_picks_first_candidate() {
        let candidates = vec![PathBuf::from("/a"), PathBuf::from("/b")];
        let selected = select_candidate(&candidates, None).expect("select");
        assert_eq!(selected, PathBuf::from("/a"));
    }

    #[test]
    fn selector_with_numeric_value_picks_nth_candidate() {
        let candidates = vec![
            PathBuf::from("/a"),
            PathBuf::from("/b"),
            PathBuf::from("/c"),
        ];
        let selected = select_candidate(&candidates, Some("2")).expect("select");
        assert_eq!(selected, PathBuf::from("/b"));
    }

    #[test]
    fn selector_out_of_range_returns_error() {
        let candidates = vec![PathBuf::from("/a")];
        let err = select_candidate(&candidates, Some("3")).expect_err("must fail");
        assert_eq!(err, SelectorError::OutOfRange { index: 3, total: 1 });
    }

    #[test]
    fn selector_path_match_returns_best_candidate() {
        let candidates = vec![
            PathBuf::from("/home/user/code-review"),
            PathBuf::from("/home/user/code"),
        ];
        let selected = select_candidate(&candidates, Some("code")).expect("select");
        assert_eq!(selected, PathBuf::from("/home/user/code"));
    }

    #[test]
    fn selector_path_with_no_match_fails() {
        let candidates = vec![PathBuf::from("/home/user/code")];
        let err = select_candidate(&candidates, Some("zzz")).expect_err("must fail");
        assert_eq!(err, SelectorError::NoMatch("zzz".to_string()));
    }

    #[test]
    fn frecents_returns_provider_data_when_available() {
        let provider = MockProvider {
            available: true,
            paths: vec![PathBuf::from("/work/a")],
        };
        let output = complete_frecents(&provider, Some("work"));
        assert_eq!(output, vec![PathBuf::from("/work/a")]);
    }

    #[test]
    fn frecents_returns_empty_when_provider_unavailable() {
        let provider = MockProvider {
            available: false,
            paths: vec![PathBuf::from("/work/a")],
        };
        let output = complete_frecents(&provider, Some("work"));
        assert!(output.is_empty());
    }
}
