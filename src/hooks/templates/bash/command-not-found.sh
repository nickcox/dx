
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
