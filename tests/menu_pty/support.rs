//! A real pseudo-terminal for `dx menu`.
//!
//! The menu only runs when it owns a terminal, so every other test reaches it
//! through structural assertions instead. That left its interactive behaviour —
//! scrolling, redraw volume, and handing the terminal back — covered by nothing,
//! which is how a scroll defect shipped in 0.12.0.
//!
//! `--prompt-row` is what makes this affordable: it skips the cursor-position
//! query, so the harness never has to answer `ESC [ 6 n`.

use std::io::{Read, Write};
use std::os::fd::{AsFd, AsRawFd, OwnedFd};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use nix::fcntl::{FcntlArg, OFlag, fcntl};
use nix::poll::{PollFd, PollFlags, PollTimeout, poll};
use nix::pty::{OpenptyResult, Winsize, openpty};

use super::common;

/// Wheel down, SGR encoded. Button 65 is one notch towards the end of the list.
pub const SCROLL_DOWN: &[u8] = b"\x1b[<65;10;5M";
/// Pointer motion, which the menu discards. Only a terminal asked for `?1003h`
/// would ever send these.
pub const POINTER_MOVE: &[u8] = b"\x1b[<35;20;9M";

pub const ROWS: u16 = 40;
pub const COLS: u16 = 120;

/// How long to wait on any single step before declaring the menu wedged. Every
/// wait is bounded so a regression fails the suite instead of hanging it.
const STEP_TIMEOUT: Duration = Duration::from_secs(20);

/// Serialises the tests in this file.
///
/// Several of them assert on how much the menu drew, which only means anything
/// if the menu had the machine to itself; run concurrently, seven terminals
/// rendering thousands of candidates starve each other badly enough that the
/// menu misses its deadline. Mirrors the single shared env lock the unit tests
/// use for the same reason.
fn serial_guard() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub struct MenuPty {
    master: std::fs::File,
    child: Child,
    /// Held for the lifetime of the terminal, so one test finishes before the
    /// next opens one.
    _guard: std::sync::MutexGuard<'static, ()>,
    /// Everything the menu has written, in order.
    pub output: Vec<u8>,
}

impl MenuPty {
    /// Starts `dx menu` against `cwd` with the cursor already at the end of
    /// `buffer`, and waits for the first frame.
    ///
    /// Takes the serial lock and holds it until dropped, so **only one may be
    /// alive at a time** — a test wanting two runs must finish with the first
    /// before starting the second, or it will deadlock against itself.
    pub fn start(buffer: &str, cwd: &std::path::Path) -> Self {
        let guard = serial_guard();
        let winsize = Winsize {
            ws_row: ROWS,
            ws_col: COLS,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let OpenptyResult { master, slave } =
            openpty(&winsize, None).expect("open a pseudo-terminal");

        let child = Self::spawn(buffer, cwd, &slave);
        drop(slave);

        // Non-blocking, so queueing a burst reports a full buffer instead of
        // deadlocking against a menu that is busy rendering.
        fcntl(master.as_raw_fd(), FcntlArg::F_SETFL(OFlag::O_NONBLOCK))
            .expect("set the master non-blocking");

        let mut pty = Self {
            master: std::fs::File::from(master),
            child,
            _guard: guard,
            output: Vec::new(),
        };
        pty.wait_for_first_frame();
        pty
    }

    fn spawn(buffer: &str, cwd: &std::path::Path, slave: &OwnedFd) -> Child {
        let mut command = Command::new(common::dx_bin());
        command
            .args([
                "menu",
                "--buffer",
                buffer,
                "--cursor",
                &buffer.len().to_string(),
                "--shell",
                "bash",
                "--cwd",
                cwd.to_str().expect("utf8 cwd"),
                // Skips the cursor-position query, so the harness does not have
                // to play terminal.
                "--prompt-row",
                "5",
            ])
            .stdin(Stdio::from(slave.try_clone().expect("clone slave")))
            .stdout(Stdio::from(slave.try_clone().expect("clone slave")))
            .stderr(Stdio::from(slave.try_clone().expect("clone slave")));

        let raw = slave.as_raw_fd();
        // SAFETY: only async-signal-safe calls between fork and exec. The menu
        // needs the pty to be its controlling terminal, which means leading a
        // new session and claiming it.
        unsafe {
            command.pre_exec(move || {
                if nix::libc::setsid() == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                if nix::libc::ioctl(raw, nix::libc::TIOCSCTTY as _, 0) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }

        command.spawn().expect("spawn dx menu")
    }

    /// Reads whatever is available, for up to `window`, appending to `output`.
    pub fn pump(&mut self, window: Duration) -> usize {
        let deadline = Instant::now() + window;
        let mut read = 0;

        while Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let millis = u16::try_from(remaining.as_millis().max(1)).unwrap_or(u16::MAX);
            let timeout = PollTimeout::from(millis);
            let mut fds = [PollFd::new(self.master.as_fd(), PollFlags::POLLIN)];
            if poll(&mut fds, timeout).unwrap_or(0) == 0 {
                continue;
            }

            let mut buffer = [0u8; 65536];
            match self.master.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    self.output.extend_from_slice(&buffer[..count]);
                    read += count;
                }
                Err(_) => break,
            }
        }

        read
    }

    /// Waits until the menu stops producing output, so a later measurement sees
    /// only what the test caused.
    ///
    /// Quiet-only: a caller that has just sent discarded input expects to see
    /// nothing, so this must not insist on output arriving.
    pub fn settle(&mut self) {
        let deadline = Instant::now() + STEP_TIMEOUT;
        while Instant::now() < deadline && self.pump(Duration::from_millis(150)) > 0 {}
    }

    /// Waits for the menu to draw at all, then for it to stop.
    ///
    /// [`settle`](Self::settle) alone is not enough at startup: it returns on
    /// the first quiet window, and a process that has not finished starting is
    /// quiet. Under load that produced tests asserting against an empty buffer.
    fn wait_for_first_frame(&mut self) {
        let deadline = Instant::now() + STEP_TIMEOUT;
        while self.output.is_empty() && Instant::now() < deadline {
            self.pump(Duration::from_millis(100));
        }
        assert!(
            !self.output.is_empty(),
            "the menu drew nothing within {STEP_TIMEOUT:?}"
        );
        self.settle();
    }

    /// Sends `event` `count` times, draining output only when the terminal's
    /// input buffer fills. Returns how many were accepted.
    pub fn send_repeated(&mut self, event: &[u8], count: usize) -> usize {
        let mut sent = 0;
        while sent < count {
            match self.master.write_all(event) {
                Ok(()) => sent += 1,
                Err(_) => {
                    self.pump(Duration::from_millis(20));
                }
            }
        }
        sent
    }

    /// Queues `count` copies of `event` in one write, without reading anything
    /// back. This is what a flick looks like: events waiting in the terminal
    /// before the menu has drawn a single frame for any of them.
    ///
    /// One write, and never more than the terminal's input buffer holds. Writing
    /// event by event risks a short write splitting an escape sequence, and a
    /// truncated sequence swallows whatever is sent next — including the Return
    /// that ends the menu.
    pub fn queue_burst(&mut self, event: &[u8], count: usize) -> usize {
        let burst = event.repeat(count);
        assert!(
            burst.len() < 1024,
            "a {}-byte burst risks a short write splitting an event",
            burst.len()
        );
        self.master
            .write_all(&burst)
            .expect("queue the burst in one write");
        count
    }

    pub fn send(&mut self, bytes: &[u8]) {
        let deadline = Instant::now() + STEP_TIMEOUT;
        while Instant::now() < deadline {
            if self.master.write_all(bytes).is_ok() {
                return;
            }
            self.pump(Duration::from_millis(20));
        }
        panic!("could not deliver {bytes:?} to the menu");
    }

    /// Accepts the highlighted candidate and returns the action the menu wrote.
    pub fn accept(&mut self) -> MenuAction {
        self.send(b"\r");
        self.finish()
    }

    fn finish(&mut self) -> MenuAction {
        let deadline = Instant::now() + STEP_TIMEOUT;
        loop {
            self.pump(Duration::from_millis(50));
            match self.child.try_wait().expect("poll the menu") {
                Some(status) => {
                    assert!(status.success(), "dx menu exited with {status}");
                    self.pump(Duration::from_millis(150));
                    return MenuAction::parse(&self.output);
                }
                None if Instant::now() >= deadline => {
                    let _ = self.child.kill();
                    panic!("dx menu never exited");
                }
                None => {}
            }
        }
    }

    /// Bytes written since `settle` was last called, as a proxy for redraws.
    pub fn output_since(&self, mark: usize) -> usize {
        self.output.len().saturating_sub(mark)
    }

    pub fn mark(&self) -> usize {
        self.output.len()
    }

    pub fn wrote(&self, needle: &[u8]) -> bool {
        self.output
            .windows(needle.len())
            .any(|window| window == needle)
    }

    /// Byte offset of `needle`, for asserting the order of escape sequences.
    pub fn position_of(&self, needle: &[u8]) -> Option<usize> {
        self.output
            .windows(needle.len())
            .position(|window| window == needle)
    }
}

impl Drop for MenuPty {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The JSON action the menu writes to stdout, teased back out of the pty where
/// it is interleaved with the frames drawn to stderr.
#[derive(Debug)]
pub struct MenuAction {
    pub action: String,
    pub value: Option<String>,
}

impl MenuAction {
    fn parse(output: &[u8]) -> Self {
        let text = String::from_utf8_lossy(output);
        let start = text
            .rfind(r#"{"action":"#)
            .unwrap_or_else(|| panic!("no menu action in output: {text:?}"));
        let rest = &text[start..];
        let end = rest
            .find('}')
            .unwrap_or_else(|| panic!("unterminated menu action: {rest:?}"));
        let json: serde_json::Value =
            serde_json::from_str(&rest[..=end]).expect("parse the menu action");

        Self {
            action: json["action"].as_str().expect("an action").to_string(),
            value: json["value"].as_str().map(str::to_string),
        }
    }

    /// The final path component the menu chose, which is what the tests assert.
    pub fn selected_name(&self) -> String {
        self.value
            .as_deref()
            .expect("a replace action carries a value")
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .expect("a non-empty value")
            .to_string()
    }
}

/// A directory of `count` predictably named children, wide enough that the menu
/// has to scroll.
pub fn candidate_tree(label: &str, count: usize) -> common::TempDir {
    let temp = common::temp_dir(label);
    for index in 0..count {
        std::fs::create_dir_all(temp.path().join(format!("dir-{index:04}-project")))
            .expect("create a candidate directory");
    }
    temp
}
