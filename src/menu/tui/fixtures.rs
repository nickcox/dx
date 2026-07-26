//! Test doubles shared by the terminal, session and layout test modules: a
//! scriptable `TerminalOps` that records what was written, plus candidate and
//! options builders.

use std::cell::{Cell, RefCell};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::resolve::CompletionCandidates;

use super::session::select_with_terminal_ops;
use super::terminal::TerminalOps;
use super::{MenuOptions, MenuRequest, MenuResult};

#[derive(Default)]
pub(super) struct WriterState {
    pub(super) bytes: Vec<u8>,
    pub(super) fail_write: bool,
    pub(super) fail_flush: bool,
}

pub(super) struct RecordingWriter {
    pub(super) state: Rc<RefCell<WriterState>>,
}

impl Write for RecordingWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let mut state = self.state.borrow_mut();
        if state.fail_write {
            return Err(std::io::Error::other("injected write failure"));
        }
        state.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.state.borrow().fail_flush {
            Err(std::io::Error::other("injected flush failure"))
        } else {
            Ok(())
        }
    }
}

pub(super) struct MockTerminalOps {
    pub(super) size: (u16, u16),
    pub(super) cursor_row: u16,
    pub(super) fail_enable: bool,
    pub(super) fail_cursor_row: bool,
    pub(super) fail_open_at: Option<usize>,
    pub(super) size_calls: Cell<usize>,
    pub(super) cursor_row_calls: Cell<usize>,
    pub(super) enable_calls: Cell<usize>,
    pub(super) disable_calls: Cell<usize>,
    pub(super) open_calls: Cell<usize>,
    pub(super) writer_state: Rc<RefCell<WriterState>>,
}

impl MockTerminalOps {
    pub(super) fn new() -> Self {
        Self {
            size: (80, 24),
            cursor_row: 23,
            fail_enable: false,
            fail_cursor_row: false,
            fail_open_at: None,
            size_calls: Cell::new(0),
            cursor_row_calls: Cell::new(0),
            enable_calls: Cell::new(0),
            disable_calls: Cell::new(0),
            open_calls: Cell::new(0),
            writer_state: Rc::new(RefCell::new(WriterState::default())),
        }
    }

    pub(super) fn output_contains(&self, sequence: &[u8]) -> bool {
        self.writer_state
            .borrow()
            .bytes
            .windows(sequence.len())
            .any(|bytes| bytes == sequence)
    }

    pub(super) fn output_contains_scroll_up(&self) -> bool {
        let state = self.writer_state.borrow();
        let bytes = &state.bytes;
        bytes.iter().enumerate().any(|(index, byte)| {
            *byte == b'S'
                && bytes[..index]
                    .iter()
                    .rev()
                    .take_while(|byte| byte.is_ascii_digit())
                    .count()
                    > 0
        })
    }
}

impl TerminalOps for MockTerminalOps {
    fn size(&self) -> std::io::Result<(u16, u16)> {
        self.size_calls.set(self.size_calls.get() + 1);
        Ok(self.size)
    }

    fn cursor_row(&self) -> std::io::Result<u16> {
        self.cursor_row_calls.set(self.cursor_row_calls.get() + 1);
        if self.fail_cursor_row {
            Err(std::io::Error::other("injected cursor row failure"))
        } else {
            Ok(self.cursor_row)
        }
    }

    fn enable_raw_mode(&self) -> std::io::Result<()> {
        self.enable_calls.set(self.enable_calls.get() + 1);
        if self.fail_enable {
            Err(std::io::Error::other("injected raw mode failure"))
        } else {
            Ok(())
        }
    }

    fn disable_raw_mode(&self) -> std::io::Result<()> {
        self.disable_calls.set(self.disable_calls.get() + 1);
        Ok(())
    }

    fn open_output(&self, _use_tty_backend: bool) -> std::io::Result<Box<dyn Write>> {
        let call = self.open_calls.get() + 1;
        self.open_calls.set(call);
        if self.fail_open_at == Some(call) {
            return Err(std::io::Error::other("injected output open failure"));
        }
        Ok(Box::new(RecordingWriter {
            state: Rc::clone(&self.writer_state),
        }))
    }
}

pub(super) fn candidates(count: usize) -> CompletionCandidates {
    CompletionCandidates {
        paths: (0..count)
            .map(|index| PathBuf::from(format!("/tmp/candidate-{index}")))
            .collect(),
        has_more: false,
    }
}

/// Presentation defaults used by the layout and terminal-setup tests:
/// 10 rows, no truncation, no border, stderr backend, no colours.
pub(super) fn test_options() -> MenuOptions {
    MenuOptions {
        max_rows: 10,
        ..MenuOptions::default()
    }
}

pub(super) fn borderless_options(item_max_len: Option<usize>) -> MenuOptions {
    MenuOptions {
        max_rows: 10,
        item_max_len,
        ..MenuOptions::default()
    }
}

pub(super) fn bordered_options(item_max_len: Option<usize>) -> MenuOptions {
    MenuOptions {
        max_rows: 10,
        item_max_len,
        show_border: true,
        ..MenuOptions::default()
    }
}

pub(super) fn select_with_mock(
    terminal_ops: &MockTerminalOps,
    candidate_count: usize,
    prompt_row_override: Option<u16>,
) -> Option<MenuResult> {
    select_with_terminal_ops(
        MenuRequest {
            candidates: candidates(candidate_count),
            query: "",
            mode: crate::menu::MenuMode::Path,
            cwd: Path::new("/tmp"),
            prompt_row: prompt_row_override,
            query_fn: Box::new(|_| panic!("setup failure must not re-query")),
        },
        &test_options(),
        terminal_ops,
    )
}

pub(super) struct FlushFailure;

impl Write for FlushFailure {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::other("injected flush failure"))
    }
}
