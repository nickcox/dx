//! The interactive menu, driven through a real pseudo-terminal.
//!
//! Everything here needs a terminal the menu will agree to draw on, so it is
//! Unix-only — matching the Rust TUI itself, which `docs/shell-setup.md`
//! documents as Unix-only.

#![cfg(unix)]

mod common;

#[path = "menu_pty/support.rs"]
mod support;

use support::{MenuPty, POINTER_MOVE, SCROLL_DOWN, candidate_tree};

/// The baseline nothing else covers: the menu opens on a real terminal, draws,
/// and returns the highlighted candidate.
#[test]
fn menu_returns_the_highlighted_candidate() {
    let temp = candidate_tree("menu-pty-accept", 60);
    let mut pty = MenuPty::start("cd ", temp.path());

    let action = pty.accept();

    assert_eq!(action.action, "replace");
    assert_eq!(action.selected_name(), "dir-0000-project");
}

/// Wheel events move the selection, and each notch moves the same distance.
///
/// One terminal at a time: `MenuPty` holds the serial lock for its lifetime, so
/// overlapping two would deadlock.
#[test]
fn wheel_scrolling_moves_the_selection_down_the_list() {
    let temp = candidate_tree("menu-pty-scroll", 200);

    let selected_after = |notches: usize| -> usize {
        let mut pty = MenuPty::start("cd ", temp.path());
        pty.send_repeated(SCROLL_DOWN, notches);
        pty.settle();
        pty.accept().selected_name()[4..8]
            .parse()
            .expect("an indexed candidate name")
    };

    let after_one = selected_after(1);
    let after_three = selected_after(3);

    assert!(after_one > 0, "one notch should move the selection");
    assert_eq!(
        after_three,
        after_one * 3,
        "three notches should move three times as far as one"
    );
}

/// Scrolling past the end stops at the last candidate rather than wrapping,
/// because a flick has no discrete end.
#[test]
fn scrolling_past_the_end_stops_at_the_last_candidate() {
    let count = 40;
    let temp = candidate_tree("menu-pty-clamp", count);
    let mut pty = MenuPty::start("cd ", temp.path());

    pty.send_repeated(SCROLL_DOWN, count * 2);
    pty.settle();

    assert_eq!(
        pty.accept().selected_name(),
        format!("dir-{:04}-project", count - 1)
    );
}

/// The regression test for the defect this harness was built for.
///
/// A flick queues far more events than there are useful frames. Folding them
/// into one move is the difference between redrawing once and redrawing for
/// every event, which is what made a long list unusable.
#[test]
fn a_queued_scroll_burst_redraws_once_not_once_per_event() {
    let temp = candidate_tree("menu-pty-burst", 2000);
    let mut pty = MenuPty::start("cd ", temp.path());

    let mark = pty.mark();
    let queued = pty.queue_burst(SCROLL_DOWN, 80);

    let action = pty.accept();
    let drawn = pty.output_since(mark);

    // A single frame of this list is a few hundred bytes, so per-event redrawing
    // lands in the tens of kilobytes. Folding keeps it to roughly one frame.
    let per_event = drawn / queued;
    assert!(
        per_event < 100,
        "{drawn} bytes for {queued} queued events ({per_event}/event) suggests \
         one redraw per event rather than one for the run"
    );
    assert_eq!(action.action, "replace");
}

/// Discarded input leaves the drawn frame alone.
///
/// Deliberately not a guard on the cost of a discarded event: the frame is
/// diffed before it is written, so an event that changes nothing emits almost
/// nothing whether or not the menu re-rendered to discover that. The waste is
/// CPU, which output volume cannot see. `menu_asks_for_wheel_tracking_without_
/// motion_reporting` is what keeps these events from arriving at all.
#[test]
fn discarded_pointer_motion_leaves_the_frame_unchanged() {
    let temp = candidate_tree("menu-pty-motion", 200);
    let mut pty = MenuPty::start("cd ", temp.path());

    pty.send_repeated(POINTER_MOVE, 40);
    pty.settle();

    // The selection has not moved, so the first candidate is still the one that
    // gets accepted.
    assert_eq!(pty.accept().selected_name(), "dir-0000-project");
}

/// Wheel tracking without motion reporting, asserted on the wire rather than
/// against the constant, so this covers what the terminal is actually told.
#[test]
fn menu_asks_for_wheel_tracking_without_motion_reporting() {
    let temp = candidate_tree("menu-pty-modes", 40);
    let mut pty = MenuPty::start("cd ", temp.path());
    pty.settle();

    assert!(
        pty.wrote(b"\x1b[?1000h"),
        "the menu needs button tracking to see the wheel"
    );
    for motion_mode in [&b"\x1b[?1002h"[..], &b"\x1b[?1003h"[..]] {
        assert!(
            !pty.wrote(motion_mode),
            "the menu asked for pointer motion it discards: {motion_mode:?}"
        );
    }
}

/// Capture must be released before the menu finishes, and before the cosmetic
/// undoing, so a cleanup that fails part way cannot leave the terminal
/// reporting the mouse to the shell.
#[test]
fn menu_releases_the_mouse_before_the_cosmetic_cleanup() {
    let temp = candidate_tree("menu-pty-release", 40);
    let mut pty = MenuPty::start("cd ", temp.path());
    pty.settle();

    let opened_at = pty
        .position_of(b"\x1b[?1000h")
        .expect("the menu should enable tracking");
    pty.accept();

    let released_at = pty
        .position_of(b"\x1b[?1000l")
        .expect("the menu should release tracking before it exits");
    assert!(
        released_at > opened_at,
        "release must follow the enable it undoes"
    );

    if let Some(cleared_at) = pty.position_of(b"\x1b[2K") {
        assert!(
            released_at < cleared_at || cleared_at < opened_at,
            "the mouse was released after clearing rows, which can fail first"
        );
    }
}

/// Resizing mid-menu ends it instead of drawing at coordinates the screen no
/// longer has.
///
/// The viewport is fixed at the geometry measured before the first frame, and
/// the rows below the prompt were reserved against that, so neither survives a
/// resize. Cancelling is recoverable; painting over the wrong rows is not.
#[test]
fn resizing_the_terminal_cancels_the_menu() {
    let temp = candidate_tree("menu-pty-resize", 200);
    let mut pty = MenuPty::start("cd ", temp.path());

    pty.resize(24, 80);
    let action = pty.wait_for_exit();

    assert_eq!(
        action.action, "cancel",
        "a resize should end the menu without replacing the buffer"
    );
    assert!(
        pty.wrote(b"\x1b[?1000l"),
        "cancelling on resize must still hand the terminal back"
    );
}
