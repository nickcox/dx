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

# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_dx_global_optspecs
    string join \n h/help V/version
end

function __fish_dx_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_dx_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_dx_using_subcommand
    set -l cmd (__fish_dx_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c dx -n "__fish_dx_needs_command" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_needs_command" -s V -l version -d 'Print version'
complete -c dx -n "__fish_dx_needs_command" -f -a "resolve"
complete -c dx -n "__fish_dx_needs_command" -f -a "init"
complete -c dx -n "__fish_dx_needs_command" -f -a "complete"
complete -c dx -n "__fish_dx_needs_command" -f -a "navigate"
complete -c dx -n "__fish_dx_needs_command" -f -a "bookmarks"
complete -c dx -n "__fish_dx_needs_command" -f -a "stack"
complete -c dx -n "__fish_dx_needs_command" -f -a "menu"
complete -c dx -n "__fish_dx_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c dx -n "__fish_dx_using_subcommand resolve" -l list
complete -c dx -n "__fish_dx_using_subcommand resolve" -l json
complete -c dx -n "__fish_dx_using_subcommand resolve" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand init" -l command-not-found
complete -c dx -n "__fish_dx_using_subcommand init" -l menu
complete -c dx -n "__fish_dx_using_subcommand init" -l native-menu
complete -c dx -n "__fish_dx_using_subcommand init" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand complete; and not __fish_seen_subcommand_from paths ancestors frecents recents stack filesystem help" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand complete; and not __fish_seen_subcommand_from paths ancestors frecents recents stack filesystem help" -f -a "paths"
complete -c dx -n "__fish_dx_using_subcommand complete; and not __fish_seen_subcommand_from paths ancestors frecents recents stack filesystem help" -f -a "ancestors"
complete -c dx -n "__fish_dx_using_subcommand complete; and not __fish_seen_subcommand_from paths ancestors frecents recents stack filesystem help" -f -a "frecents"
complete -c dx -n "__fish_dx_using_subcommand complete; and not __fish_seen_subcommand_from paths ancestors frecents recents stack filesystem help" -f -a "recents"
complete -c dx -n "__fish_dx_using_subcommand complete; and not __fish_seen_subcommand_from paths ancestors frecents recents stack filesystem help" -f -a "stack"
complete -c dx -n "__fish_dx_using_subcommand complete; and not __fish_seen_subcommand_from paths ancestors frecents recents stack filesystem help" -f -a "filesystem"
complete -c dx -n "__fish_dx_using_subcommand complete; and not __fish_seen_subcommand_from paths ancestors frecents recents stack filesystem help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from paths" -l limit -r
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from paths" -l json
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from paths" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from ancestors" -l limit -r
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from ancestors" -l json
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from ancestors" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from frecents" -l limit -r
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from frecents" -l json
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from frecents" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from recents" -l session -r
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from recents" -l limit -r
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from recents" -l json
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from recents" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from stack" -l direction -r -f -a "back\t''
forward\t''"
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from stack" -l session -r
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from stack" -l limit -r
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from stack" -l json
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from stack" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from filesystem" -l limit -r
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from filesystem" -l json
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from filesystem" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from help" -f -a "paths"
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from help" -f -a "ancestors"
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from help" -f -a "frecents"
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from help" -f -a "recents"
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from help" -f -a "stack"
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from help" -f -a "filesystem"
complete -c dx -n "__fish_dx_using_subcommand complete; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c dx -n "__fish_dx_using_subcommand navigate" -l session -r
complete -c dx -n "__fish_dx_using_subcommand navigate" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and not __fish_seen_subcommand_from add remove list prune help" -l json -d 'Output as JSON'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and not __fish_seen_subcommand_from add remove list prune help" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and not __fish_seen_subcommand_from add remove list prune help" -f -a "add" -d 'Save a bookmark for a directory'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and not __fish_seen_subcommand_from add remove list prune help" -f -a "remove" -d 'Remove a saved bookmark'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and not __fish_seen_subcommand_from add remove list prune help" -f -a "list" -d 'List saved bookmarks (default when no subcommand given)'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and not __fish_seen_subcommand_from add remove list prune help" -f -a "prune" -d 'Remove bookmarks whose target directory no longer exists'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and not __fish_seen_subcommand_from add remove list prune help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and __fish_seen_subcommand_from add" -l json -d 'Output as JSON'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and __fish_seen_subcommand_from remove" -l json -d 'Output as JSON'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and __fish_seen_subcommand_from remove" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and __fish_seen_subcommand_from prune" -l json -d 'Output as JSON'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and __fish_seen_subcommand_from prune" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and __fish_seen_subcommand_from help" -f -a "add" -d 'Save a bookmark for a directory'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and __fish_seen_subcommand_from help" -f -a "remove" -d 'Remove a saved bookmark'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and __fish_seen_subcommand_from help" -f -a "list" -d 'List saved bookmarks (default when no subcommand given)'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and __fish_seen_subcommand_from help" -f -a "prune" -d 'Remove bookmarks whose target directory no longer exists'
complete -c dx -n "__fish_dx_using_subcommand bookmarks; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c dx -n "__fish_dx_using_subcommand stack; and not __fish_seen_subcommand_from push back forward help" -l direction -r -f -a "back\t''
forward\t''
both\t''"
complete -c dx -n "__fish_dx_using_subcommand stack; and not __fish_seen_subcommand_from push back forward help" -l session -r
complete -c dx -n "__fish_dx_using_subcommand stack; and not __fish_seen_subcommand_from push back forward help" -l list
complete -c dx -n "__fish_dx_using_subcommand stack; and not __fish_seen_subcommand_from push back forward help" -l clear
complete -c dx -n "__fish_dx_using_subcommand stack; and not __fish_seen_subcommand_from push back forward help" -l json
complete -c dx -n "__fish_dx_using_subcommand stack; and not __fish_seen_subcommand_from push back forward help" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand stack; and not __fish_seen_subcommand_from push back forward help" -f -a "push"
complete -c dx -n "__fish_dx_using_subcommand stack; and not __fish_seen_subcommand_from push back forward help" -f -a "back" -d 'Step back through the session\'s history, the way `back` does'
complete -c dx -n "__fish_dx_using_subcommand stack; and not __fish_seen_subcommand_from push back forward help" -f -a "forward" -d 'Step forward again, the way `forward` does'
complete -c dx -n "__fish_dx_using_subcommand stack; and not __fish_seen_subcommand_from push back forward help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c dx -n "__fish_dx_using_subcommand stack; and __fish_seen_subcommand_from push" -l session -r
complete -c dx -n "__fish_dx_using_subcommand stack; and __fish_seen_subcommand_from push" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand stack; and __fish_seen_subcommand_from back" -l session -r
complete -c dx -n "__fish_dx_using_subcommand stack; and __fish_seen_subcommand_from back" -l target -r
complete -c dx -n "__fish_dx_using_subcommand stack; and __fish_seen_subcommand_from back" -l preview -d 'Print the destination without changing session history'
complete -c dx -n "__fish_dx_using_subcommand stack; and __fish_seen_subcommand_from back" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand stack; and __fish_seen_subcommand_from forward" -l session -r
complete -c dx -n "__fish_dx_using_subcommand stack; and __fish_seen_subcommand_from forward" -l target -r
complete -c dx -n "__fish_dx_using_subcommand stack; and __fish_seen_subcommand_from forward" -l preview -d 'Print the destination without changing session history'
complete -c dx -n "__fish_dx_using_subcommand stack; and __fish_seen_subcommand_from forward" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand stack; and __fish_seen_subcommand_from help" -f -a "push"
complete -c dx -n "__fish_dx_using_subcommand stack; and __fish_seen_subcommand_from help" -f -a "back" -d 'Step back through the session\'s history, the way `back` does'
complete -c dx -n "__fish_dx_using_subcommand stack; and __fish_seen_subcommand_from help" -f -a "forward" -d 'Step forward again, the way `forward` does'
complete -c dx -n "__fish_dx_using_subcommand stack; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c dx -n "__fish_dx_using_subcommand menu" -l buffer -d 'Full command-line buffer text' -r
complete -c dx -n "__fish_dx_using_subcommand menu" -l cursor -d 'Cursor byte position within the buffer' -r
complete -c dx -n "__fish_dx_using_subcommand menu" -l cwd -d 'Working directory (defaults to current directory)' -r -f -a "(__fish_complete_directories)"
complete -c dx -n "__fish_dx_using_subcommand menu" -l session -d 'Session identifier (defaults to DX_SESSION env var)' -r
complete -c dx -n "__fish_dx_using_subcommand menu" -l prompt-row -d 'Prompt row override for shells that can provide buffer cursor row' -r
complete -c dx -n "__fish_dx_using_subcommand menu" -l mode -d 'Explicit mapped-command menu mode for init-generated external command hooks' -r -f -a "path\t''
directory\t''
file\t''"
complete -c dx -n "__fish_dx_using_subcommand menu" -l shell -d 'Shell syntax used for replacement text' -r -f -a "bash\t''
zsh\t''
fish\t''
pwsh\t''"
complete -c dx -n "__fish_dx_using_subcommand menu" -s h -l help -d 'Print help'
complete -c dx -n "__fish_dx_using_subcommand help; and not __fish_seen_subcommand_from resolve init complete navigate bookmarks stack menu help" -f -a "resolve"
complete -c dx -n "__fish_dx_using_subcommand help; and not __fish_seen_subcommand_from resolve init complete navigate bookmarks stack menu help" -f -a "init"
complete -c dx -n "__fish_dx_using_subcommand help; and not __fish_seen_subcommand_from resolve init complete navigate bookmarks stack menu help" -f -a "complete"
complete -c dx -n "__fish_dx_using_subcommand help; and not __fish_seen_subcommand_from resolve init complete navigate bookmarks stack menu help" -f -a "navigate"
complete -c dx -n "__fish_dx_using_subcommand help; and not __fish_seen_subcommand_from resolve init complete navigate bookmarks stack menu help" -f -a "bookmarks"
complete -c dx -n "__fish_dx_using_subcommand help; and not __fish_seen_subcommand_from resolve init complete navigate bookmarks stack menu help" -f -a "stack"
complete -c dx -n "__fish_dx_using_subcommand help; and not __fish_seen_subcommand_from resolve init complete navigate bookmarks stack menu help" -f -a "menu"
complete -c dx -n "__fish_dx_using_subcommand help; and not __fish_seen_subcommand_from resolve init complete navigate bookmarks stack menu help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c dx -n "__fish_dx_using_subcommand help; and __fish_seen_subcommand_from complete" -f -a "paths"
complete -c dx -n "__fish_dx_using_subcommand help; and __fish_seen_subcommand_from complete" -f -a "ancestors"
complete -c dx -n "__fish_dx_using_subcommand help; and __fish_seen_subcommand_from complete" -f -a "frecents"
complete -c dx -n "__fish_dx_using_subcommand help; and __fish_seen_subcommand_from complete" -f -a "recents"
complete -c dx -n "__fish_dx_using_subcommand help; and __fish_seen_subcommand_from complete" -f -a "stack"
complete -c dx -n "__fish_dx_using_subcommand help; and __fish_seen_subcommand_from complete" -f -a "filesystem"
complete -c dx -n "__fish_dx_using_subcommand help; and __fish_seen_subcommand_from bookmarks" -f -a "add" -d 'Save a bookmark for a directory'
complete -c dx -n "__fish_dx_using_subcommand help; and __fish_seen_subcommand_from bookmarks" -f -a "remove" -d 'Remove a saved bookmark'
complete -c dx -n "__fish_dx_using_subcommand help; and __fish_seen_subcommand_from bookmarks" -f -a "list" -d 'List saved bookmarks (default when no subcommand given)'
complete -c dx -n "__fish_dx_using_subcommand help; and __fish_seen_subcommand_from bookmarks" -f -a "prune" -d 'Remove bookmarks whose target directory no longer exists'
complete -c dx -n "__fish_dx_using_subcommand help; and __fish_seen_subcommand_from stack" -f -a "push"
complete -c dx -n "__fish_dx_using_subcommand help; and __fish_seen_subcommand_from stack" -f -a "back" -d 'Step back through the session\'s history, the way `back` does'
complete -c dx -n "__fish_dx_using_subcommand help; and __fish_seen_subcommand_from stack" -f -a "forward" -d 'Step forward again, the way `forward` does'


complete -c cd -a '(dx complete paths (commandline -ct) 2>/dev/null)'
complete -c up -a '(dx complete ancestors (commandline -ct) 2>/dev/null)'
complete -c cdf -a '(dx complete frecents (commandline -ct) 2>/dev/null)'
complete -c z -a '(dx complete frecents (commandline -ct) 2>/dev/null)'
complete -c cdr -a '(dx complete recents (commandline -ct) 2>/dev/null)'
complete -c back -a '(dx complete stack --direction back (commandline -ct) 2>/dev/null)'
complete -c cd- -a '(dx complete stack --direction back (commandline -ct) 2>/dev/null)'
complete -c forward -a '(dx complete stack --direction forward (commandline -ct) 2>/dev/null)'
complete -c cd+ -a '(dx complete stack --direction forward (commandline -ct) 2>/dev/null)'

function __dx_menu_complete
  if test "$DX_MENU" = "0"; or not type -q dx
    commandline -f complete
    return
  end

  set -l buf (commandline)
  set -l cur (commandline -C)
  set -l cur_prefix (string sub -l $cur -- "$buf")
  set cur (string length --bytes -- "$cur_prefix")
  set -l first (string split ' ' -- "$buf")[1]
  set -l dx_menu_mode

  switch "$first"

    case cd up cdf z cdr back forward 'cd-' 'cd+'
      # dx navigation command — try menu
    case '*'
      commandline -f complete
      return
  end

  if test -n "$dx_menu_mode"
    set -l json (dx menu --shell fish --mode "$dx_menu_mode" --buffer "$buf" --cursor $cur --cwd "$PWD" --session "$DX_SESSION" </dev/tty 2>/dev/tty)
  else
    set -l json (dx menu --shell fish --buffer "$buf" --cursor $cur --cwd "$PWD" --session "$DX_SESSION" </dev/tty 2>/dev/tty)
  end
  if test $status -ne 0
    commandline -f complete
    return
  end

  set -l action (string replace -r '.*"action":"([^"]+)".*' '$1' -- "$json")
  if test "$action" = "cancel"
    commandline -C (string length -- "$buf")
    commandline -f repaint
    return
  end
  if test "$action" != "replace"
    commandline -f complete
    return
  end

  set -l value_match (string match -r '.*"value":"((\\.|[^"])*)".*' -- "$json")
  if test (count $value_match) -lt 2
    commandline -f complete
    return
  end
  set -l value_escaped "$value_match[2]"
  set -l value "$value_escaped"
  set value (string replace -a '\\"' '"' -- "$value")
  set value (string replace -a '\\\\' '\\' -- "$value")
  set value (string replace -a '\\/' '/' -- "$value")
  if test -z "$value"
    commandline -f complete
    return
  end

  set -l rs_match (string match -r '.*"replaceStart":([0-9]+).*' -- "$json")
  if test (count $rs_match) -lt 2
    commandline -f complete
    return
  end
  set -l rs "$rs_match[2]"

  set -l re_match (string match -r '.*"replaceEnd":([0-9]+).*' -- "$json")
  if test (count $re_match) -lt 2
    commandline -f complete
    return
  end
  set -l re "$re_match[2]"

  if test $re -lt $rs
    commandline -f complete
    return
  end

  set -l buflen (string length -- "$buf")
  if test $rs -gt $buflen; or test $re -gt $buflen
    commandline -f complete
    return
  end

  set -l terminal (string replace -r '.*\"terminal\":\"([^\"[:space:]]+)\".*' '$1' -- "$json")
  if test "$terminal" != "clean" -a "$terminal" != "dirty"
    commandline -f complete
    return
  end

  set -l prefix (string sub -l $rs -- "$buf")
  set -l suffix (string sub -s (math $re + 1) -- "$buf")
  commandline -r -- "$prefix$value$suffix"
  commandline -C (math $rs + (string length "$value"))
  if test "$terminal" = "dirty"
    commandline -f repaint
  end
end

bind \t __dx_menu_complete

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
