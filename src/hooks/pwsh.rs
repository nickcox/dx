//! PowerShell hook generation, including the PSReadLine key binding and the
//! native `TabExpansion2` completer.

use super::common::{
    apply_template_replacements, menu_eligible_commands, pwsh_quoted_words,
    render_pwsh_completion_bindings, render_pwsh_exported_functions, render_pwsh_frecency_wrappers,
    render_pwsh_managed_aliases, render_pwsh_menu_mapping_list,
    render_pwsh_native_completion_bindings,
};
use thiserror::Error;

use super::{InitMenuMode, MenuCommandMapping};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PwshMenuKeyError {
    #[error("key contains unsupported character {0:?}")]
    UnsafeCharacter(char),
}

pub fn parse_pwsh_menu_key(raw: &str) -> Result<String, PwshMenuKeyError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok("Tab".to_string());
    }

    for ch in trimmed.chars() {
        if matches!(ch, '\n' | '\r' | '\'' | '"') {
            return Err(PwshMenuKeyError::UnsafeCharacter(ch));
        }
    }

    Ok(trimmed.to_string())
}

pub fn generate(
    command_not_found: bool,
    menu_mode: InitMenuMode,
    mappings: &[MenuCommandMapping],
    menu_key: &str,
    clap_completion: &str,
    frecency: bool,
) -> String {
    let menu = menu_mode == InitMenuMode::Tui;
    let native_menu = menu_mode == InitMenuMode::NativePwsh;
    let mut script = String::from(include_str!("templates/pwsh/base.ps1"));

    if native_menu {
        script.push_str(include_str!("templates/pwsh/native-menu.ps1"));
    }

    if menu {
        script.push_str(include_str!("templates/pwsh/menu.ps1"));
    }

    if command_not_found {
        script.push_str(include_str!("templates/pwsh/command-not-found.ps1"));
    }

    script.push_str(include_str!("templates/pwsh/on-remove-head.ps1"));

    if menu {
        script.push_str(include_str!("templates/pwsh/on-remove-menu.ps1"));
    }

    script.push_str(include_str!("templates/pwsh/on-remove-tail.ps1"));

    let navigation_completion_bindings = if native_menu {
        render_pwsh_native_completion_bindings()
    } else {
        render_pwsh_completion_bindings(frecency)
    };
    let completion_bindings = format!("{}\n\n{}", clap_completion, navigation_completion_bindings);

    let script = apply_template_replacements(
        script,
        [
            (
                "__DX_MENU_ELIGIBLE_COMMANDS__",
                pwsh_quoted_words(&menu_eligible_commands(frecency)),
            ),
            (
                "__DX_MENU_MAPPINGS__",
                render_pwsh_menu_mapping_list(mappings),
            ),
            ("__DX_PWSH_MENU_KEY__", menu_key.to_string()),
            ("__DX_PWSH_COMPLETION_BINDINGS__", completion_bindings),
            (
                "__DX_PWSH_FRECENCY_WRAPPERS__",
                render_pwsh_frecency_wrappers(frecency),
            ),
            (
                "__DX_PWSH_EXPORTED_FUNCTIONS__",
                render_pwsh_exported_functions(frecency),
            ),
            (
                "__DX_PWSH_MANAGED_ALIASES__",
                render_pwsh_managed_aliases(frecency),
            ),
        ],
    );

    hoist_using_statements(&script)
}

/// Moves every `using` statement to the top of the assembled script.
///
/// PowerShell requires `using` before any other statement, and clap emits one at
/// the top of a completion script that gets spliced into the middle of the hook.
fn hoist_using_statements(script: &str) -> String {
    let (using, body): (Vec<&str>, Vec<&str>) = script
        .lines()
        .partition(|line| line.starts_with("using ") || line.starts_with("using\t"));

    if using.is_empty() {
        return script.to_string();
    }

    let mut hoisted = String::with_capacity(script.len());
    for statement in using {
        hoisted.push_str(statement);
        hoisted.push('\n');
    }
    hoisted.push_str(&body.join("\n"));
    if script.ends_with('\n') {
        hoisted.push('\n');
    }
    hoisted
}
