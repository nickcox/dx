pub mod storage;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
        Ok(path)
    }

    pub fn pop(&mut self) -> Result<PathBuf, StackError> {
        let next = self.undo.pop().ok_or(StackError::NothingToPop)?;
        ensure_absolute(&next)?;
        self.cwd = Some(next.clone());
        Ok(next)
    }

    pub fn undo(&mut self) -> Result<PathBuf, StackError> {
        let next = self.undo.pop().ok_or(StackError::NothingToUndo)?;
        ensure_absolute(&next)?;

        if let Some(current) = self.cwd.take() {
            ensure_absolute(&current)?;
            self.redo.push(current);
        }
        self.cwd = Some(next.clone());
        Ok(next)
    }

    pub fn redo(&mut self) -> Result<PathBuf, StackError> {
        let next = self.redo.pop().ok_or(StackError::NothingToRedo)?;
        ensure_absolute(&next)?;

        if let Some(current) = self.cwd.take() {
            ensure_absolute(&current)?;
            self.undo.push(current);
        }
        self.cwd = Some(next.clone());
        Ok(next)
    }

    pub fn sanitize(&mut self) {
        if let Some(path) = self.cwd.as_ref() {
            if !path.is_absolute() {
                self.cwd = None;
            }
        }
        self.undo.retain(|path| path.is_absolute());
        self.redo.retain(|path| path.is_absolute());
    }
}

fn ensure_absolute(path: &PathBuf) -> Result<(), StackError> {
    if path.is_absolute() {
        return Ok(());
    }
    Err(StackError::PathNotAbsolute(path.display().to_string()))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{SessionStack, StackError};

    fn p(path: &str) -> PathBuf {
        PathBuf::from(path)
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
            .push(p("relative/path"))
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
    fn sanitize_drops_relative_entries() {
        let mut stack = SessionStack {
            cwd: Some(p("relative")),
            undo: vec![p("/a"), p("b")],
            redo: vec![p("/c"), p("d")],
        };

        stack.sanitize();

        assert_eq!(stack.cwd, None);
        assert_eq!(stack.undo, vec![p("/a")]);
        assert_eq!(stack.redo, vec![p("/c")]);
    }
}
