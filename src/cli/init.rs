use crate::hooks::{
    self, DEFAULT_PWSH_MENU_KEY, HookOptions, InitMenuMode, Shell, parse_menu_command_mappings,
    parse_pwsh_menu_key,
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

    let mappings = if menu_mode == InitMenuMode::Disabled {
        Vec::new()
    } else {
        match std::env::var("DX_MENU_COMMAND_MAPPINGS") {
            Ok(raw) => parse_menu_command_mappings(&raw)?,
            Err(_) => Vec::new(),
        }
    };

    let pwsh_menu_key = if menu_mode == InitMenuMode::Tui && shell == Shell::Pwsh {
        match std::env::var("DX_PWSH_MENU_KEY") {
            Ok(raw) => parse_pwsh_menu_key(&raw)?,
            Err(_) => DEFAULT_PWSH_MENU_KEY.to_string(),
        }
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
        )
    );
    Ok(())
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
