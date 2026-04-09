use super::common::{
    apply_template_replacements, render_bash_completion_bindings, render_bash_completion_functions,
    render_bash_menu_fallback_case, render_posix_wrapper_declarations, shell_words,
    DX_COMPLETE_MODES, DX_TOP_LEVEL_SUBCOMMANDS,
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

__dx_cd_native() {
  builtin cd "$@"
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
  command -v dx >/dev/null 2>&1 || return 1

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

  __dx_cd_native "$__dx_target" || return $?
  __dx_push_pwd
  return 0
}

__dx_stack_wrapper() {
  local __dx_op="$1"
  local __dx_selector="${2:-}"
  command -v dx >/dev/null 2>&1 || return 1

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
  __dx_cd_native "$__dx_dest"
}

__dx_jump_mode() {
  local __dx_mode="$1"
  local __dx_query="${2:-}"
  command -v dx >/dev/null 2>&1 || return 1

  local __dx_target=""
  if [[ -n "$__dx_query" ]]; then
    __dx_target="$(__dx_complete_first < <(dx complete "$__dx_mode" "$__dx_query" 2>/dev/null))"
  else
    __dx_target="$(__dx_complete_first < <(dx complete "$__dx_mode" 2>/dev/null))"
  fi

  if [[ -z "$__dx_target" ]]; then
    return 1
  fi

  __dx_cd_native "$__dx_target" || return $?
  __dx_push_pwd
  return 0
}

cd() {
  local __dx_status=0

  if [[ $# -eq 0 ]]; then
    __dx_push_pwd
    __dx_cd_native
    __dx_status=$?
    if [[ $__dx_status -eq 0 ]]; then
      __dx_push_pwd
    fi
    return $__dx_status
  fi

  if [[ "$1" == "-" && $# -eq 1 ]]; then
    __dx_push_pwd
    __dx_cd_native -
    __dx_status=$?
    if [[ $__dx_status -eq 0 ]]; then
      __dx_push_pwd
    fi
    return $__dx_status
  fi

  local __dx_flags=()
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
    __dx_cd_native "$@"
    return $?
  fi

  __dx_push_pwd
  local __dx_resolved=""
  if command -v dx >/dev/null 2>&1; then
    __dx_resolved="$(dx resolve "$__dx_path_arg" 2>/dev/null)"
    if [[ $? -eq 0 && -n "$__dx_resolved" ]]; then
      __dx_cd_native "${__dx_flags[@]}" "$__dx_resolved"
      __dx_status=$?
    else
      __dx_cd_native "$@"
      __dx_status=$?
    fi
  else
    __dx_cd_native "$@"
    __dx_status=$?
  fi

  if [[ $__dx_status -eq 0 ]]; then
    __dx_push_pwd
  fi

  return $__dx_status
}

__DX_POSIX_WRAPPER_DECLARATIONS__

__DX_BASH_COMPLETION_FUNCTIONS__

_dx_complete_dx() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local sub="${COMP_WORDS[1]:-}"
  COMPREPLY=()

  if [[ ${COMP_CWORD} -eq 1 ]]; then
    COMPREPLY=( $(compgen -W "__DX_TOP_LEVEL_SUBCOMMANDS__" -- "$cur") )
    return 0
  fi

  case "$sub" in
    resolve)
      _dx_complete_paths
      ;;
    complete)
      if [[ ${COMP_CWORD} -eq 2 ]]; then
        COMPREPLY=( $(compgen -W "__DX_COMPLETE_MODES__" -- "$cur") )
      fi
      ;;
    stack)
      return 1
      ;;
    *)
      ;;
  esac
  return 0
}

__DX_BASH_COMPLETION_BINDINGS__
"#,
    );

    if menu {
        script.push_str(
            r#"
__dx_try_menu() {
  [[ "${DX_MENU:-}" == "0" ]] && return 1
  command -v dx >/dev/null 2>&1 || return 1

  __dx_json_extract_string() {
    local __dx_key="$1"
    local __dx_json_input="$2"
    local __dx_marker="\"$__dx_key\":\""
    [[ "$__dx_json_input" == *"$__dx_marker"* ]] || return 1

    local __dx_rest="${__dx_json_input#*"$__dx_marker"}"
    local __dx_i=0
    local __dx_len=${#__dx_rest}
    local __dx_escape=0
    local __dx_ch
    local __dx_out=""

    while (( __dx_i < __dx_len )); do
      __dx_ch="${__dx_rest:__dx_i:1}"
      if (( __dx_escape )); then
        case "$__dx_ch" in
          '"'|'\\'|'/') __dx_out+="$__dx_ch" ;;
          *) return 1 ;;
        esac
        __dx_escape=0
        ((__dx_i++))
        continue
      fi

      if [[ "$__dx_ch" == "\\" ]]; then
        __dx_escape=1
        ((__dx_i++))
        continue
      fi

      if [[ "$__dx_ch" == '"' ]]; then
        printf '%s' "$__dx_out"
        return 0
      fi

      __dx_out+="$__dx_ch"
      ((__dx_i++))
    done

    return 1
  }

  __dx_json_extract_uint() {
    local __dx_key="$1"
    local __dx_json_input="$2"
    local __dx_marker="\"$__dx_key\":"
    [[ "$__dx_json_input" == *"$__dx_marker"* ]] || return 1

    local __dx_rest="${__dx_json_input#*"$__dx_marker"}"
    local __dx_num="${__dx_rest%%[^0-9]*}"
    [[ -n "$__dx_num" ]] || return 1
    printf '%s' "$__dx_num"
  }

  local __dx_json
  __dx_json="$(dx menu --buffer "$COMP_LINE" --cursor "$COMP_POINT" --cwd "$PWD" --session "${DX_SESSION:-}" </dev/tty 2>/dev/tty)" || return 1

  local __dx_action
  __dx_action="$(__dx_json_extract_string action "$__dx_json")" || return 1
  [[ "$__dx_action" == "replace" ]] || return 1

  local __dx_rs __dx_re
  __dx_rs="$(__dx_json_extract_uint replaceStart "$__dx_json")" || return 1
  __dx_re="$(__dx_json_extract_uint replaceEnd "$__dx_json")" || return 1
  (( __dx_re >= __dx_rs )) || return 1

  local __dx_value
  __dx_value="$(__dx_json_extract_string value "$__dx_json")" || return 1
  [[ -n "$__dx_value" ]] || return 1

  COMPREPLY=("$__dx_value")
  return 0
}

_dx_menu_wrapper() {
  if __dx_try_menu; then
    return 0
  fi
  local __dx_cmd="${COMP_LINE%% *}"
  case "$__dx_cmd" in
__DX_BASH_MENU_FALLBACK_CASE__
  esac
}

complete -o default -F _dx_menu_wrapper cd
complete -F _dx_menu_wrapper up
complete -F _dx_menu_wrapper cdf
complete -F _dx_menu_wrapper z
complete -F _dx_menu_wrapper cdr
complete -F _dx_menu_wrapper back
complete -F _dx_menu_wrapper cd-
complete -F _dx_menu_wrapper forward
complete -F _dx_menu_wrapper cd+
"#,
        );
    }

    if command_not_found {
        script.push_str(
            r#"
command_not_found_handle() {
  local __dx_cmd="$1"

  if [[ -n "${DX_RESOLVE_GUARD:-}" ]]; then
    printf "%s: command not found\n" "$__dx_cmd" >&2
    return 127
  fi

  if ! __dx_is_path_like "$__dx_cmd"; then
    printf "%s: command not found\n" "$__dx_cmd" >&2
    return 127
  fi

  if ! command -v dx >/dev/null 2>&1; then
    printf "%s: command not found\n" "$__dx_cmd" >&2
    return 127
  fi

  local __dx_resolved
  __dx_resolved="$(DX_RESOLVE_GUARD=1 dx resolve "$__dx_cmd" 2>/dev/null)"
  if [[ $? -ne 0 || -z "$__dx_resolved" ]]; then
    printf "%s: command not found\n" "$__dx_cmd" >&2
    return 127
  fi

  __dx_cd_native "$__dx_resolved" || return $?
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
                "__DX_BASH_COMPLETION_BINDINGS__",
                render_bash_completion_bindings(),
            ),
            (
                "__DX_BASH_COMPLETION_FUNCTIONS__",
                render_bash_completion_functions(),
            ),
            (
                "__DX_POSIX_WRAPPER_DECLARATIONS__",
                render_posix_wrapper_declarations(),
            ),
            (
                "__DX_BASH_MENU_FALLBACK_CASE__",
                render_bash_menu_fallback_case(),
            ),
        ],
    )
}
