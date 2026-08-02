//! Canonical cross-shell contract tables and shared renderer helpers live here.
//! Shell-specific parsing and control flow remain in
//! `bash.rs`, `zsh.rs`, `fish.rs`, and `pwsh.rs`.

use super::MenuCommandMapping;

/// The frecency commands, omitted from generated hooks when zoxide is absent.
/// They are the only ones that depend on it; `cdr` reads the session stack.
const FRECENCY_COMMANDS: &[&str] = &["cdf", "z"];

/// Menu-eligible commands, minus the frecency ones when they are not installed.
pub fn menu_eligible_commands(frecency: bool) -> Vec<&'static str> {
    MENU_ELIGIBLE_COMMANDS
        .iter()
        .copied()
        .filter(|command| frecency || !FRECENCY_COMMANDS.contains(command))
        .collect()
}

/// Completion routes, minus the frecency route when it is not installed.
pub fn completion_routes(frecency: bool) -> Vec<CompletionRoute> {
    COMPLETION_ROUTES
        .iter()
        .copied()
        .filter(|route| frecency || route.mode != "frecents")
        .collect()
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UniqueCompletionHandler {
    handler: &'static str,
    mode: &'static str,
    stack_direction: Option<&'static str>,
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

pub fn render_bash_completion_bindings(frecency: bool) -> String {
    let mut lines = Vec::new();
    for route in completion_routes(frecency) {
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

fn unique_completion_handlers(
    frecency: bool,
    pick_handler: impl Fn(&CompletionRoute) -> &'static str,
) -> Vec<UniqueCompletionHandler> {
    let mut seen_handlers: Vec<&str> = Vec::new();
    let mut unique = Vec::new();

    for route in completion_routes(frecency) {
        let handler = pick_handler(&route);
        if seen_handlers.contains(&handler) {
            continue;
        }
        seen_handlers.push(handler);
        unique.push(UniqueCompletionHandler {
            handler,
            mode: route.mode,
            stack_direction: route.stack_direction,
        });
    }

    unique
}

fn dx_complete_command(mode: &str, stack_direction: Option<&str>, current_word: &str) -> String {
    if let Some(direction) = stack_direction {
        format!("dx complete {mode} --direction {direction} \"{current_word}\" 2>/dev/null")
    } else {
        format!("dx complete {mode} \"{current_word}\" 2>/dev/null")
    }
}

fn fish_complete_rhs(mode: &str, stack_direction: Option<&str>) -> String {
    if let Some(direction) = stack_direction {
        format!("'(dx complete {mode} --direction {direction} (commandline -ct) 2>/dev/null)'")
    } else {
        format!("'(dx complete {mode} (commandline -ct) 2>/dev/null)'")
    }
}

fn pwsh_complete_invocation(mode: &str, stack_direction: Option<&str>) -> String {
    if let Some(direction) = stack_direction {
        format!(
            "__dx_emit_completion (__dx_complete_mode -Mode {mode} -Word $wordToComplete -ExtraArgs @('--direction', '{direction}'))"
        )
    } else {
        format!("__dx_emit_completion (__dx_complete_mode -Mode {mode} -Word $wordToComplete)")
    }
}

pub fn render_zsh_completion_bindings(frecency: bool) -> String {
    let mut lines = Vec::new();
    for route in completion_routes(frecency) {
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

pub fn render_fish_completion_bindings(frecency: bool) -> String {
    let mut lines = Vec::new();
    for route in completion_routes(frecency) {
        for command in route.commands {
            let rhs = fish_complete_rhs(route.mode, route.stack_direction);
            lines.push(format!("complete -c {command} -a {rhs}"));
        }
    }
    lines.join("\n")
}

/// Binds the menu completer to each command the hook actually installs, so a
/// command dx chose not to define keeps whatever completion it already had.
/// The fish `cdf`/`z` wrappers, or nothing when zoxide is absent.
/// The module's exported functions. `Set-FrecentLocation` is only defined when
/// zoxide is present, and exporting a function that does not exist is an error.
/// Aliases the module takes over and hands back on unload.
///
/// `cdf` and `z` belong here only when dx actually defines them. Listing an
/// alias dx never installs would have the module remove somebody else's on the
/// way out.
pub fn render_pwsh_managed_aliases(frecency: bool) -> String {
    let mut names = vec!["cd", "up", "..", "back", "forward", "cd-", "cd+"];
    if frecency {
        names.push("cdf");
    }
    names.push("cdr");
    if frecency {
        names.push("z");
    }
    names
        .into_iter()
        .map(|name| format!("'{name}'"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn render_pwsh_exported_functions(frecency: bool) -> String {
    let mut names = vec![
        "Set-DxLocation",
        "Step-Up",
        "Undo-Location",
        "Redo-Location",
    ];
    if frecency {
        names.push("Set-FrecentLocation");
    }
    names.push("Set-RecentLocation");
    names.join(", ")
}

pub fn render_fish_frecency_wrappers(frecency: bool) -> String {
    if !frecency {
        return String::new();
    }

    r#"function cdf
  __dx_jump_mode frecents "$argv[1]"
end

function z
  cdf $argv
end

"#
    .to_string()
}

/// The PowerShell frecency command and its aliases, or nothing without zoxide.
pub fn render_pwsh_frecency_wrappers(frecency: bool) -> String {
    if !frecency {
        return String::new();
    }

    r#"function Set-FrecentLocation {
    param([string]$Query)
    $target = __dx_complete_first (__dx_complete_mode -Mode frecents -Word $Query)
    if ($target) {
        __dx_push_pwd
        __dx_set_location_native @($target)
        if ($?) { __dx_push_pwd }
    }
}

__dx_set_alias cdf Set-FrecentLocation
__dx_set_alias z Set-FrecentLocation"#
        .to_string()
}

pub fn render_bash_menu_complete_bindings(frecency: bool) -> String {
    menu_eligible_commands(frecency)
        .into_iter()
        .map(|command| {
            if command == "cd" {
                "complete -o default -F _dx_menu_wrapper cd".to_string()
            } else {
                format!("complete -F _dx_menu_wrapper {command}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_bash_menu_fallback_case(frecency: bool) -> String {
    completion_routes(frecency)
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

pub fn render_bash_menu_mapping_case(mappings: &[MenuCommandMapping]) -> String {
    mappings
        .iter()
        .map(|mapping| {
            format!(
                "    {}) __dx_menu_mode=\"{}\" ;;",
                bash_case_pattern(&[mapping.command()]),
                mapping.mode().as_cli_arg()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_zsh_menu_mapping_case(mappings: &[MenuCommandMapping]) -> String {
    mappings
        .iter()
        .map(|mapping| {
            let command = quote_if_special(mapping.command());
            format!(
                "    {command}) __dx_menu_mode=\"{}\" ;;",
                mapping.mode().as_cli_arg()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_fish_menu_mapping_cases(mappings: &[MenuCommandMapping]) -> String {
    mappings
        .iter()
        .map(|mapping| {
            let command = quote_if_special(mapping.command());
            format!(
                "    case {command}\n      set -l dx_menu_mode {}",
                mapping.mode().as_cli_arg()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_pwsh_menu_mapping_list(mappings: &[MenuCommandMapping]) -> String {
    mappings
        .iter()
        .map(|mapping| format!("'{}={}'", mapping.command(), mapping.mode().as_cli_arg()))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn render_posix_wrapper_declarations(frecency: bool) -> String {
    let mut out = r#"up() {
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

cdr() {
  __dx_jump_mode recents "${1:-}"
}"#
    .to_string();

    // `cdr` reads the session stack, so it works regardless; only these two go
    // through zoxide, and a wrapper that can only ever find nothing is worse
    // than no command at all.
    if frecency {
        out.push_str(
            r#"

cdf() {
  __dx_jump_mode frecents "${1:-}"
}

z() {
  cdf "$@"
}"#,
        );
    }

    out
}

pub fn render_bash_completion_functions(frecency: bool) -> String {
    let mut out = Vec::new();

    for handler in unique_completion_handlers(frecency, |route| route.bash_handler) {
        let dx_complete_call = dx_complete_command(handler.mode, handler.stack_direction, "$cur");

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
            handler.handler, dx_complete_call
        ));
    }

    out.join("\n\n")
}

pub fn render_zsh_completion_functions(frecency: bool) -> String {
    let mut out = Vec::new();

    for handler in unique_completion_handlers(frecency, |route| route.zsh_handler) {
        let dx_complete_call = dx_complete_command(handler.mode, handler.stack_direction, "$cur");

        out.push(format!(
            r#"{}() {{
  (( $+commands[dx] )) || return 1
  local cur="$words[CURRENT]"
  local -a candidates
  candidates=("${{(@f)$({})}}")
  (( ${{#candidates}} )) && compadd -a candidates
}}"#,
            handler.handler, dx_complete_call
        ));
    }

    out.join("\n\n")
}

pub fn render_posix_menu_eligible_case_pattern(frecency: bool) -> String {
    bash_case_pattern(&menu_eligible_commands(frecency))
}

fn render_pwsh_route_binding(route: &CompletionRoute) -> String {
    let command_names = route.commands.join(",");
    let invocation = pwsh_complete_invocation(route.mode, route.stack_direction);
    format!(
        "Register-ArgumentCompleter -CommandName {command_names} -ScriptBlock {{\n    param($wordToComplete, $commandAst, $cursorPosition)\n    {invocation}\n}}"
    )
}

pub fn render_pwsh_navigation_completion_bindings(frecency: bool) -> String {
    completion_routes(frecency)
        .iter()
        .filter(|route| route.commands != ["cd"])
        .map(render_pwsh_route_binding)
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub fn render_pwsh_completion_bindings(frecency: bool) -> String {
    let cd = r#"Register-ArgumentCompleter -CommandName cd,Set-Location -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    __dx_emit_completion (__dx_complete_mode -Mode paths -Word $wordToComplete)
}"#;

    [
        cd.to_string(),
        render_pwsh_navigation_completion_bindings(frecency),
    ]
    .join("\n\n")
}

pub fn render_pwsh_native_completion_bindings() -> String {
    r#"Register-ArgumentCompleter -CommandName Set-DxLocation,cd,Set-Location -ParameterName Path -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    __dx_emit_native_completion (__dx_complete_json -Mode paths -Word $wordToComplete) -Directory
}

Register-ArgumentCompleter -CommandName Step-Up,up,'..' -ParameterName Selector -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    __dx_emit_native_completion (__dx_complete_json -Mode ancestors -Word $wordToComplete)
}

Register-ArgumentCompleter -CommandName Undo-Location,back,'cd-' -ParameterName Selector -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    __dx_emit_native_completion (__dx_complete_json -Mode stack -Word $wordToComplete -ExtraArgs @('--direction', 'back'))
}

Register-ArgumentCompleter -CommandName Redo-Location,forward,'cd+' -ParameterName Selector -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    __dx_emit_native_completion (__dx_complete_json -Mode stack -Word $wordToComplete -ExtraArgs @('--direction', 'forward'))
}

Register-ArgumentCompleter -CommandName Set-FrecentLocation,cdf,z -ParameterName Query -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    __dx_emit_native_completion (__dx_complete_json -Mode frecents -Word $wordToComplete)
}

Register-ArgumentCompleter -CommandName Set-RecentLocation,cdr -ParameterName Query -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    __dx_emit_native_completion (__dx_complete_json -Mode recents -Word $wordToComplete)
}"#
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        COMPLETION_ROUTES, dx_complete_command, fish_complete_rhs, pwsh_complete_invocation,
        unique_completion_handlers,
    };

    #[test]
    fn unique_completion_handlers_dedupes_by_selected_handler() {
        let bash_handlers = unique_completion_handlers(true, |route| route.bash_handler);
        let zsh_handlers = unique_completion_handlers(true, |route| route.zsh_handler);
        let bash_handler_names = bash_handlers
            .iter()
            .map(|handler| handler.handler)
            .collect::<Vec<_>>();

        assert_eq!(bash_handlers.len(), 6);
        assert_eq!(zsh_handlers.len(), 6);
        assert_eq!(bash_handlers[0].handler, "_dx_complete_paths");
        assert_eq!(bash_handlers[5].handler, "_dx_complete_stack_forward");
        assert!(bash_handler_names.contains(&"_dx_complete_stack_back"));
        assert_eq!(COMPLETION_ROUTES.len(), 6);
    }

    #[test]
    fn shared_completion_command_assembly_preserves_stack_direction_forms() {
        assert_eq!(
            dx_complete_command("paths", None, "$cur"),
            "dx complete paths \"$cur\" 2>/dev/null"
        );
        assert_eq!(
            dx_complete_command("stack", Some("back"), "$cur"),
            "dx complete stack --direction back \"$cur\" 2>/dev/null"
        );
        assert_eq!(
            fish_complete_rhs("stack", Some("forward")),
            "'(dx complete stack --direction forward (commandline -ct) 2>/dev/null)'"
        );
        assert_eq!(
            pwsh_complete_invocation("stack", Some("back")),
            "__dx_emit_completion (__dx_complete_mode -Mode stack -Word $wordToComplete -ExtraArgs @('--direction', 'back'))"
        );
    }
}
