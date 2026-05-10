use crate::hooks::{self, parse_menu_command_mappings, parse_pwsh_menu_key, Shell};

pub fn run_init(shell: &str, command_not_found: bool, menu: bool) -> i32 {
    let Some(shell) = Shell::parse(shell) else {
        eprintln!(
            "dx init: unsupported shell '{shell}' (supported: {})",
            Shell::supported_list()
        );
        return 1;
    };

    let mappings = if menu {
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

    let pwsh_menu_key = if menu && shell == Shell::Pwsh {
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

    let script = hooks::generate_with_mappings_and_pwsh_key(
        shell,
        command_not_found,
        menu,
        &mappings,
        &pwsh_menu_key,
    );
    print!("{script}");
    0
}

#[cfg(test)]
mod tests {
    use crate::test_support;

    use super::run_init;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        test_support::env_lock()
    }

    #[test]
    fn init_rejects_unknown_shell() {
        let code = run_init("unknown", false, false);
        assert_eq!(code, 1);
    }

    #[test]
    fn init_rejects_invalid_menu_mappings_when_menu_enabled() {
        let _guard = env_lock();
        unsafe { std::env::set_var("DX_MENU_COMMAND_MAPPINGS", "ls=path,badentry") };

        let code = run_init("bash", false, true);

        unsafe { std::env::remove_var("DX_MENU_COMMAND_MAPPINGS") };
        assert_eq!(code, 1);
    }

    #[test]
    fn init_ignores_invalid_menu_mappings_when_menu_disabled() {
        let _guard = env_lock();
        unsafe { std::env::set_var("DX_MENU_COMMAND_MAPPINGS", "ls=path,badentry") };

        let code = run_init("bash", false, false);

        unsafe { std::env::remove_var("DX_MENU_COMMAND_MAPPINGS") };
        assert_eq!(code, 0);
    }
}
