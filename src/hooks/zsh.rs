use super::common::{
    apply_template_replacements, render_posix_menu_eligible_case_pattern,
    render_posix_wrapper_declarations, render_zsh_completion_bindings,
    render_zsh_completion_functions, shell_words, DX_COMPLETE_MODES, DX_TOP_LEVEL_SUBCOMMANDS,
};

pub fn generate(command_not_found: bool, menu: bool) -> String {
    let mut script = String::from(
        r#"if [[ -z "${DX_SESSION:-}" ]]; then
  export DX_SESSION="$$"
fi

__dx_is_path_like() {
  local __dx_cmd="$1"
  [[ "$__dx_cmd" == */* || "$__dx_cmd" == .* || "$__dx_cmd" == ~* || "$__dx_cmd" == ...* ]]
}

__dx_push_pwd() {
  command -v dx >/dev/null 2>&1 || return 0
  dx stack push "$PWD" >/dev/null 2>&1 || true
}

__dx_complete_first() {
  local __dx_target=""
  local __dx_line
  while IFS= read -r __dx_line; do
    if [[ -n "$__dx_line" ]]; then
      __dx_target="$__dx_line"
      break
    fi
  done
  printf '%s' "$__dx_target"
}

__dx_nav_wrapper() {
  local __dx_mode="$1"
  local __dx_selector="${2:-}"
  (( $+commands[dx] )) || return 1

  __dx_push_pwd

  local __dx_target=""
  if [[ -n "$__dx_selector" ]]; then
    __dx_target="$(dx navigate "$__dx_mode" "$__dx_selector")"
  else
    __dx_target="$(dx navigate "$__dx_mode")"
  fi

  if [[ -z "$__dx_target" ]]; then
    return 1
  fi

  builtin cd "$__dx_target" || return $?
  __dx_push_pwd
  return 0
}

__dx_stack_wrapper() {
  local __dx_op="$1"
  local __dx_selector="${2:-}"
  (( $+commands[dx] )) || return 1

  local __dx_undo_or_redo
  if [[ "$__dx_op" == "back" ]]; then
    __dx_undo_or_redo="undo"
  else
    __dx_undo_or_redo="redo"
  fi

  local __dx_dest=""
  if [[ -n "$__dx_selector" ]]; then
    local __dx_target
    __dx_target="$(dx navigate "$__dx_op" "$__dx_selector")" || return 1
    [[ -n "$__dx_target" ]] || return 1
    __dx_dest="$(dx stack "$__dx_undo_or_redo" --target "$__dx_target")" || return 1
  else
    __dx_dest="$(dx stack "$__dx_undo_or_redo")" || return 1
  fi

  [[ -n "$__dx_dest" ]] || return 1
  builtin cd "$__dx_dest"
}

__dx_jump_mode() {
  local __dx_mode="$1"
  local __dx_query="${2:-}"
  (( $+commands[dx] )) || return 1

  local __dx_target=""
  if [[ -n "$__dx_query" ]]; then
    __dx_target="$(__dx_complete_first < <(dx complete "$__dx_mode" "$__dx_query" 2>/dev/null))"
  else
    __dx_target="$(__dx_complete_first < <(dx complete "$__dx_mode" 2>/dev/null))"
  fi

  if [[ -z "$__dx_target" ]]; then
    return 1
  fi

  builtin cd "$__dx_target" || return $?
  __dx_push_pwd
  return 0
}

cd() {
  local __dx_status=0

  if [[ $# -eq 0 ]]; then
    __dx_push_pwd
    builtin cd
    __dx_status=$?
    if [[ $__dx_status -eq 0 ]]; then
      __dx_push_pwd
    fi
    return $__dx_status
  fi

  if [[ "$1" == "-" && $# -eq 1 ]]; then
    __dx_push_pwd
    builtin cd -
    __dx_status=$?
    if [[ $__dx_status -eq 0 ]]; then
      __dx_push_pwd
    fi
    return $__dx_status
  fi

  local -a __dx_flags
  local __dx_path_arg=""
  local __dx_seen_path=0
  local __dx_arg

  for __dx_arg in "$@"; do
    if [[ $__dx_seen_path -eq 0 && "$__dx_arg" == -* && "$__dx_arg" != "-" ]]; then
      __dx_flags+=("$__dx_arg")
    elif [[ $__dx_seen_path -eq 0 ]]; then
      __dx_path_arg="$__dx_arg"
      __dx_seen_path=1
    fi
  done

  if [[ -z "$__dx_path_arg" ]]; then
    builtin cd "$@"
    return $?
  fi

  __dx_push_pwd
  local __dx_resolved=""
  if (( $+commands[dx] )); then
    __dx_resolved="$(dx resolve "$__dx_path_arg" 2>/dev/null)"
    if [[ $? -eq 0 && -n "$__dx_resolved" ]]; then
      builtin cd "${__dx_flags[@]}" "$__dx_resolved"
      __dx_status=$?
    else
      builtin cd "$@"
      __dx_status=$?
    fi
  else
    builtin cd "$@"
    __dx_status=$?
  fi

  if [[ $__dx_status -eq 0 ]]; then
    __dx_push_pwd
  fi

  return $__dx_status
}

__DX_POSIX_WRAPPER_DECLARATIONS__

__DX_ZSH_COMPLETION_FUNCTIONS__

_dx_complete_dx() {
  local cur="$words[CURRENT]"
  local sub="$words[2]"

  if (( CURRENT == 2 )); then
    compadd -- __DX_TOP_LEVEL_SUBCOMMANDS__
    return 0
  fi

  case "$sub" in
    resolve)
      _dx_complete_paths
      ;;
    complete)
      if (( CURRENT == 3 )); then
        compadd -- __DX_COMPLETE_MODES__
      fi
      ;;
    stack)
      _path_files -/
      ;;
    *)
      ;;
  esac
  return 0
}

__DX_ZSH_COMPLETION_BINDINGS__
"#,
    );

    if menu {
        script.push_str(
            r#"
__dx_menu_widget() {
  if [[ "${DX_MENU:-}" == "0" ]] || ! (( $+commands[dx] )); then
    zle expand-or-complete
    return
  fi

  local __dx_first="${BUFFER%% *}"
  case "$__dx_first" in
    __DX_ZSH_MENU_CASE__ ) ;;
    *)
      zle expand-or-complete
      return
      ;;
  esac

  local __dx_json
  __dx_json="$(dx menu --buffer "$BUFFER" --cursor $CURSOR --cwd "$PWD" --session "${DX_SESSION:-}" </dev/tty 2>/dev/tty)"
  local __dx_exit=$?

  # On cancel (noop) or error, leave the buffer unchanged and fall back
  # to native completion-equivalent behavior.
  if [[ $__dx_exit -ne 0 ]]; then
    zle expand-or-complete
    return
  fi

  local __dx_action_marker='"action":"'
  [[ "$__dx_json" == *$__dx_action_marker* ]] || { zle expand-or-complete; return }
  local __dx_action_rest="${__dx_json#*$__dx_action_marker}"
  local __dx_action="${__dx_action_rest%%\"*}"
  [[ "$__dx_action" == "replace" ]] || { zle expand-or-complete; return }

  local __dx_rs_marker='"replaceStart":'
  [[ "$__dx_json" == *$__dx_rs_marker* ]] || { zle expand-or-complete; return }
  local __dx_rs_rest="${__dx_json#*$__dx_rs_marker}"
  local __dx_rs="${__dx_rs_rest%%[^0-9]*}"
  [[ -n "$__dx_rs" ]] || { zle expand-or-complete; return }

  local __dx_re_marker='"replaceEnd":'
  [[ "$__dx_json" == *$__dx_re_marker* ]] || { zle expand-or-complete; return }
  local __dx_re_rest="${__dx_json#*$__dx_re_marker}"
  local __dx_re="${__dx_re_rest%%[^0-9]*}"
  [[ -n "$__dx_re" ]] || { zle expand-or-complete; return }

  (( __dx_re >= __dx_rs )) || { zle expand-or-complete; return }

  local __dx_value_marker='"value":"'
  [[ "$__dx_json" == *$__dx_value_marker* ]] || { zle expand-or-complete; return }
  local __dx_rest="${__dx_json#*$__dx_value_marker}"
  local __dx_value=""
  local __dx_i=1
  local __dx_len=${#__dx_rest}
  local __dx_escape=0
  local __dx_closed=0
  local __dx_ch

  while (( __dx_i <= __dx_len )); do
    __dx_ch="${__dx_rest[__dx_i]}"
    if (( __dx_escape )); then
      case "$__dx_ch" in
        '"'|'\\'|'/') __dx_value+="$__dx_ch" ;;
        *) zle expand-or-complete; return ;;
      esac
      __dx_escape=0
      (( __dx_i++ ))
      continue
    fi

    if [[ "$__dx_ch" == "\\" ]]; then
      __dx_escape=1
      (( __dx_i++ ))
      continue
    fi

    if [[ "$__dx_ch" == '"' ]]; then
      __dx_closed=1
      break
    fi

    __dx_value+="$__dx_ch"
    (( __dx_i++ ))
  done

  (( __dx_closed )) || { zle expand-or-complete; return }
  [[ -n "$__dx_value" ]] || { zle expand-or-complete; return }

  BUFFER="${BUFFER[1,$__dx_rs]}${__dx_value}${BUFFER[$((${__dx_re}+1)),-1]}"
  CURSOR=$(( __dx_rs + ${#__dx_value} ))
  zle reset-prompt
}

zle -N __dx_menu_widget
bindkey '^I' __dx_menu_widget
"#,
        );
    }

    if command_not_found {
        script.push_str(
            r#"
command_not_found_handler() {
  local __dx_cmd="$1"

  if [[ -n "${DX_RESOLVE_GUARD:-}" ]]; then
    print -u2 -- "zsh: command not found: $__dx_cmd"
    return 127
  fi

  if ! __dx_is_path_like "$__dx_cmd"; then
    print -u2 -- "zsh: command not found: $__dx_cmd"
    return 127
  fi

  if ! command -v dx >/dev/null 2>&1; then
    print -u2 -- "zsh: command not found: $__dx_cmd"
    return 127
  fi

  local __dx_resolved
  __dx_resolved="$(DX_RESOLVE_GUARD=1 dx resolve "$__dx_cmd" 2>/dev/null)"
  if [[ $? -ne 0 || -z "$__dx_resolved" ]]; then
    print -u2 -- "zsh: command not found: $__dx_cmd"
    return 127
  fi

  builtin cd "$__dx_resolved" || return $?
  __dx_push_pwd
  return 0
}
"#,
        );
    }

    apply_template_replacements(
        script,
        [
            (
                "__DX_TOP_LEVEL_SUBCOMMANDS__",
                shell_words(DX_TOP_LEVEL_SUBCOMMANDS),
            ),
            ("__DX_COMPLETE_MODES__", shell_words(DX_COMPLETE_MODES)),
            (
                "__DX_ZSH_COMPLETION_BINDINGS__",
                render_zsh_completion_bindings(),
            ),
            (
                "__DX_ZSH_COMPLETION_FUNCTIONS__",
                render_zsh_completion_functions(),
            ),
            (
                "__DX_POSIX_WRAPPER_DECLARATIONS__",
                render_posix_wrapper_declarations(),
            ),
            (
                "__DX_ZSH_MENU_CASE__",
                render_posix_menu_eligible_case_pattern(),
            ),
        ],
    )
}
