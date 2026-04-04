pub fn generate(command_not_found: bool, menu: bool) -> String {
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
    dx stack push "$PWD" >/dev/null 2>/dev/null
  end
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
  if test -n "$selector"
    set target (dx navigate $mode "$selector")
  else
    set target (dx navigate $mode)
  end

  if test -z "$target"
    return 1
  end

  __dx_cd_native "$target"
  if test $status -ne 0
    return $status
  end

  __dx_push_pwd
  return 0
end

function __dx_stack_wrapper --argument op selector
  if not type -q dx
    return 1
  end

  set -l undo_or_redo
  if test "$op" = "back"
    set undo_or_redo "undo"
  else
    set undo_or_redo "redo"
  end

  set -l dest
  if test -n "$selector"
    set -l target (dx navigate $op "$selector")
    or return 1
    test -n "$target"; or return 1
    set dest (dx stack $undo_or_redo --target "$target")
    or return 1
  else
    set dest (dx stack $undo_or_redo)
    or return 1
  end

  test -n "$dest"; or return 1
  __dx_cd_native "$dest"
end

function __dx_jump_mode --argument mode query
  if not type -q dx
    return 1
  end

  set -l target
  if test -n "$query"
    set -l values (dx complete $mode "$query" 2>/dev/null)
    set target $values[1]
  else
    set -l values (dx complete $mode 2>/dev/null)
    set target $values[1]
  end

  if test -z "$target"
    return 1
  end

  __dx_cd_native "$target"
  if test $status -ne 0
    return $status
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

complete -c dx -n '__fish_use_subcommand' -a 'resolve complete init bookmarks stack navigate menu'
complete -c dx -n '__fish_seen_subcommand_from complete; and not __fish_seen_subcommand_from paths ancestors frecents recents stack' -a 'paths ancestors frecents recents stack'
complete -c dx -n '__fish_seen_subcommand_from resolve' -a '(dx complete paths (commandline -ct) 2>/dev/null)'

complete -c cd -a '(dx complete paths (commandline -ct) 2>/dev/null)'
complete -c up -a '(dx complete ancestors (commandline -ct) 2>/dev/null)'
complete -c cdf -a '(dx complete frecents (commandline -ct) 2>/dev/null)'
complete -c z -a '(dx complete frecents (commandline -ct) 2>/dev/null)'
complete -c cdr -a '(dx complete recents (commandline -ct) 2>/dev/null)'
complete -c back -a '(dx complete stack --direction back (commandline -ct) 2>/dev/null)'
complete -c cd- -a '(dx complete stack --direction back (commandline -ct) 2>/dev/null)'
complete -c forward -a '(dx complete stack --direction forward (commandline -ct) 2>/dev/null)'
complete -c cd+ -a '(dx complete stack --direction forward (commandline -ct) 2>/dev/null)'
"#,
    );

    if menu {
        script.push_str(
            r#"
function __dx_menu_complete
  if test "$DX_MENU" = "0"; or not type -q dx
    commandline -f complete
    return
  end

  set -l buf (commandline)
  set -l cur (commandline -C)
  set -l first (string split ' ' -- "$buf")[1]

  switch "$first"
    case cd up cdf z cdr back forward 'cd-' 'cd+'
      # dx navigation command — try menu
    case '*'
      commandline -f complete
      return
  end

  set -l json (dx menu --buffer "$buf" --cursor $cur --cwd "$PWD" --session "$DX_SESSION" </dev/tty 2>/dev/tty)
  if test $status -ne 0
    commandline -f complete
    return
  end

  if not string match -q '*"action":"replace"*' -- "$json"
    commandline -f complete
    return
  end

  set -l value (string replace -r '.*"value":"([^"]*)".*' '$1' -- "$json")
  set -l rs (string replace -r '.*"replaceStart":([0-9]+).*' '$1' -- "$json")
  set -l re (string replace -r '.*"replaceEnd":([0-9]+).*' '$1' -- "$json")

  set -l prefix (string sub -l $rs -- "$buf")
  set -l suffix (string sub -s (math $re + 1) -- "$buf")
  commandline -r -- "$prefix$value$suffix"
  commandline -C (math $rs + (string length "$value"))
end

bind \t __dx_menu_complete
"#,
        );
    }

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
