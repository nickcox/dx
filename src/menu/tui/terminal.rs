//! Terminal ownership for one menu session: raw mode, the cursor-position
//! probe, reserving rows below the prompt, and handing all of it back.

use std::fs::OpenOptions;
use std::io::{BufWriter, Read, Write, stderr};
use std::os::fd::AsFd;
use std::time::{Duration, Instant};

use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, terminal,
};
use nix::sys::select::{FdSet, select as select_fds};
use nix::sys::time::{TimeVal, TimeValLike};
use ratatui::layout::Rect;

#[derive(Clone, Copy)]
pub(super) struct CleanupRegion {
    pub(super) prompt_row: u16,
    pub(super) area: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpaceReservation {
    pub(super) prompt_row: u16,
    pub(super) scroll_rows: u16,
}

pub(super) trait TerminalOps {
    fn size(&self) -> std::io::Result<(u16, u16)>;
    fn cursor_row(&self) -> std::io::Result<u16>;
    fn enable_raw_mode(&self) -> std::io::Result<()>;
    fn disable_raw_mode(&self) -> std::io::Result<()>;
    fn open_output(&self, use_tty_backend: bool) -> std::io::Result<Box<dyn Write>>;
}

pub(super) struct CrosstermTerminalOps;

impl TerminalOps for CrosstermTerminalOps {
    fn size(&self) -> std::io::Result<(u16, u16)> {
        terminal::size()
    }

    fn cursor_row(&self) -> std::io::Result<u16> {
        cursor_row_via_tty(Duration::from_millis(250))
    }

    fn enable_raw_mode(&self) -> std::io::Result<()> {
        terminal::enable_raw_mode()
    }

    fn disable_raw_mode(&self) -> std::io::Result<()> {
        terminal::disable_raw_mode()
    }

    fn open_output(&self, use_tty_backend: bool) -> std::io::Result<Box<dyn Write>> {
        if use_tty_backend {
            Ok(Box::new(BufWriter::new(
                OpenOptions::new().write(true).open("/dev/tty")?,
            )))
        } else {
            Ok(Box::new(stderr()))
        }
    }
}

pub(super) fn cursor_row_via_tty(timeout: Duration) -> std::io::Result<u16> {
    let mut tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
    tty.write_all(b"\x1b[6n")?;
    tty.flush()?;

    let deadline = Instant::now() + timeout;
    let mut response = Vec::with_capacity(16);
    while response.len() < 32 {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }

        let mut read_fds = FdSet::new();
        read_fds.insert(tty.as_fd());
        let timeout_ms = i64::try_from(remaining.as_millis().max(1)).unwrap_or(i64::MAX);
        let mut select_timeout = TimeVal::milliseconds(timeout_ms);
        if select_fds(
            None,
            Some(&mut read_fds),
            None,
            None,
            Some(&mut select_timeout),
        )
        .map_err(std::io::Error::other)?
            == 0
        {
            break;
        }

        let mut byte = [0u8; 1];
        tty.read_exact(&mut byte)?;
        response.push(byte[0]);
        if byte[0] == b'R' {
            return parse_cursor_row_response(&response).ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "terminal returned an invalid cursor position",
                )
            });
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "terminal did not report its cursor position",
    ))
}

pub(super) fn parse_cursor_row_response(response: &[u8]) -> Option<u16> {
    let response = std::str::from_utf8(response).ok()?;
    let position = response.rsplit_once("\x1b[")?.1.strip_suffix('R')?;
    let (row, _column) = position.split_once(';')?;
    row.parse::<u16>().ok()?.checked_sub(1)
}

pub(super) struct TerminalSession<'a> {
    pub(super) terminal_ops: &'a dyn TerminalOps,
    pub(super) use_tty_backend: bool,
    pub(super) output: Option<Box<dyn Write>>,
    pub(super) cursor_hidden: bool,
    pub(super) mouse_captured: bool,
    pub(super) reservation: Option<SpaceReservation>,
    pub(super) cleanup_region: Option<CleanupRegion>,
}

impl<'a> TerminalSession<'a> {
    pub(super) fn start(
        terminal_ops: &'a dyn TerminalOps,
        use_tty_backend: bool,
    ) -> std::io::Result<Self> {
        terminal_ops.enable_raw_mode()?;
        Ok(Self {
            terminal_ops,
            use_tty_backend,
            output: None,
            cursor_hidden: false,
            mouse_captured: false,
            reservation: None,
            cleanup_region: None,
        })
    }

    pub(super) fn output(&mut self) -> std::io::Result<&mut Box<dyn Write>> {
        if self.output.is_none() {
            self.output = Some(self.terminal_ops.open_output(self.use_tty_backend)?);
        }

        Ok(self.output.as_mut().expect("output initialized"))
    }

    pub(super) fn reserve_space(
        &mut self,
        prompt_row: u16,
        terminal_rows: u16,
        needed_height: u16,
    ) -> std::io::Result<SpaceReservation> {
        let scroll_needed = scroll_rows_needed(prompt_row, terminal_rows, needed_height);
        if scroll_needed == 0 {
            return Ok(SpaceReservation {
                prompt_row,
                scroll_rows: 0,
            });
        }

        let reservation =
            scroll_terminal(self.output()?, prompt_row, terminal_rows, scroll_needed)?;
        self.reservation = Some(reservation);
        Ok(reservation)
    }

    pub(super) fn set_cleanup_region(&mut self, prompt_row: u16, area: Rect) {
        self.cleanup_region = Some(CleanupRegion { prompt_row, area });
    }

    /// Without this the terminal keeps the wheel to itself and scrolls its own
    /// scrollback, so the menu never sees the gesture.
    pub(super) fn capture_mouse(&mut self) -> std::io::Result<()> {
        execute!(self.output()?, EnableMouseCapture)?;
        self.mouse_captured = true;
        self.output()?.flush()?;
        Ok(())
    }

    pub(super) fn hide_cursor(&mut self) -> std::io::Result<()> {
        execute!(self.output()?, cursor::Hide)?;
        self.cursor_hidden = true;
        self.output()?.flush()?;
        Ok(())
    }
}

impl Drop for TerminalSession<'_> {
    fn drop(&mut self) {
        let acquired = AcquiredState {
            cleanup_region: self.cleanup_region,
            reservation: self.reservation,
            cursor_hidden: self.cursor_hidden,
            mouse_captured: self.mouse_captured,
        };
        if !acquired.is_empty()
            && let Some(output) = self.output.as_mut()
        {
            restore_terminal_output(output, acquired);
        }

        let _ = self.terminal_ops.disable_raw_mode();
    }
}

pub(super) fn scroll_terminal<W: Write>(
    output: &mut W,
    prompt_row: u16,
    terminal_rows: u16,
    scroll_needed: u16,
) -> std::io::Result<SpaceReservation> {
    let next_prompt_row = prompt_row.saturating_sub(scroll_needed);
    execute!(
        output,
        cursor::MoveTo(0, terminal_rows.saturating_sub(1)),
        terminal::ScrollUp(scroll_needed),
        cursor::MoveTo(0, next_prompt_row)
    )?;
    output.flush()?;
    Ok(SpaceReservation {
        prompt_row: next_prompt_row,
        scroll_rows: scroll_needed,
    })
}

/// What the session acquired and must hand back.
#[derive(Default, Clone, Copy)]
pub(super) struct AcquiredState {
    pub(super) cleanup_region: Option<CleanupRegion>,
    pub(super) reservation: Option<SpaceReservation>,
    pub(super) cursor_hidden: bool,
    pub(super) mouse_captured: bool,
}

impl AcquiredState {
    pub(super) fn is_empty(self) -> bool {
        self.cleanup_region.is_none()
            && self.reservation.is_none()
            && !self.cursor_hidden
            && !self.mouse_captured
    }
}

pub(super) fn restore_terminal_output<W: Write>(output: &mut W, state: AcquiredState) {
    let AcquiredState {
        cleanup_region,
        reservation,
        cursor_hidden,
        mouse_captured,
    } = state;
    if let Some(region) = cleanup_region {
        let _ = execute!(output, cursor::MoveTo(0, region.prompt_row));
        for row in region.prompt_row.saturating_add(1)..region.area.bottom() {
            let _ = execute!(
                output,
                cursor::MoveTo(0, row),
                terminal::Clear(terminal::ClearType::CurrentLine)
            );
        }
        let _ = execute!(output, cursor::MoveTo(0, region.prompt_row));
    } else if let Some(reservation) = reservation {
        let _ = execute!(output, cursor::MoveTo(0, reservation.prompt_row));
    }

    if cursor_hidden {
        let _ = execute!(output, cursor::Show);
    }
    // Left enabled, the terminal would keep reporting clicks as input once dx exits.
    if mouse_captured {
        let _ = execute!(output, DisableMouseCapture);
    }
    let _ = output.flush();
}

pub(super) fn prompt_gap_rows(show_border: bool) -> u16 {
    if show_border { 0 } else { 1 }
}

pub(super) fn required_rows_below(rendered_height: u16, show_border: bool) -> u16 {
    rendered_height.saturating_add(prompt_gap_rows(show_border))
}

pub(super) fn clamp_prompt_row(prompt_row: u16, terminal_rows: u16) -> u16 {
    prompt_row.min(terminal_rows.saturating_sub(1))
}

pub(super) fn menu_top_row(
    prompt_row: u16,
    terminal_rows: u16,
    rendered_height: u16,
    show_border: bool,
) -> u16 {
    prompt_row
        .saturating_add(1)
        .saturating_add(prompt_gap_rows(show_border))
        .min(terminal_rows.saturating_sub(rendered_height))
}

pub(super) fn scroll_rows_needed(prompt_row: u16, terminal_rows: u16, needed_height: u16) -> u16 {
    let rows_below = terminal_rows.saturating_sub(prompt_row + 1);
    needed_height.saturating_sub(rows_below)
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::*;

    /// Capture left on would make the terminal report clicks as input after exit.
    #[test]
    fn restore_disables_mouse_capture_it_enabled() {
        let mut output = Vec::new();
        restore_terminal_output(
            &mut output,
            AcquiredState {
                mouse_captured: true,
                ..AcquiredState::default()
            },
        );
        let written = String::from_utf8_lossy(&output).into_owned();

        let mut expected = Vec::new();
        execute!(&mut expected, DisableMouseCapture).expect("write escape");
        assert!(
            written.contains(&String::from_utf8_lossy(&expected).into_owned()),
            "restore did not disable mouse capture: {written:?}"
        );

        let mut untouched = Vec::new();
        restore_terminal_output(&mut untouched, AcquiredState::default());
        assert!(
            untouched.is_empty(),
            "restore wrote {untouched:?} without having acquired anything"
        );
    }

    #[test]
    fn successful_scroll_reports_reserved_prompt_geometry() {
        let mut output = Vec::new();
        let reservation = scroll_terminal(&mut output, 23, 24, 10).expect("scroll should succeed");

        assert_eq!(
            reservation,
            SpaceReservation {
                prompt_row: 13,
                scroll_rows: 10,
            }
        );
        assert!(!output.is_empty());
    }

    #[test]
    fn reservation_without_scroll_reports_zero_scroll_rows() {
        let terminal_ops = MockTerminalOps::new();
        let mut session = TerminalSession::start(&terminal_ops, false).expect("start session");

        let reservation = session
            .reserve_space(5, 24, 10)
            .expect("reserve existing rows");

        assert_eq!(
            reservation,
            SpaceReservation {
                prompt_row: 5,
                scroll_rows: 0,
            }
        );
        assert_eq!(terminal_ops.open_calls.get(), 0);
    }

    #[test]
    fn failed_scroll_flush_does_not_produce_a_reservation() {
        let error = scroll_terminal(&mut FlushFailure, 23, 24, 10)
            .expect_err("flush failure should fail reservation");

        assert_eq!(error.kind(), std::io::ErrorKind::Other);
    }

    #[test]
    fn restoration_without_acquired_state_emits_nothing() {
        let mut output = Vec::new();
        restore_terminal_output(&mut output, AcquiredState::default());
        assert!(output.is_empty());
    }

    #[test]
    fn restoration_only_shows_cursor_when_session_hid_it() {
        let region = CleanupRegion {
            prompt_row: 2,
            area: Rect::new(0, 3, 80, 3),
        };
        let mut hidden_output = Vec::new();
        restore_terminal_output(
            &mut hidden_output,
            AcquiredState {
                cleanup_region: Some(region),
                cursor_hidden: true,
                ..AcquiredState::default()
            },
        );
        assert!(hidden_output.windows(6).any(|bytes| bytes == b"\x1b[?25h"));

        let mut visible_output = Vec::new();
        restore_terminal_output(
            &mut visible_output,
            AcquiredState {
                cleanup_region: Some(region),
                ..AcquiredState::default()
            },
        );
        assert!(!visible_output.windows(6).any(|bytes| bytes == b"\x1b[?25h"));
        assert!(visible_output.windows(4).any(|bytes| bytes == b"\x1b[2K"));
    }

    #[test]
    fn borderless_menu_adds_prompt_gap_row() {
        assert_eq!(prompt_gap_rows(true), 0);
        assert_eq!(prompt_gap_rows(false), 1);
        assert_eq!(required_rows_below(12, true), 12);
        assert_eq!(required_rows_below(12, false), 13);
    }

    #[test]
    fn prompt_row_is_clamped_to_terminal_bounds() {
        assert_eq!(clamp_prompt_row(5, 24), 5);
        assert_eq!(clamp_prompt_row(30, 24), 23);
        assert_eq!(clamp_prompt_row(0, 0), 0);
    }

    #[test]
    fn cursor_position_response_parses_zero_based_row() {
        assert_eq!(parse_cursor_row_response(b"\x1b[6;42R"), Some(5));
        assert_eq!(parse_cursor_row_response(b"noise\x1b[24;1R"), Some(23));
        assert_eq!(parse_cursor_row_response(b"\x1b[0;1R"), None);
        assert_eq!(parse_cursor_row_response(b"invalid"), None);
    }

    #[test]
    fn measured_prompt_near_top_does_not_scroll_when_menu_fits() {
        let prompt_row = clamp_prompt_row(5, 24);
        let scroll_needed = scroll_rows_needed(prompt_row, 24, 10);

        assert_eq!(scroll_needed, 0);
        assert_eq!(menu_top_row(prompt_row, 24, 10, true), 6);
    }

    #[test]
    fn menu_top_row_leaves_blank_line_when_borderless() {
        assert_eq!(menu_top_row(5, 24, 10, true), 6);
        assert_eq!(menu_top_row(5, 24, 10, false), 7);
    }

    #[test]
    fn scroll_rows_needed_only_when_not_enough_rows_below() {
        assert_eq!(scroll_rows_needed(5, 24, 10), 0);
        assert_eq!(scroll_rows_needed(20, 24, 10), 7);
        assert_eq!(scroll_rows_needed(23, 24, 2), 2);
    }
}
