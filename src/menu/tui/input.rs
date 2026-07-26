//! Key and mouse events mapped to menu actions, and the refinement the user
//! types while the menu is open.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::widgets::ListState;

use crate::resolve::CompletionCandidates;

use super::QueryFn;
use super::selection::reset_selection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FilterState {
    // Interactive edits can only refine the initial query; they never broaden it.
    pub(super) initial_query: String,
    pub(super) typed_refinement: String,
}

impl FilterState {
    pub(super) fn new(initial_query: &str) -> Self {
        Self {
            initial_query: initial_query.to_string(),
            typed_refinement: String::new(),
        }
    }

    pub(super) fn effective_query(&self) -> String {
        format!("{}{}", self.initial_query, self.typed_refinement)
    }

    pub(super) fn changed_query(&self) -> bool {
        !self.typed_refinement.is_empty()
    }

    pub(super) fn typed_refinement(&self) -> &str {
        &self.typed_refinement
    }

    pub(super) fn push(&mut self, ch: char) {
        self.typed_refinement.push(ch);
    }

    pub(super) fn backspace(&mut self) -> bool {
        self.typed_refinement.pop().is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MenuKeyAction {
    Submit,
    Cancel,
    MoveLinear(isize),
    MoveGridVertical(isize),
    MovePage(isize),
    ScrollRow(isize),
    Backspace,
    InputChar(char),
    Ignore,
}

pub(super) fn map_key_event(key: KeyEvent, use_grid: bool) -> MenuKeyAction {
    match (key.code, key.modifiers) {
        (KeyCode::Enter, _) => MenuKeyAction::Submit,
        (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => MenuKeyAction::Cancel,
        (KeyCode::Right, _) if use_grid => MenuKeyAction::MoveLinear(1),
        (KeyCode::Left, _) if use_grid => MenuKeyAction::MoveLinear(-1),
        (KeyCode::Down, _) if use_grid => MenuKeyAction::MoveGridVertical(1),
        (KeyCode::Up, _) if use_grid => MenuKeyAction::MoveGridVertical(-1),
        (KeyCode::PageDown, _) => MenuKeyAction::MovePage(1),
        (KeyCode::PageUp, _) => MenuKeyAction::MovePage(-1),
        (KeyCode::Tab, KeyModifiers::NONE) if use_grid => MenuKeyAction::MoveLinear(1),
        (KeyCode::BackTab, _) if use_grid => MenuKeyAction::MoveLinear(-1),
        (KeyCode::Down, _) | (KeyCode::Tab, KeyModifiers::NONE) => MenuKeyAction::MoveLinear(1),
        (KeyCode::Up, _) | (KeyCode::BackTab, _) => MenuKeyAction::MoveLinear(-1),
        (KeyCode::Backspace, _) => MenuKeyAction::Backspace,
        (KeyCode::Char(ch), KeyModifiers::NONE) | (KeyCode::Char(ch), KeyModifiers::SHIFT) => {
            if ch.is_control() {
                MenuKeyAction::Ignore
            } else {
                MenuKeyAction::InputChar(ch)
            }
        }
        _ => MenuKeyAction::Ignore,
    }
}

/// Wheel and trackpad gestures move one row and stop at the ends. Only the
/// arrow keys wrap: a gesture has no discrete end, so wrapping would spin the
/// selection past the last candidate on a single flick. Horizontal scroll is
/// ignored; on a trackpad it fires too easily to bind to a selection change.
pub(super) fn map_mouse_event(mouse: MouseEvent) -> MenuKeyAction {
    match mouse.kind {
        MouseEventKind::ScrollUp => MenuKeyAction::ScrollRow(-1),
        MouseEventKind::ScrollDown => MenuKeyAction::ScrollRow(1),
        _ => MenuKeyAction::Ignore,
    }
}

pub(super) fn apply_filter_edit(
    filter_state: &mut FilterState,
    completion: &mut CompletionCandidates,
    list_state: &mut ListState,
    action: MenuKeyAction,
    query_fn: &QueryFn<'_>,
) {
    let query_changed = match action {
        MenuKeyAction::Backspace => filter_state.backspace(),
        MenuKeyAction::InputChar(ch) => {
            filter_state.push(ch);
            true
        }
        _ => return,
    };

    if !query_changed {
        return;
    }

    let query = filter_state.effective_query();
    *completion = query_fn(&query);
    reset_selection(list_state, completion.paths.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;

    use std::path::PathBuf;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

    use ratatui::widgets::ListState;

    use crate::resolve::CompletionCandidates;

    use super::super::QueryFn;

    fn wheel(kind: MouseEventKind) -> MouseEvent {
        MouseEvent {
            kind,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn wheel_moves_one_row_in_either_layout() {
        assert_eq!(
            map_mouse_event(wheel(MouseEventKind::ScrollUp)),
            MenuKeyAction::ScrollRow(-1)
        );
        assert_eq!(
            map_mouse_event(wheel(MouseEventKind::ScrollDown)),
            MenuKeyAction::ScrollRow(1)
        );
    }

    #[test]
    fn other_mouse_events_are_ignored() {
        for kind in [
            MouseEventKind::Moved,
            MouseEventKind::ScrollLeft,
            MouseEventKind::ScrollRight,
            MouseEventKind::Down(crossterm::event::MouseButton::Left),
            MouseEventKind::Up(crossterm::event::MouseButton::Left),
            MouseEventKind::Drag(crossterm::event::MouseButton::Left),
        ] {
            assert_eq!(
                map_mouse_event(wheel(kind)),
                MenuKeyAction::Ignore,
                "{kind:?} should not move the selection"
            );
        }
    }

    #[test]
    fn key_event_mapping_j_is_filter_input_not_navigation() {
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(map_key_event(key, false), MenuKeyAction::InputChar('j'));
    }

    #[test]
    fn key_event_mapping_k_is_filter_input_not_navigation() {
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert_eq!(map_key_event(key, false), MenuKeyAction::InputChar('k'));
    }

    #[test]
    fn key_event_mapping_arrows_remain_navigation() {
        let down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(map_key_event(down, false), MenuKeyAction::MoveLinear(1));
        assert_eq!(map_key_event(up, false), MenuKeyAction::MoveLinear(-1));
    }

    #[test]
    fn key_event_mapping_page_keys_move_by_page() {
        let page_down = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);
        let page_up = KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE);
        assert_eq!(map_key_event(page_down, false), MenuKeyAction::MovePage(1));
        assert_eq!(map_key_event(page_up, false), MenuKeyAction::MovePage(-1));
        assert_eq!(map_key_event(page_down, true), MenuKeyAction::MovePage(1));
        assert_eq!(map_key_event(page_up, true), MenuKeyAction::MovePage(-1));
    }

    #[test]
    fn key_event_mapping_tab_and_backtab_remain_navigation() {
        let tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let backtab = KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT);
        assert_eq!(map_key_event(tab, false), MenuKeyAction::MoveLinear(1));
        assert_eq!(map_key_event(backtab, false), MenuKeyAction::MoveLinear(-1));
    }

    #[test]
    fn filter_state_clamps_backspace_to_initial_query() {
        let mut filter = FilterState::new("Do");
        assert!(!filter.backspace());
        assert_eq!(filter.effective_query(), "Do");

        filter.push('w');
        assert_eq!(filter.effective_query(), "Dow");
        assert!(filter.backspace());
        assert_eq!(filter.effective_query(), "Do");
        assert!(!filter.changed_query());
    }

    #[test]
    fn filter_state_empty_seed_can_return_to_empty() {
        let mut filter = FilterState::new("");
        filter.push('a');
        assert_eq!(filter.effective_query(), "a");
        assert!(filter.changed_query());

        assert!(filter.backspace());
        assert_eq!(filter.effective_query(), "");
        assert!(!filter.changed_query());
        assert!(!filter.backspace());
    }

    #[test]
    fn apply_filter_edit_backspace_at_seed_does_not_requery() {
        let calls = RefCell::new(Vec::new());
        let query_fn: QueryFn<'_> = Box::new(|query| {
            calls.borrow_mut().push(query.to_string());
            CompletionCandidates {
                paths: Vec::new(),
                has_more: false,
            }
        });
        let mut filter = FilterState::new("Do");
        let mut completion = CompletionCandidates {
            paths: vec![PathBuf::from("/tmp/Do")],
            has_more: false,
        };
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        apply_filter_edit(
            &mut filter,
            &mut completion,
            &mut list_state,
            MenuKeyAction::Backspace,
            &query_fn,
        );

        assert!(calls.borrow().is_empty());
        assert_eq!(filter.effective_query(), "Do");
        assert_eq!(completion.paths, vec![PathBuf::from("/tmp/Do")]);
        assert_eq!(list_state.selected(), Some(0));
    }

    #[test]
    fn apply_filter_edit_requeries_with_seed_plus_typed_refinement() {
        let calls = RefCell::new(Vec::new());
        let query_fn: QueryFn<'_> = Box::new(|query| {
            calls.borrow_mut().push(query.to_string());
            CompletionCandidates {
                paths: if query == "Doz" {
                    Vec::new()
                } else {
                    vec![PathBuf::from(format!("/tmp/{query}"))]
                },
                has_more: false,
            }
        });
        let mut filter = FilterState::new("Do");
        let mut completion = CompletionCandidates {
            paths: vec![PathBuf::from("/tmp/Do")],
            has_more: false,
        };
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        apply_filter_edit(
            &mut filter,
            &mut completion,
            &mut list_state,
            MenuKeyAction::InputChar('z'),
            &query_fn,
        );

        assert_eq!(calls.borrow().as_slice(), ["Doz"]);
        assert_eq!(filter.effective_query(), "Doz");
        assert!(completion.paths.is_empty());
        assert_eq!(list_state.selected(), None);
    }
}
