pub fn generate(command_not_found: bool) -> String {
    let mut script = String::from(
        r#"if not set -q DX_SESSION
  set -gx DX_SESSION $fish_pid
end

function __dx_is_path_like --argument __dx_cmd
  if string match -rq -- '.*/|^\.|^~|^\.{3,}$' "$__dx_cmd"
    return 0
  end
  return 1
end

function __dx_push_pwd
  if type -q dx
    dx push "$PWD" >/dev/null 2>/dev/null
  end
end

function __dx_cd_native
  builtin cd $argv
end

function cd
  if test (count $argv) -eq 0
    __dx_cd_native
    set -l __dx_status $status
    if test $__dx_status -eq 0
      __dx_push_pwd
    end
    return $__dx_status
  end

  if test (count $argv) -eq 1; and test "$argv[1]" = "-"
    __dx_cd_native -
    set -l __dx_status $status
    if test $__dx_status -eq 0
      __dx_push_pwd
    end
    return $__dx_status
  end

  set -l __dx_flags
  set -l __dx_path_arg
  set -l __dx_seen_path 0

  for __dx_arg in $argv
    if test $__dx_seen_path -eq 0; and string match -qr -- '^-' "$__dx_arg"; and test "$__dx_arg" != "-"
      set __dx_flags $__dx_flags "$__dx_arg"
    else if test $__dx_seen_path -eq 0
      set __dx_path_arg "$__dx_arg"
      set __dx_seen_path 1
    end
  end

  if test -z "$__dx_path_arg"
    __dx_cd_native $argv
    return $status
  end

  set -l __dx_status 0
  if type -q dx
    set -l __dx_resolved (dx resolve "$__dx_path_arg" 2>/dev/null)
    set -l __dx_resolve_status $status
    if test $__dx_resolve_status -eq 0; and test -n "$__dx_resolved"
      __dx_cd_native $__dx_flags "$__dx_resolved"
      set __dx_status $status
    else
      __dx_cd_native $argv
      set __dx_status $status
    end
  else
    __dx_cd_native $argv
    set __dx_status $status
  end

  if test $__dx_status -eq 0
    __dx_push_pwd
  end

  return $__dx_status
end
"#,
    );

    if command_not_found {
        script.push_str(
            r#"
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
  if test $status -ne 0
    return $status
  end

  __dx_push_pwd
  return 0
end
"#,
        );
    }

    script
}
