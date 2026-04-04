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

up() {
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
}

_dx_complete_paths() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  COMPREPLY=()
  command -v dx >/dev/null 2>&1 || return 1
  local line
  while IFS= read -r line; do
    [[ -n "$line" ]] && COMPREPLY+=("$line")
  done < <(dx complete paths "$cur" 2>/dev/null)
}

_dx_complete_ancestors() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  COMPREPLY=()
  command -v dx >/dev/null 2>&1 || return 1
  local line
  while IFS= read -r line; do
    [[ -n "$line" ]] && COMPREPLY+=("$line")
  done < <(dx complete ancestors "$cur" 2>/dev/null)
}

_dx_complete_frecents() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  COMPREPLY=()
  command -v dx >/dev/null 2>&1 || return 1
  local line
  while IFS= read -r line; do
    [[ -n "$line" ]] && COMPREPLY+=("$line")
  done < <(dx complete frecents "$cur" 2>/dev/null)
}

_dx_complete_recents() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  COMPREPLY=()
  command -v dx >/dev/null 2>&1 || return 1
  local line
  while IFS= read -r line; do
    [[ -n "$line" ]] && COMPREPLY+=("$line")
  done < <(dx complete recents "$cur" 2>/dev/null)
}

_dx_complete_stack_back() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  COMPREPLY=()
  command -v dx >/dev/null 2>&1 || return 1
  local line
  while IFS= read -r line; do
    [[ -n "$line" ]] && COMPREPLY+=("$line")
  done < <(dx complete stack --direction back "$cur" 2>/dev/null)
}

_dx_complete_stack_forward() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  COMPREPLY=()
  command -v dx >/dev/null 2>&1 || return 1
  local line
  while IFS= read -r line; do
    [[ -n "$line" ]] && COMPREPLY+=("$line")
  done < <(dx complete stack --direction forward "$cur" 2>/dev/null)
}

_dx_complete_dx() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local sub="${COMP_WORDS[1]:-}"
  COMPREPLY=()

  if [[ ${COMP_CWORD} -eq 1 ]]; then
    COMPREPLY=( $(compgen -W "resolve complete init bookmarks stack navigate menu" -- "$cur") )
    return 0
  fi

  case "$sub" in
    resolve)
      _dx_complete_paths
      ;;
    complete)
      if [[ ${COMP_CWORD} -eq 2 ]]; then
        COMPREPLY=( $(compgen -W "paths ancestors frecents recents stack" -- "$cur") )
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

complete -o default -F _dx_complete_dx dx
complete -o default -F _dx_complete_paths cd
complete -F _dx_complete_ancestors up
complete -F _dx_complete_frecents cdf
complete -F _dx_complete_frecents z
complete -F _dx_complete_recents cdr
complete -F _dx_complete_stack_back back
complete -F _dx_complete_stack_back cd-
complete -F _dx_complete_stack_forward forward
complete -F _dx_complete_stack_forward cd+
"#,
    );

    if menu {
        script.push_str(
            r#"
__dx_try_menu() {
  [[ "${DX_MENU:-}" == "0" ]] && return 1
  command -v dx >/dev/null 2>&1 || return 1
  local __dx_json
  __dx_json="$(dx menu --buffer "$COMP_LINE" --cursor "$COMP_POINT" --cwd "$PWD" --session "${DX_SESSION:-}" </dev/tty 2>/dev/tty)" || return 1
  [[ "$__dx_json" == *'"action":"replace"'* ]] || return 1
  local __dx_value="${__dx_json##*\"value\":\"}"
  __dx_value="${__dx_value%%\"*}"
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
    cd) _dx_complete_paths ;;
    up) _dx_complete_ancestors ;;
    cdf|z) _dx_complete_frecents ;;
    cdr) _dx_complete_recents ;;
    back|cd-) _dx_complete_stack_back ;;
    forward|cd+) _dx_complete_stack_forward ;;
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

    script
}
