
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
