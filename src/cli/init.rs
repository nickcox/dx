use clap::ValueEnum;

use crate::hooks::{self, InitMenuMode, Shell, parse_menu_command_mappings, parse_pwsh_menu_key};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum InitShell {
    Bash,
    Zsh,
    Fish,
    Pwsh,
}

impl From<InitShell> for Shell {
    fn from(value: InitShell) -> Self {
        match value {
            InitShell::Bash => Self::Bash,
            InitShell::Zsh => Self::Zsh,
            InitShell::Fish => Self::Fish,
            InitShell::Pwsh => Self::Pwsh,
        }
    }
}

pub fn run_init(shell: InitShell, command_not_found: bool, menu: bool, native_menu: bool) -> i32 {
    let shell = shell.into();

    if native_menu && shell != Shell::Pwsh {
        eprintln!("dx init: --native-menu is only supported for pwsh");
        return 1;
    }

    let menu_mode = if menu {
        InitMenuMode::Tui
    } else if native_menu {
        InitMenuMode::NativePwsh
    } else {
        InitMenuMode::Disabled
    };

    let mappings = if menu_mode != InitMenuMode::Disabled {
        match std::env::var("DX_MENU_COMMAND_MAPPINGS") {
            Ok(raw) => match parse_menu_command_mappings(&raw) {
                Ok(parsed) => parsed,
                Err(err) => {
                    eprintln!("dx init: invalid DX_MENU_COMMAND_MAPPINGS: {err}");
                    return 1;
                }
            },
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };

    let pwsh_menu_key = if menu_mode == InitMenuMode::Tui && shell == Shell::Pwsh {
        match std::env::var("DX_PWSH_MENU_KEY") {
            Ok(raw) => match parse_pwsh_menu_key(&raw) {
                Ok(key) => key,
                Err(err) => {
                    eprintln!("dx init: invalid DX_PWSH_MENU_KEY: {err}");
                    return 1;
                }
            },
            Err(_) => "Tab".to_string(),
        }
    } else {
        "Tab".to_string()
    };

    let script = hooks::generate_with_menu_mode_and_pwsh_key(
        shell,
        command_not_found,
        menu_mode,
        &mappings,
        &pwsh_menu_key,
    );
    print!("{script}");
    0
}

#[cfg(test)]
mod tests {
    use crate::test_support::ScopedProcess;

    use super::{InitShell, run_init};

    #[test]
    fn init_rejects_invalid_menu_mappings_when_menu_enabled() {
        let mut process = ScopedProcess::new();
        process.set("DX_MENU_COMMAND_MAPPINGS", "ls=path,badentry");

        let code = run_init(InitShell::Bash, false, true, false);

        assert_eq!(code, 1);
    }

    #[test]
    fn init_ignores_invalid_menu_mappings_when_menu_disabled() {
        let mut process = ScopedProcess::new();
        process.set("DX_MENU_COMMAND_MAPPINGS", "ls=path,badentry");

        let code = run_init(InitShell::Bash, false, false, false);

        assert_eq!(code, 0);
    }
}
