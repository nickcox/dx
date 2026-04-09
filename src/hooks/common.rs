//! Canonical cross-shell contract tables and shared renderer helpers live here.
//! Shell-specific parsing and control flow remain in
//! `bash.rs`, `zsh.rs`, `fish.rs`, and `pwsh.rs`.

pub const DX_TOP_LEVEL_SUBCOMMANDS: &[&str] = &[
    "resolve",
    "complete",
    "init",
    "bookmarks",
    "stack",
    "navigate",
    "menu",
];

pub const DX_COMPLETE_MODES: &[&str] = &["paths", "ancestors", "frecents", "recents", "stack"];

pub const MENU_ELIGIBLE_COMMANDS: &[&str] = &[
    "cd", "up", "cdf", "z", "cdr", "back", "forward", "cd-", "cd+",
];

#[derive(Debug, Clone, Copy)]
pub struct CompletionRoute {
    pub commands: &'static [&'static str],
    pub mode: &'static str,
    pub stack_direction: Option<&'static str>,
    pub bash_handler: &'static str,
    pub zsh_handler: &'static str,
}

pub const COMPLETION_ROUTES: &[CompletionRoute] = &[
    CompletionRoute {
        commands: &["cd"],
        mode: "paths",
        stack_direction: None,
        bash_handler: "_dx_complete_paths",
        zsh_handler: "_dx_complete_paths",
    },
    CompletionRoute {
        commands: &["up"],
        mode: "ancestors",
        stack_direction: None,
        bash_handler: "_dx_complete_ancestors",
        zsh_handler: "_dx_complete_ancestors",
    },
    CompletionRoute {
        commands: &["cdf", "z"],
        mode: "frecents",
        stack_direction: None,
        bash_handler: "_dx_complete_frecents",
        zsh_handler: "_dx_complete_frecents",
    },
    CompletionRoute {
        commands: &["cdr"],
        mode: "recents",
        stack_direction: None,
        bash_handler: "_dx_complete_recents",
        zsh_handler: "_dx_complete_recents",
    },
    CompletionRoute {
        commands: &["back", "cd-"],
        mode: "stack",
        stack_direction: Some("back"),
        bash_handler: "_dx_complete_stack_back",
        zsh_handler: "_dx_complete_stack_back",
    },
    CompletionRoute {
        commands: &["forward", "cd+"],
        mode: "stack",
        stack_direction: Some("forward"),
        bash_handler: "_dx_complete_stack_forward",
        zsh_handler: "_dx_complete_stack_forward",
    },
];

pub fn shell_words(words: &[&str]) -> String {
    words.join(" ")
}

pub fn bash_case_pattern(commands: &[&str]) -> String {
    commands.join("|")
}

fn quote_if_special(command: &str) -> String {
    if command.contains('-') || command.contains('+') {
        format!("'{command}'")
    } else {
        command.to_string()
    }
}

pub fn fish_case_words(commands: &[&str]) -> String {
    commands
        .iter()
        .map(|command| quote_if_special(command))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn pwsh_quoted_words(words: &[&str]) -> String {
    words
        .iter()
        .map(|word| format!("'{word}'"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn apply_template_replacements<'a, I>(mut template: String, replacements: I) -> String
where
    I: IntoIterator<Item = (&'a str, String)>,
{
    for (token, value) in replacements {
        template = template.replace(token, &value);
    }
    template
}

pub fn render_bash_completion_bindings() -> String {
    let mut lines = vec!["complete -o default -F _dx_complete_dx dx".to_string()];
    for route in COMPLETION_ROUTES {
        for command in route.commands {
            if *command == "cd" {
                lines.push(format!(
                    "complete -o default -F {} {command}",
                    route.bash_handler
                ));
            } else {
                lines.push(format!("complete -F {} {command}", route.bash_handler));
            }
        }
    }
    lines.join("\n")
}

pub fn render_zsh_completion_bindings() -> String {
    let mut lines = vec!["compdef _dx_complete_dx dx".to_string()];
    for route in COMPLETION_ROUTES {
        let commands = route
            .commands
            .iter()
            .map(|command| quote_if_special(command))
            .collect::<Vec<_>>()
            .join(" ");
        lines.push(format!("compdef {} {commands}", route.zsh_handler));
    }
    lines.join("\n")
}

pub fn render_fish_completion_bindings() -> String {
    let mut lines = Vec::new();
    for route in COMPLETION_ROUTES {
        for command in route.commands {
            let rhs = if let Some(direction) = route.stack_direction {
                format!(
                    "'(dx complete {} --direction {} (commandline -ct) 2>/dev/null)'",
                    route.mode, direction
                )
            } else {
                format!(
                    "'(dx complete {} (commandline -ct) 2>/dev/null)'",
                    route.mode
                )
            };
            lines.push(format!("complete -c {command} -a {rhs}"));
        }
    }
    lines.join("\n")
}

pub fn render_fish_dx_root_completion_bindings() -> String {
    format!(
        "complete -c dx -n '__fish_use_subcommand' -a '{top}'\ncomplete -c dx -n '__fish_seen_subcommand_from complete; and not __fish_seen_subcommand_from {modes}' -a '{modes}'\ncomplete -c dx -n '__fish_seen_subcommand_from resolve' -a '(dx complete paths (commandline -ct) 2>/dev/null)'",
        top = shell_words(DX_TOP_LEVEL_SUBCOMMANDS),
        modes = shell_words(DX_COMPLETE_MODES)
    )
}

pub fn render_bash_menu_fallback_case() -> String {
    COMPLETION_ROUTES
        .iter()
        .map(|route| {
            format!(
                "    {}) {} ;;",
                bash_case_pattern(route.commands),
                route.bash_handler
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_posix_wrapper_declarations() -> String {
    r#"up() {
  __dx_nav_wrapper up "${1:-}"
}

back() {
  __dx_stack_wrapper back "${1:-}"
}

forward() {
  __dx_stack_wrapper forward "${1:-}"
}

cd-() {
  back "$@"
}

cd+() {
  forward "$@"
}

cdf() {
  __dx_jump_mode frecents "${1:-}"
}

z() {
  cdf "$@"
}

cdr() {
  __dx_jump_mode recents "${1:-}"
}"#
    .to_string()
}

pub fn render_bash_completion_functions() -> String {
    let mut seen_handlers: Vec<&str> = Vec::new();
    let mut out = Vec::new();

    for route in COMPLETION_ROUTES {
        if seen_handlers.contains(&route.bash_handler) {
            continue;
        }
        seen_handlers.push(route.bash_handler);

        let dx_complete_call = if let Some(direction) = route.stack_direction {
            format!(
                "dx complete {} --direction {} \"$cur\" 2>/dev/null",
                route.mode, direction
            )
        } else {
            format!("dx complete {} \"$cur\" 2>/dev/null", route.mode)
        };

        out.push(format!(
            r#"{}() {{
  local cur="${{COMP_WORDS[COMP_CWORD]}}"
  COMPREPLY=()
  command -v dx >/dev/null 2>&1 || return 1
  local line
  while IFS= read -r line; do
    [[ -n "$line" ]] && COMPREPLY+=("$line")
  done < <({})
}}"#,
            route.bash_handler, dx_complete_call
        ));
    }

    out.join("\n\n")
}

pub fn render_zsh_completion_functions() -> String {
    let mut seen_handlers: Vec<&str> = Vec::new();
    let mut out = Vec::new();

    for route in COMPLETION_ROUTES {
        if seen_handlers.contains(&route.zsh_handler) {
            continue;
        }
        seen_handlers.push(route.zsh_handler);

        let dx_complete_call = if let Some(direction) = route.stack_direction {
            format!(
                "dx complete {} --direction {} \"$cur\" 2>/dev/null",
                route.mode, direction
            )
        } else {
            format!("dx complete {} \"$cur\" 2>/dev/null", route.mode)
        };

        out.push(format!(
            r#"{}() {{
  (( $+commands[dx] )) || return 1
  local cur="$words[CURRENT]"
  local -a candidates
  candidates=("${{(@f)$({})}}")
  (( ${{#candidates}} )) && compadd -a candidates
}}"#,
            route.zsh_handler, dx_complete_call
        ));
    }

    out.join("\n\n")
}

pub fn render_posix_menu_eligible_case_pattern() -> String {
    bash_case_pattern(MENU_ELIGIBLE_COMMANDS)
}

fn render_pwsh_route_binding(route: &CompletionRoute) -> String {
    let command_names = route.commands.join(",");
    if let Some(direction) = route.stack_direction {
        format!(
            "Register-ArgumentCompleter -CommandName {command_names} -ScriptBlock {{\n    param($wordToComplete, $commandAst, $cursorPosition)\n    __dx_emit_completion (__dx_complete_mode -Mode {} -Word $wordToComplete -ExtraArgs @('--direction', '{}'))\n}}",
            route.mode, direction
        )
    } else {
        format!(
            "Register-ArgumentCompleter -CommandName {command_names} -ScriptBlock {{\n    param($wordToComplete, $commandAst, $cursorPosition)\n    __dx_emit_completion (__dx_complete_mode -Mode {} -Word $wordToComplete)\n}}",
            route.mode
        )
    }
}

pub fn render_pwsh_navigation_completion_bindings() -> String {
    COMPLETION_ROUTES
        .iter()
        .filter(|route| route.commands != ["cd"])
        .map(render_pwsh_route_binding)
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub fn render_pwsh_completion_bindings() -> String {
    let top_level = pwsh_quoted_words(DX_TOP_LEVEL_SUBCOMMANDS);
    let modes = pwsh_quoted_words(DX_COMPLETE_MODES);

    let dx = format!(
        r#"Register-ArgumentCompleter -CommandName dx -ScriptBlock {{
    param($wordToComplete, $commandAst, $cursorPosition)

    $elements = @($commandAst.CommandElements | ForEach-Object {{ $_.Extent.Text }})
    if ($elements.Count -le 1) {{
        __dx_emit_completion @({top_level})
        return
    }}

    $sub = $elements[1]
    switch ($sub) {{
        'resolve' {{
            __dx_emit_completion (__dx_complete_mode -Mode paths -Word $wordToComplete)
            break
        }}
        'complete' {{
            if ($elements.Count -le 3) {{
                __dx_emit_completion @({modes})
            }}
            break
        }}
        default {{
            break
        }}
    }}
}}"#
    );

    let cd = r#"Register-ArgumentCompleter -CommandName cd,Set-Location -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    __dx_emit_completion (__dx_complete_mode -Mode paths -Word $wordToComplete)
}"#;

    [
        dx,
        cd.to_string(),
        render_pwsh_navigation_completion_bindings(),
    ]
    .join("\n\n")
}
