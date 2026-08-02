if not set -q DX_SESSION
  set -gx DX_SESSION $fish_pid
end

function __dx_is_path_like --argument __dx_cmd
  if string match -rq -- '.*/|^\.|^~|^\.{3,}$|.*-|.*_|.*\.\..*' "$__dx_cmd"
    return 0
  end
  return 1
end

function __dx_push_pwd
  if type -q dx
    dx stack push "$PWD" >/dev/null 2>/dev/null
  end
end

function __dx_stack_run
  if not type -q dx
    return 127
  end

  set -l __dx_origin "$PWD"
  builtin cd "$HOME" >/dev/null 2>/dev/null; or builtin cd /tmp >/dev/null 2>/dev/null
  or return 1
  dx $argv
  set -l __dx_status $status
  builtin cd "$__dx_origin" >/dev/null 2>/dev/null
  return $__dx_status
end

function __dx_cd_native
  builtin cd $argv
end

function __dx_nav_wrapper --argument mode selector
  if not type -q dx
    return 1
  end

  __dx_push_pwd

  set -l target
  set -l dx_status
  if test -n "$selector"
    set target (dx navigate $mode "$selector")
  else
    set target (dx navigate $mode)
  end
  set -l dx_status $status

  if test $dx_status -ne 0
    return $dx_status
  end
  test -n "$target"; or return 1

  __dx_cd_native "$target"
  set dx_status $status
  if test $dx_status -ne 0
    return $dx_status
  end

  __dx_push_pwd
  return 0
end

function __dx_stack_wrapper --argument op selector
  if not type -q dx
    return 1
  end

  set -l dest
  set -l origin "$PWD"
  if test -n "$selector"
    set -l target (__dx_stack_run navigate $op "$selector")
    or return 1
    test -n "$target"; or return 1
    set dest (__dx_stack_run stack $op --preview --target "$target")
    or return 1
  else
    set dest (__dx_stack_run stack $op --preview)
    or return 1
  end

  test -n "$dest"; or return 1
  __dx_cd_native "$dest"
  set -l dx_status $status
  if test $dx_status -ne 0
    return $dx_status
  end
  __dx_stack_run stack $op --target "$dest" >/dev/null
  set dx_status $status
  if test $dx_status -ne 0
    __dx_cd_native "$origin" >/dev/null 2>/dev/null
    return $dx_status
  end
  return 0
end

function __dx_jump_mode --argument mode query
  if not type -q dx
    return 1
  end

  set -l target
  set -l dx_status
  if test -n "$query"
    set -l values (dx complete $mode "$query" 2>/dev/null)
    set dx_status $status
    set target $values[1]
  else
    set -l values (dx complete $mode 2>/dev/null)
    set dx_status $status
    set target $values[1]
  end

  if test $dx_status -ne 0
    return $dx_status
  end
  test -n "$target"; or return 1

  __dx_push_pwd
  __dx_cd_native "$target"
  set dx_status $status
  if test $dx_status -ne 0
    return $dx_status
  end

  __dx_push_pwd
  return 0
end

function cd
  if test (count $argv) -eq 0
    __dx_push_pwd
    __dx_cd_native
    set -l __dx_status $status
    if test $__dx_status -eq 0
      __dx_push_pwd
    end
    return $__dx_status
  end

  if test (count $argv) -eq 1; and test "$argv[1]" = "-"
    __dx_push_pwd
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

  __dx_push_pwd
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

function up
  __dx_nav_wrapper up "$argv[1]"
end

function back
  __dx_stack_wrapper back "$argv[1]"
end

function forward
  __dx_stack_wrapper forward "$argv[1]"
end

function cd-
  back $argv
end

function cd+
  forward $argv
end

function cdf
  __dx_jump_mode frecents "$argv[1]"
end

function z
  cdf $argv
end

function cdr
  __dx_jump_mode recents "$argv[1]"
end

__DX_CLAP_COMPLETION__

__DX_FISH_COMPLETION_BINDINGS__
