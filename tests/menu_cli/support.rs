//! Helpers shared by the `dx menu` integration tests.

use std::ffi::OsString;
#[cfg(unix)]
use std::io::Write;
#[cfg(not(unix))]
use std::process::Command;
#[cfg(unix)]
use std::process::{Command, Stdio};

use super::common;

pub(crate) fn dx() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dx"))
}

pub(crate) fn pwsh_available() -> bool {
    common::optional_tool_available("pwsh")
}

pub(crate) fn path_with_dx_binary() -> OsString {
    let mut paths =
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect::<Vec<_>>();
    let dx_binary = std::path::PathBuf::from(env!("CARGO_BIN_EXE_dx"));
    if let Some(dx_dir) = dx_binary.parent()
        && !paths.iter().any(|existing| existing == dx_dir)
    {
        paths.insert(0, dx_dir.to_path_buf());
    }
    std::env::join_paths(paths).expect("join PATH entries")
}

#[cfg(unix)]
pub(crate) fn assert_hook_parses_with(shell: &str, command: &str, args: &[&str]) {
    if command == "bash" {
        assert!(
            common::tool_available(command),
            "bash is required for syntax checks"
        );
    } else if !common::optional_tool_available(command) {
        return;
    }

    let generated = dx()
        .args(["init", shell, "--menu"])
        .env(
            "DX_MENU_COMMAND_MAPPINGS",
            "Get-ChildItem=path,git.status=file",
        )
        .output()
        .expect("generate hook for syntax check");
    assert!(
        generated.status.success(),
        "failed to generate {shell} hook: {}",
        String::from_utf8_lossy(&generated.stderr)
    );

    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn shell syntax checker");
    child
        .stdin
        .take()
        .expect("syntax checker stdin")
        .write_all(&generated.stdout)
        .expect("write generated hook");
    let checked = child.wait_with_output().expect("wait for syntax checker");
    assert!(
        checked.status.success(),
        "generated {shell} hook failed syntax check: {}",
        String::from_utf8_lossy(&checked.stderr)
    );
}
