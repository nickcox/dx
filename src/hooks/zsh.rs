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
  dx push "$PWD" >/dev/null 2>&1 || true
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

  __dx_cd_native "$__dx_target" || return $?
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
    __dx_dest="$(dx "$__dx_undo_or_redo" --target "$__dx_target")" || return 1
  else
    __dx_dest="$(dx "$__dx_undo_or_redo")" || return 1
  fi

  [[ -n "$__dx_dest" ]] || return 1
  __dx_cd_native "$__dx_dest"
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

  __dx_cd_native "$__dx_target" || return $?
  __dx_push_pwd
  return 0
}

cd() {
  local __dx_status=0

  if [[ $# -eq 0 ]]; then
    __dx_cd_native
    __dx_status=$?
    if [[ $__dx_status -eq 0 ]]; then
      __dx_push_pwd
    fi
    return $__dx_status
  fi

  if [[ "$1" == "-" && $# -eq 1 ]]; then
    __dx_cd_native -
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
    __dx_cd_native "$@"
    return $?
  fi

  local __dx_resolved=""
  if (( $+commands[dx] )); then
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
  (( $+commands[dx] )) || return 1
  local cur="$words[CURRENT]"
  local -a candidates
  candidates=("${(@f)$(dx complete paths "$cur" 2>/dev/null)}")
  (( ${#candidates} )) && compadd -a candidates
}

_dx_complete_ancestors() {
  (( $+commands[dx] )) || return 1
  local cur="$words[CURRENT]"
  local -a candidates
  candidates=("${(@f)$(dx complete ancestors "$cur" 2>/dev/null)}")
  (( ${#candidates} )) && compadd -a candidates
}

_dx_complete_frecents() {
  (( $+commands[dx] )) || return 1
  local cur="$words[CURRENT]"
  local -a candidates
  candidates=("${(@f)$(dx complete frecents "$cur" 2>/dev/null)}")
  (( ${#candidates} )) && compadd -a candidates
}

_dx_complete_recents() {
  (( $+commands[dx] )) || return 1
  local cur="$words[CURRENT]"
  local -a candidates
  candidates=("${(@f)$(dx complete recents "$cur" 2>/dev/null)}")
  (( ${#candidates} )) && compadd -a candidates
}

_dx_complete_stack_back() {
  (( $+commands[dx] )) || return 1
  local cur="$words[CURRENT]"
  local -a candidates
  candidates=("${(@f)$(dx complete stack --direction back "$cur" 2>/dev/null)}")
  (( ${#candidates} )) && compadd -a candidates
}

_dx_complete_stack_forward() {
  (( $+commands[dx] )) || return 1
  local cur="$words[CURRENT]"
  local -a candidates
  candidates=("${(@f)$(dx complete stack --direction forward "$cur" 2>/dev/null)}")
  (( ${#candidates} )) && compadd -a candidates
}

_dx_complete_dx() {
  local cur="$words[CURRENT]"
  local sub="$words[2]"

  if (( CURRENT == 2 )); then
    compadd -- resolve complete init mark unmark bookmarks push pop undo redo navigate
    return 0
  fi

  case "$sub" in
    resolve)
      _dx_complete_paths
      ;;
    complete)
      if (( CURRENT == 3 )); then
        compadd -- paths ancestors frecents recents stack
      fi
      ;;
    push)
      _path_files -/
      ;;
    *)
      ;;
  esac
  return 0
}

compdef _dx_complete_dx dx
compdef _dx_complete_paths cd
compdef _dx_complete_ancestors up
compdef _dx_complete_frecents cdf z
compdef _dx_complete_recents cdr
compdef _dx_complete_stack_back back 'cd-'
compdef _dx_complete_stack_forward forward 'cd+'
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
    cd|up|cdf|z|cdr|back|forward|cd-|cd+) ;;
    *)
      zle expand-or-complete
      return
      ;;
  esac

  local __dx_json
  __dx_json="$(dx menu --buffer "$BUFFER" --cursor $CURSOR --cwd "$PWD" --session "${DX_SESSION:-}" </dev/tty 2>/dev/tty)"
  local __dx_exit=$?

  # On cancel (noop) or error, just redraw the prompt at its current position
  # and leave the buffer unchanged — do NOT fall through to native completion.
  if [[ $__dx_exit -ne 0 ]] || [[ "$__dx_json" != *'"action":"replace"'* ]]; then
    zle reset-prompt
    return
  fi

  local __dx_value="${__dx_json##*\"value\":\"}"
  __dx_value="${__dx_value%%\"*}"
  local __dx_rs="${__dx_json##*\"replaceStart\":}"
  __dx_rs="${__dx_rs%%[,\}]*}"
  local __dx_re="${__dx_json##*\"replaceEnd\":}"
  __dx_re="${__dx_re%%[,\}]*}"

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

  __dx_cd_native "$__dx_resolved" || return $?
  __dx_push_pwd
  return 0
}
"#,
        );
    }

    script
}
