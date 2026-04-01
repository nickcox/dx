use crate::hooks::{self, Shell};

pub fn run_init(shell: &str, command_not_found: bool, menu: bool) -> i32 {
    let Some(shell) = Shell::parse(shell) else {
        eprintln!(
            "dx init: unsupported shell '{shell}' (supported: {})",
            Shell::supported_list()
        );
        return 1;
    };

    let script = hooks::generate(shell, command_not_found, menu);
    print!("{script}");
    0
}

#[cfg(test)]
mod tests {
    use super::run_init;

    #[test]
    fn init_rejects_unknown_shell() {
        let code = run_init("unknown", false, false);
        assert_eq!(code, 1);
    }
}
