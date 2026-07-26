if [[ -z "${DX_SESSION:-}" ]]; then
  export DX_SESSION="$$"
fi

__dx_is_path_like() {
  local __dx_cmd="$1"
  [[ "$__dx_cmd" == */* || "$__dx_cmd" == .* || "$__dx_cmd" == ~* || "$__dx_cmd" == ...* || "$__dx_cmd" == *-* || "$__dx_cmd" == *_* || "$__dx_cmd" == *..* ]]
}

__dx_push_pwd() {
  command -v dx >/dev/null 2>&1 || return 0
  dx stack push "$PWD" >/dev/null 2>&1 || true
}

__dx_stack_run() {
  command -v dx >/dev/null 2>&1 || return 127

  (
    builtin cd "${HOME:-/tmp}" >/dev/null 2>&1 || builtin cd /tmp >/dev/null 2>&1 || return 1
    dx "$@"
  )
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
  local __dx_status=$?

  [[ $__dx_status -eq 0 ]] || return "$__dx_status"
  [[ -n "$__dx_target" ]] || return 1

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
  local __dx_origin="$PWD"
  if [[ -n "$__dx_selector" ]]; then
    local __dx_target
    __dx_target="$(__dx_stack_run navigate "$__dx_op" "$__dx_selector")" || return 1
    [[ -n "$__dx_target" ]] || return 1
    __dx_dest="$(__dx_stack_run stack "$__dx_undo_or_redo" --preview --target "$__dx_target")" || return 1
  else
    __dx_dest="$(__dx_stack_run stack "$__dx_undo_or_redo" --preview)" || return 1
  fi

  [[ -n "$__dx_dest" ]] || return 1
  __dx_cd_native "$__dx_dest" || return $?
  __dx_stack_run stack "$__dx_undo_or_redo" --target "$__dx_dest" >/dev/null || {
    __dx_cd_native "$__dx_origin" >/dev/null 2>&1
    return 1
  }
  return 0
}

__dx_jump_mode() {
  local __dx_mode="$1"
  local __dx_query="${2:-}"
  command -v dx >/dev/null 2>&1 || return 1
  local __dx_target=""
  local __dx_output=""
  if [[ -n "$__dx_query" ]]; then
    __dx_output="$(dx complete "$__dx_mode" "$__dx_query" 2>/dev/null)"
  else
    __dx_output="$(dx complete "$__dx_mode" 2>/dev/null)"
  fi
  local __dx_status=$?
  [[ $__dx_status -eq 0 ]] || return "$__dx_status"
  __dx_target="$(__dx_complete_first <<< "$__dx_output")"

  [[ -n "$__dx_target" ]] || return 1

  __dx_push_pwd
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

__DX_CLAP_COMPLETION__

__DX_BASH_COMPLETION_BINDINGS__
