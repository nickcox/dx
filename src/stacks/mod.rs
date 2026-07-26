pub mod storage;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Deepest history kept per direction. A scripted `cd` loop adds thousands of
/// entries a minute, and past ~45k both `dx stack push` and `back <TAB>` exceed
/// 10 ms; 5000 stays well under that while keeping far more history than anyone
/// navigates through.
pub const MAX_DEPTH: usize = 5000;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionStack {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    #[serde(default)]
    pub undo: Vec<PathBuf>,
    #[serde(default)]
    pub redo: Vec<PathBuf>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum StackError {
    #[error("path must be absolute: {0}")]
    PathNotAbsolute(String),
    #[error("nothing to pop")]
    NothingToPop,
    #[error("nothing to undo")]
    NothingToUndo,
    #[error("nothing to redo")]
    NothingToRedo,
}

impl SessionStack {
    pub fn validate(&self) -> Result<(), StackError> {
        if let Some(cwd) = &self.cwd {
            ensure_absolute(cwd)?;
        }
        for path in self.undo.iter().chain(&self.redo) {
            ensure_absolute(path)?;
        }
        Ok(())
    }

    pub fn push(&mut self, path: PathBuf) -> Result<PathBuf, StackError> {
        ensure_absolute(&path)?;

        if self.cwd.as_ref() == Some(&path) {
            return Ok(path);
        }

        if let Some(previous) = self.cwd.take() {
            self.undo.push(previous);
        }
        self.cwd = Some(path.clone());
        self.redo.clear();
        self.truncate_to_max_depth();
        Ok(path)
    }

    /// Drops the oldest entries once either direction exceeds [`MAX_DEPTH`].
    /// Without this the file grows for the life of the shell, and every prompt
    /// reads and rewrites all of it.
    fn truncate_to_max_depth(&mut self) {
        for history in [&mut self.undo, &mut self.redo] {
            if let Some(excess) = history.len().checked_sub(MAX_DEPTH)
                && excess > 0
            {
                history.drain(..excess);
            }
        }
    }

    pub fn pop(&mut self) -> Result<PathBuf, StackError> {
        let next = self.undo.last().ok_or(StackError::NothingToPop)?;
        ensure_absolute(next)?;
        let next = self.undo.pop().expect("checked non-empty undo stack");
        self.cwd = Some(next.clone());
        Ok(next)
    }

    pub fn undo(&mut self) -> Result<PathBuf, StackError> {
        let next = self.undo.last().ok_or(StackError::NothingToUndo)?;
        ensure_absolute(next)?;
        if let Some(current) = &self.cwd {
            ensure_absolute(current)?;
        }
        let next = self.undo.pop().expect("checked non-empty undo stack");

        if let Some(current) = self.cwd.take() {
            self.redo.push(current);
        }
        self.cwd = Some(next.clone());
        Ok(next)
    }

    pub fn redo(&mut self) -> Result<PathBuf, StackError> {
        let next = self.redo.last().ok_or(StackError::NothingToRedo)?;
        ensure_absolute(next)?;
        if let Some(current) = &self.cwd {
            ensure_absolute(current)?;
        }
        let next = self.redo.pop().expect("checked non-empty redo stack");

        if let Some(current) = self.cwd.take() {
            self.undo.push(current);
        }
        self.cwd = Some(next.clone());
        Ok(next)
    }

    pub fn sanitize(&mut self) {
        if let Some(path) = self.cwd.as_ref()
            && !path.is_absolute()
        {
            self.cwd = None;
        }
        self.undo.retain(|path| path.is_absolute());
        self.redo.retain(|path| path.is_absolute());
        self.truncate_to_max_depth();
    }
}

fn ensure_absolute(path: &Path) -> Result<(), StackError> {
    if path.is_absolute() {
        return Ok(());
    }
    Err(StackError::PathNotAbsolute(path.display().to_string()))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{MAX_DEPTH, SessionStack, StackError};

    fn p(path: &str) -> PathBuf {
        std::env::temp_dir().join(path.trim_start_matches('/'))
    }

    #[test]
    fn serde_round_trip_preserves_stack_state() {
        let stack = SessionStack {
            cwd: Some(p("/a")),
            undo: vec![p("/b"), p("/c")],
            redo: vec![p("/d")],
        };

        let raw = serde_json::to_string(&stack).expect("serialize stack");
        let parsed = serde_json::from_str::<SessionStack>(&raw).expect("deserialize stack");
        assert_eq!(parsed, stack);
    }

    #[test]
    fn push_onto_empty_session_sets_cwd() {
        let mut stack = SessionStack::default();
        let output = stack.push(p("/home/user")).expect("push");

        assert_eq!(output, p("/home/user"));
        assert_eq!(stack.cwd, Some(p("/home/user")));
        assert!(stack.undo.is_empty());
        assert!(stack.redo.is_empty());
    }

    #[test]
    fn push_with_existing_history_moves_cwd_to_undo_and_clears_redo() {
        let mut stack = SessionStack {
            cwd: Some(p("/a")),
            undo: vec![p("/b")],
            redo: vec![p("/c")],
        };

        let output = stack.push(p("/d")).expect("push");

        assert_eq!(output, p("/d"));
        assert_eq!(stack.cwd, Some(p("/d")));
        assert_eq!(stack.undo, vec![p("/b"), p("/a")]);
        assert!(stack.redo.is_empty());
    }

    #[test]
    fn push_duplicate_is_no_op_and_preserves_redo() {
        let mut stack = SessionStack {
            cwd: Some(p("/a")),
            undo: vec![p("/b")],
            redo: vec![p("/c")],
        };

        let output = stack.push(p("/a")).expect("push");

        assert_eq!(output, p("/a"));
        assert_eq!(stack.cwd, Some(p("/a")));
        assert_eq!(stack.undo, vec![p("/b")]);
        assert_eq!(stack.redo, vec![p("/c")]);
    }

    #[test]
    fn push_rejects_relative_path() {
        let mut stack = SessionStack::default();
        let err = stack
            .push(PathBuf::from("relative/path"))
            .expect_err("relative path fails");
        assert!(matches!(err, StackError::PathNotAbsolute(_)));
    }

    #[test]
    fn pop_returns_top_undo_without_touching_redo() {
        let mut stack = SessionStack {
            cwd: Some(p("/a")),
            undo: vec![p("/b"), p("/c")],
            redo: vec![p("/d")],
        };

        let output = stack.pop().expect("pop");

        assert_eq!(output, p("/c"));
        assert_eq!(stack.cwd, Some(p("/c")));
        assert_eq!(stack.undo, vec![p("/b")]);
        assert_eq!(stack.redo, vec![p("/d")]);
    }

    #[test]
    fn pop_fails_when_undo_empty() {
        let mut stack = SessionStack {
            cwd: Some(p("/a")),
            ..SessionStack::default()
        };
        let err = stack.pop().expect_err("pop fails");
        assert_eq!(err, StackError::NothingToPop);
    }

    #[test]
    fn undo_moves_cwd_to_redo_and_restores_previous_entry() {
        let mut stack = SessionStack {
            cwd: Some(p("/a")),
            undo: vec![p("/b"), p("/c")],
            ..SessionStack::default()
        };

        let output = stack.undo().expect("undo");

        assert_eq!(output, p("/c"));
        assert_eq!(stack.cwd, Some(p("/c")));
        assert_eq!(stack.undo, vec![p("/b")]);
        assert_eq!(stack.redo, vec![p("/a")]);
    }

    #[test]
    fn consecutive_undos_build_redo_stack() {
        let mut stack = SessionStack {
            cwd: Some(p("/a")),
            undo: vec![p("/b"), p("/c")],
            ..SessionStack::default()
        };

        let first = stack.undo().expect("first undo");
        let second = stack.undo().expect("second undo");

        assert_eq!(first, p("/c"));
        assert_eq!(second, p("/b"));
        assert_eq!(stack.cwd, Some(p("/b")));
        assert!(stack.undo.is_empty());
        assert_eq!(stack.redo, vec![p("/a"), p("/c")]);
    }

    #[test]
    fn undo_fails_when_no_history() {
        let mut stack = SessionStack {
            cwd: Some(p("/a")),
            ..SessionStack::default()
        };
        let err = stack.undo().expect_err("undo fails");
        assert_eq!(err, StackError::NothingToUndo);
    }

    #[test]
    fn redo_restores_forward_position() {
        let mut stack = SessionStack {
            cwd: Some(p("/c")),
            undo: vec![p("/b")],
            redo: vec![p("/a")],
        };

        let output = stack.redo().expect("redo");

        assert_eq!(output, p("/a"));
        assert_eq!(stack.cwd, Some(p("/a")));
        assert_eq!(stack.undo, vec![p("/b"), p("/c")]);
        assert!(stack.redo.is_empty());
    }

    #[test]
    fn redo_fails_when_no_future_history() {
        let mut stack = SessionStack {
            cwd: Some(p("/a")),
            undo: vec![p("/b")],
            ..SessionStack::default()
        };
        let err = stack.redo().expect_err("redo fails");
        assert_eq!(err, StackError::NothingToRedo);
    }

    #[test]
    fn push_caps_history_and_drops_the_oldest_entries() {
        let mut stack = SessionStack::default();
        for index in 0..(MAX_DEPTH + 50) {
            stack.push(p(&format!("/dir{index}"))).expect("push");
        }

        assert_eq!(stack.undo.len(), MAX_DEPTH);
        // The oldest entries go; the most recent history is what `back` needs.
        // The first push has no previous cwd to record, so MAX_DEPTH + 50 pushes
        // leave MAX_DEPTH + 49 entries and drain 49 of them.
        assert_eq!(stack.undo.first(), Some(&p("/dir49")));
        assert_eq!(
            stack.undo.last(),
            Some(&p(&format!("/dir{}", MAX_DEPTH + 48)))
        );
        assert_eq!(stack.cwd, Some(p(&format!("/dir{}", MAX_DEPTH + 49))));
    }

    #[test]
    fn undo_still_walks_back_through_a_capped_stack() {
        let mut stack = SessionStack::default();
        for index in 0..(MAX_DEPTH + 10) {
            stack.push(p(&format!("/dir{index}"))).expect("push");
        }

        // Most recent first, exactly as before capping.
        assert_eq!(
            stack.undo().expect("undo"),
            p(&format!("/dir{}", MAX_DEPTH + 8))
        );
        assert_eq!(
            stack.undo().expect("undo"),
            p(&format!("/dir{}", MAX_DEPTH + 7))
        );
    }

    #[test]
    fn sanitize_trims_an_oversized_stack_from_an_earlier_version() {
        let mut stack = SessionStack {
            cwd: Some(p("/now")),
            undo: (0..MAX_DEPTH * 3).map(|i| p(&format!("/old{i}"))).collect(),
            redo: Vec::new(),
        };

        stack.sanitize();

        assert_eq!(stack.undo.len(), MAX_DEPTH);
        assert_eq!(
            stack.undo.last(),
            Some(&p(&format!("/old{}", MAX_DEPTH * 3 - 1)))
        );
    }

    #[test]
    fn sanitize_drops_relative_entries() {
        let mut stack = SessionStack {
            cwd: Some(PathBuf::from("relative")),
            undo: vec![p("/a"), PathBuf::from("b")],
            redo: vec![p("/c"), PathBuf::from("d")],
        };

        stack.sanitize();

        assert_eq!(stack.cwd, None);
        assert_eq!(stack.undo, vec![p("/a")]);
        assert_eq!(stack.redo, vec![p("/c")]);
    }

    #[test]
    fn failed_undo_leaves_invalid_stack_unchanged() {
        let mut stack = SessionStack {
            cwd: Some(PathBuf::from("relative")),
            undo: vec![p("/previous")],
            redo: Vec::new(),
        };
        let original = stack.clone();

        assert!(matches!(stack.undo(), Err(StackError::PathNotAbsolute(_))));
        assert_eq!(stack, original);
    }

    #[test]
    fn failed_redo_leaves_invalid_stack_unchanged() {
        let mut stack = SessionStack {
            cwd: Some(p("/current")),
            undo: Vec::new(),
            redo: vec![PathBuf::from("relative")],
        };
        let original = stack.clone();

        assert!(matches!(stack.redo(), Err(StackError::PathNotAbsolute(_))));
        assert_eq!(stack, original);
    }
}
