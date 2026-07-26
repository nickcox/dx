
function fish_command_not_found --argument __dx_cmd
  if set -q DX_RESOLVE_GUARD
    printf '%s: command not found\n' "$__dx_cmd" >&2
    return 127
  end

  if not __dx_is_path_like "$__dx_cmd"
    printf '%s: command not found\n' "$__dx_cmd" >&2
    return 127
  end

  if not type -q dx
    printf '%s: command not found\n' "$__dx_cmd" >&2
    return 127
  end

  set -lx DX_RESOLVE_GUARD 1
  set -l __dx_resolved (dx resolve "$__dx_cmd" 2>/dev/null)
  set -l __dx_resolve_status $status
  set -e DX_RESOLVE_GUARD

  if test $__dx_resolve_status -ne 0; or test -z "$__dx_resolved"
    printf '%s: command not found\n' "$__dx_cmd" >&2
    return 127
  end

  __dx_cd_native "$__dx_resolved"
  set -l __dx_cd_status $status
  if test $__dx_cd_status -ne 0
    return $__dx_cd_status
  end

  __dx_push_pwd
  return 0
end
