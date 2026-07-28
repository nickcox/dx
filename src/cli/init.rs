//! `dx init` — prints the shell hook. Loads config leniently, because a broken
//! config file must not stop a shell profile from getting a usable hook.

use crate::config::AppConfig;
use crate::hooks::{
    self, DEFAULT_PWSH_MENU_KEY, HookOptions, InitMenuMode, MenuCommandMapping, Shell,
    parse_menu_command_mappings, parse_pwsh_menu_key,
};

use super::CliError;

pub fn run_init(
    shell: Shell,
    command_not_found: bool,
    menu: bool,
    native_menu: bool,
) -> Result<(), CliError> {
    if native_menu && shell != Shell::Pwsh {
        return Err(CliError::NativeMenuRequiresPwsh);
    }

    let menu_mode = if menu {
        InitMenuMode::Tui
    } else if native_menu {
        InitMenuMode::NativePwsh
    } else {
        InitMenuMode::Disabled
    };

    let config = load_config_leniently();

    let mappings = if menu_mode == InitMenuMode::Disabled {
        Vec::new()
    } else {
        menu_command_mappings(config.menu.command_mappings.as_deref())?
    };

    let pwsh_menu_key = if menu_mode == InitMenuMode::Tui && shell == Shell::Pwsh {
        pwsh_menu_key(config.menu.pwsh_key.as_deref())?
    } else {
        DEFAULT_PWSH_MENU_KEY.to_string()
    };

    print!(
        "{}",
        hooks::generate(
            shell,
            &HookOptions {
                command_not_found,
                menu_mode,
                mappings,
                pwsh_menu_key,
            },
            &super::completion_script(shell),
        )
    );
    Ok(())
}

/// `dx init` output is evaluated by shell profiles, so a broken config file must
/// not stop a usable hook being emitted. Every other subcommand still fails
/// loudly, since a wrong search root should not be silently ignored.
fn load_config_leniently() -> AppConfig {
    match AppConfig::load() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("dx init: warning: ignoring config file: {error}");
            AppConfig::default()
        }
    }
}

/// The environment wins, and a bad value there is an error the user can fix
/// immediately. A bad value in the config file is only a warning, for the same
/// reason `load_config_leniently` exists.
fn menu_command_mappings(from_file: Option<&str>) -> Result<Vec<MenuCommandMapping>, CliError> {
    if let Ok(raw) = std::env::var("DX_MENU_COMMAND_MAPPINGS") {
        return Ok(parse_menu_command_mappings(&raw)?);
    }

    let Some(raw) = from_file else {
        return Ok(Vec::new());
    };
    Ok(parse_menu_command_mappings(raw).unwrap_or_else(|error| {
        eprintln!("dx init: warning: ignoring menu.command_mappings: {error}");
        Vec::new()
    }))
}

fn pwsh_menu_key(from_file: Option<&str>) -> Result<String, CliError> {
    if let Ok(raw) = std::env::var("DX_PWSH_MENU_KEY") {
        return Ok(parse_pwsh_menu_key(&raw)?);
    }

    let Some(raw) = from_file else {
        return Ok(DEFAULT_PWSH_MENU_KEY.to_string());
    };
    Ok(parse_pwsh_menu_key(raw).unwrap_or_else(|error| {
        eprintln!("dx init: warning: ignoring menu.pwsh_key: {error}");
        DEFAULT_PWSH_MENU_KEY.to_string()
    }))
}

#[cfg(test)]
mod tests {
    use crate::test_support::ScopedProcess;

    use super::{CliError, Shell, run_init};

    #[test]
    fn init_rejects_invalid_menu_mappings_when_menu_enabled() {
        let mut process = ScopedProcess::new();
        process.set("DX_MENU_COMMAND_MAPPINGS", "ls=path,badentry");

        let error = run_init(Shell::Bash, false, true, false)
            .expect_err("invalid mappings must fail when the menu is enabled");

        assert!(matches!(error, CliError::MenuCommandMappings(_)));
    }

    #[test]
    fn init_ignores_invalid_menu_mappings_when_menu_disabled() {
        let mut process = ScopedProcess::new();
        process.set("DX_MENU_COMMAND_MAPPINGS", "ls=path,badentry");

        run_init(Shell::Bash, false, false, false)
            .expect("mappings are not parsed when the menu is disabled");
    }

    #[test]
    fn init_rejects_native_menu_for_non_pwsh_shells() {
        let error =
            run_init(Shell::Bash, false, false, true).expect_err("--native-menu is pwsh-only");

        assert!(matches!(error, CliError::NativeMenuRequiresPwsh));
    }
}
