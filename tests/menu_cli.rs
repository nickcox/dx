//! `dx menu` integration tests, split by what is under test.
//!
//! Submodules rather than separate test targets: every extra `tests/*.rs` is
//! another binary to link, roughly a second each on an incremental test run.
//! The `#[path]` attributes are needed because a test-target root resolves `mod`
//! against `tests/`, where each file would become a target of its own.

mod common;

#[path = "menu_cli/support.rs"]
mod support;

#[path = "menu_cli/hook_behaviour.rs"]
mod hook_behaviour;
#[path = "menu_cli/hooks.rs"]
mod hooks;
#[path = "menu_cli/hooks_pwsh.rs"]
mod hooks_pwsh;
#[path = "menu_cli/interactive.rs"]
mod interactive;
#[path = "menu_cli/noop.rs"]
mod noop;
