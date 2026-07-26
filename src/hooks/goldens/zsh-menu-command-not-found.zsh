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
  (( $+commands[dx] )) || return 127

  (
    builtin cd "${HOME:-/tmp}" >/dev/null 2>&1 || builtin cd /tmp >/dev/null 2>&1 || return 1
    dx "$@"
  )
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
  local __dx_status=$?

  [[ $__dx_status -eq 0 ]] || return $__dx_status
  [[ -n "$__dx_target" ]] || return 1

  builtin cd "$__dx_target" || return $?
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
  builtin cd "$__dx_dest" || return $?
  __dx_stack_run stack "$__dx_undo_or_redo" --target "$__dx_dest" >/dev/null || {
    builtin cd "$__dx_origin" >/dev/null 2>&1
    return 1
  }
  return 0
}

__dx_jump_mode() {
  local __dx_mode="$1"
  local __dx_query="${2:-}"
  (( $+commands[dx] )) || return 1
  local __dx_target=""
  local __dx_output=""
  if [[ -n "$__dx_query" ]]; then
    __dx_output="$(dx complete "$__dx_mode" "$__dx_query" 2>/dev/null)"
  else
    __dx_output="$(dx complete "$__dx_mode" 2>/dev/null)"
  fi
  local __dx_status=$?
  [[ $__dx_status -eq 0 ]] || return $__dx_status
  __dx_target="$(__dx_complete_first <<< "$__dx_output")"

  [[ -n "$__dx_target" ]] || return 1

  __dx_push_pwd
  builtin cd "$__dx_target" || return $?
  __dx_push_pwd
  return 0
}

cd() {
  local __dx_status=0

  if [[ $# -eq 0 ]]; then
    __dx_push_pwd
    builtin cd
    __dx_status=$?
    if [[ $__dx_status -eq 0 ]]; then
      __dx_push_pwd
    fi
    return $__dx_status
  fi

  if [[ "$1" == "-" && $# -eq 1 ]]; then
    __dx_push_pwd
    builtin cd -
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
    builtin cd "$@"
    return $?
  fi

  __dx_push_pwd
  local __dx_resolved=""
  if (( $+commands[dx] )); then
    __dx_resolved="$(dx resolve "$__dx_path_arg" 2>/dev/null)"
    if [[ $? -eq 0 && -n "$__dx_resolved" ]]; then
      builtin cd "${__dx_flags[@]}" "$__dx_resolved"
      __dx_status=$?
    else
      builtin cd "$@"
      __dx_status=$?
    fi
  else
    builtin cd "$@"
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

#compdef dx

autoload -U is-at-least

_dx() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
":: :_dx_commands" \
"*::: :->dx" \
&& ret=0
    case $state in
    (dx)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:dx-command-$line[1]:"
        case $line[1] in
            (resolve)
_arguments "${_arguments_options[@]}" : \
'--list[]' \
'--json[]' \
'-h[Print help]' \
'--help[Print help]' \
':query:_files -/' \
&& ret=0
;;
(init)
_arguments "${_arguments_options[@]}" : \
'--command-not-found[]' \
'(--native-menu)--menu[]' \
'(--menu)--native-menu[]' \
'-h[Print help]' \
'--help[Print help]' \
':shell:(bash zsh fish pwsh)' \
&& ret=0
;;
(complete)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
":: :_dx__subcmd__complete_commands" \
"*::: :->complete" \
&& ret=0

    case $state in
    (complete)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:dx-complete-command-$line[1]:"
        case $line[1] in
            (paths)
_arguments "${_arguments_options[@]}" : \
'--limit=[]:LIMIT:_default' \
'--json[]' \
'-h[Print help]' \
'--help[Print help]' \
'::query:_default' \
&& ret=0
;;
(ancestors)
_arguments "${_arguments_options[@]}" : \
'--limit=[]:LIMIT:_default' \
'--json[]' \
'-h[Print help]' \
'--help[Print help]' \
'::query:_default' \
&& ret=0
;;
(frecents)
_arguments "${_arguments_options[@]}" : \
'--limit=[]:LIMIT:_default' \
'--json[]' \
'-h[Print help]' \
'--help[Print help]' \
'::query:_default' \
&& ret=0
;;
(recents)
_arguments "${_arguments_options[@]}" : \
'--session=[]:SESSION:_default' \
'--limit=[]:LIMIT:_default' \
'--json[]' \
'-h[Print help]' \
'--help[Print help]' \
'::query:_default' \
&& ret=0
;;
(stack)
_arguments "${_arguments_options[@]}" : \
'--direction=[]:DIRECTION:(back forward)' \
'--session=[]:SESSION:_default' \
'--limit=[]:LIMIT:_default' \
'--json[]' \
'-h[Print help]' \
'--help[Print help]' \
'::query:_default' \
&& ret=0
;;
(filesystem)
_arguments "${_arguments_options[@]}" : \
'--limit=[]:LIMIT:_default' \
'--json[]' \
'-h[Print help]' \
'--help[Print help]' \
':kind:(path directory file)' \
'::query:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_dx__subcmd__complete__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:dx-complete-help-command-$line[1]:"
        case $line[1] in
            (paths)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(ancestors)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(frecents)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(recents)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(stack)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(filesystem)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(navigate)
_arguments "${_arguments_options[@]}" : \
'--session=[]:SESSION:_default' \
'-h[Print help]' \
'--help[Print help]' \
':mode:(up back forward)' \
'::selector:_default' \
&& ret=0
;;
(bookmarks)
_arguments "${_arguments_options[@]}" : \
'--json[Output as JSON]' \
'-h[Print help]' \
'--help[Print help]' \
":: :_dx__subcmd__bookmarks_commands" \
"*::: :->bookmarks" \
&& ret=0

    case $state in
    (bookmarks)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:dx-bookmarks-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
'--json[Output as JSON]' \
'-h[Print help]' \
'--help[Print help]' \
':name -- Bookmark name (alphanumeric, hyphens, underscores):_default' \
'::path -- Directory path to bookmark (defaults to current directory):_files -/' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'--json[Output as JSON]' \
'-h[Print help]' \
'--help[Print help]' \
':name -- Bookmark name to remove:_default' \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
'--json[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_dx__subcmd__bookmarks__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:dx-bookmarks-help-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(stack)
_arguments "${_arguments_options[@]}" : \
'--direction=[]:DIRECTION:(undo redo both)' \
'--session=[]:SESSION:_default' \
'--list[]' \
'--clear[]' \
'--json[]' \
'-h[Print help]' \
'--help[Print help]' \
":: :_dx__subcmd__stack_commands" \
"*::: :->stack" \
&& ret=0

    case $state in
    (stack)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:dx-stack-command-$line[1]:"
        case $line[1] in
            (push)
_arguments "${_arguments_options[@]}" : \
'--session=[]:SESSION:_default' \
'-h[Print help]' \
'--help[Print help]' \
':path:_files -/' \
&& ret=0
;;
(undo)
_arguments "${_arguments_options[@]}" : \
'--session=[]:SESSION:_default' \
'--target=[]:TARGET:_default' \
'--preview[Print the destination without changing session history]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(redo)
_arguments "${_arguments_options[@]}" : \
'--session=[]:SESSION:_default' \
'--target=[]:TARGET:_default' \
'--preview[Print the destination without changing session history]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_dx__subcmd__stack__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:dx-stack-help-command-$line[1]:"
        case $line[1] in
            (push)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(undo)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(redo)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(menu)
_arguments "${_arguments_options[@]}" : \
'--buffer=[Full command-line buffer text]:BUFFER:_default' \
'--cursor=[Cursor byte position within the buffer]:CURSOR:_default' \
'--cwd=[Working directory (defaults to current directory)]:CWD:_files -/' \
'--session=[Session identifier (defaults to DX_SESSION env var)]:SESSION:_default' \
'--prompt-row=[Prompt row override for shells that can provide buffer cursor row]:PROMPT_ROW:_default' \
'--mode=[Explicit mapped-command menu mode for init-generated external command hooks]:MODE:(path directory file)' \
'--shell=[Shell syntax used for replacement text]:SHELL:(bash zsh fish pwsh)' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_dx__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:dx-help-command-$line[1]:"
        case $line[1] in
            (resolve)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(init)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(complete)
_arguments "${_arguments_options[@]}" : \
":: :_dx__subcmd__help__subcmd__complete_commands" \
"*::: :->complete" \
&& ret=0

    case $state in
    (complete)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:dx-help-complete-command-$line[1]:"
        case $line[1] in
            (paths)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(ancestors)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(frecents)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(recents)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(stack)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(filesystem)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(navigate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(bookmarks)
_arguments "${_arguments_options[@]}" : \
":: :_dx__subcmd__help__subcmd__bookmarks_commands" \
"*::: :->bookmarks" \
&& ret=0

    case $state in
    (bookmarks)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:dx-help-bookmarks-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(stack)
_arguments "${_arguments_options[@]}" : \
":: :_dx__subcmd__help__subcmd__stack_commands" \
"*::: :->stack" \
&& ret=0

    case $state in
    (stack)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:dx-help-stack-command-$line[1]:"
        case $line[1] in
            (push)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(undo)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(redo)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(menu)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
}

(( $+functions[_dx_commands] )) ||
_dx_commands() {
    local commands; commands=(
'resolve:' \
'init:' \
'complete:' \
'navigate:' \
'bookmarks:' \
'stack:' \
'menu:' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'dx commands' commands "$@"
}
(( $+functions[_dx__subcmd__bookmarks_commands] )) ||
_dx__subcmd__bookmarks_commands() {
    local commands; commands=(
'add:Save a bookmark for a directory' \
'remove:Remove a saved bookmark' \
'list:List saved bookmarks (default when no subcommand given)' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'dx bookmarks commands' commands "$@"
}
(( $+functions[_dx__subcmd__bookmarks__subcmd__add_commands] )) ||
_dx__subcmd__bookmarks__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'dx bookmarks add commands' commands "$@"
}
(( $+functions[_dx__subcmd__bookmarks__subcmd__help_commands] )) ||
_dx__subcmd__bookmarks__subcmd__help_commands() {
    local commands; commands=(
'add:Save a bookmark for a directory' \
'remove:Remove a saved bookmark' \
'list:List saved bookmarks (default when no subcommand given)' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'dx bookmarks help commands' commands "$@"
}
(( $+functions[_dx__subcmd__bookmarks__subcmd__help__subcmd__add_commands] )) ||
_dx__subcmd__bookmarks__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'dx bookmarks help add commands' commands "$@"
}
(( $+functions[_dx__subcmd__bookmarks__subcmd__help__subcmd__help_commands] )) ||
_dx__subcmd__bookmarks__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'dx bookmarks help help commands' commands "$@"
}
(( $+functions[_dx__subcmd__bookmarks__subcmd__help__subcmd__list_commands] )) ||
_dx__subcmd__bookmarks__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'dx bookmarks help list commands' commands "$@"
}
(( $+functions[_dx__subcmd__bookmarks__subcmd__help__subcmd__remove_commands] )) ||
_dx__subcmd__bookmarks__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'dx bookmarks help remove commands' commands "$@"
}
(( $+functions[_dx__subcmd__bookmarks__subcmd__list_commands] )) ||
_dx__subcmd__bookmarks__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'dx bookmarks list commands' commands "$@"
}
(( $+functions[_dx__subcmd__bookmarks__subcmd__remove_commands] )) ||
_dx__subcmd__bookmarks__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'dx bookmarks remove commands' commands "$@"
}
(( $+functions[_dx__subcmd__complete_commands] )) ||
_dx__subcmd__complete_commands() {
    local commands; commands=(
'paths:' \
'ancestors:' \
'frecents:' \
'recents:' \
'stack:' \
'filesystem:' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'dx complete commands' commands "$@"
}
(( $+functions[_dx__subcmd__complete__subcmd__ancestors_commands] )) ||
_dx__subcmd__complete__subcmd__ancestors_commands() {
    local commands; commands=()
    _describe -t commands 'dx complete ancestors commands' commands "$@"
}
(( $+functions[_dx__subcmd__complete__subcmd__filesystem_commands] )) ||
_dx__subcmd__complete__subcmd__filesystem_commands() {
    local commands; commands=()
    _describe -t commands 'dx complete filesystem commands' commands "$@"
}
(( $+functions[_dx__subcmd__complete__subcmd__frecents_commands] )) ||
_dx__subcmd__complete__subcmd__frecents_commands() {
    local commands; commands=()
    _describe -t commands 'dx complete frecents commands' commands "$@"
}
(( $+functions[_dx__subcmd__complete__subcmd__help_commands] )) ||
_dx__subcmd__complete__subcmd__help_commands() {
    local commands; commands=(
'paths:' \
'ancestors:' \
'frecents:' \
'recents:' \
'stack:' \
'filesystem:' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'dx complete help commands' commands "$@"
}
(( $+functions[_dx__subcmd__complete__subcmd__help__subcmd__ancestors_commands] )) ||
_dx__subcmd__complete__subcmd__help__subcmd__ancestors_commands() {
    local commands; commands=()
    _describe -t commands 'dx complete help ancestors commands' commands "$@"
}
(( $+functions[_dx__subcmd__complete__subcmd__help__subcmd__filesystem_commands] )) ||
_dx__subcmd__complete__subcmd__help__subcmd__filesystem_commands() {
    local commands; commands=()
    _describe -t commands 'dx complete help filesystem commands' commands "$@"
}
(( $+functions[_dx__subcmd__complete__subcmd__help__subcmd__frecents_commands] )) ||
_dx__subcmd__complete__subcmd__help__subcmd__frecents_commands() {
    local commands; commands=()
    _describe -t commands 'dx complete help frecents commands' commands "$@"
}
(( $+functions[_dx__subcmd__complete__subcmd__help__subcmd__help_commands] )) ||
_dx__subcmd__complete__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'dx complete help help commands' commands "$@"
}
(( $+functions[_dx__subcmd__complete__subcmd__help__subcmd__paths_commands] )) ||
_dx__subcmd__complete__subcmd__help__subcmd__paths_commands() {
    local commands; commands=()
    _describe -t commands 'dx complete help paths commands' commands "$@"
}
(( $+functions[_dx__subcmd__complete__subcmd__help__subcmd__recents_commands] )) ||
_dx__subcmd__complete__subcmd__help__subcmd__recents_commands() {
    local commands; commands=()
    _describe -t commands 'dx complete help recents commands' commands "$@"
}
(( $+functions[_dx__subcmd__complete__subcmd__help__subcmd__stack_commands] )) ||
_dx__subcmd__complete__subcmd__help__subcmd__stack_commands() {
    local commands; commands=()
    _describe -t commands 'dx complete help stack commands' commands "$@"
}
(( $+functions[_dx__subcmd__complete__subcmd__paths_commands] )) ||
_dx__subcmd__complete__subcmd__paths_commands() {
    local commands; commands=()
    _describe -t commands 'dx complete paths commands' commands "$@"
}
(( $+functions[_dx__subcmd__complete__subcmd__recents_commands] )) ||
_dx__subcmd__complete__subcmd__recents_commands() {
    local commands; commands=()
    _describe -t commands 'dx complete recents commands' commands "$@"
}
(( $+functions[_dx__subcmd__complete__subcmd__stack_commands] )) ||
_dx__subcmd__complete__subcmd__stack_commands() {
    local commands; commands=()
    _describe -t commands 'dx complete stack commands' commands "$@"
}
(( $+functions[_dx__subcmd__help_commands] )) ||
_dx__subcmd__help_commands() {
    local commands; commands=(
'resolve:' \
'init:' \
'complete:' \
'navigate:' \
'bookmarks:' \
'stack:' \
'menu:' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'dx help commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__bookmarks_commands] )) ||
_dx__subcmd__help__subcmd__bookmarks_commands() {
    local commands; commands=(
'add:Save a bookmark for a directory' \
'remove:Remove a saved bookmark' \
'list:List saved bookmarks (default when no subcommand given)' \
    )
    _describe -t commands 'dx help bookmarks commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__bookmarks__subcmd__add_commands] )) ||
_dx__subcmd__help__subcmd__bookmarks__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'dx help bookmarks add commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__bookmarks__subcmd__list_commands] )) ||
_dx__subcmd__help__subcmd__bookmarks__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'dx help bookmarks list commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__bookmarks__subcmd__remove_commands] )) ||
_dx__subcmd__help__subcmd__bookmarks__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'dx help bookmarks remove commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__complete_commands] )) ||
_dx__subcmd__help__subcmd__complete_commands() {
    local commands; commands=(
'paths:' \
'ancestors:' \
'frecents:' \
'recents:' \
'stack:' \
'filesystem:' \
    )
    _describe -t commands 'dx help complete commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__complete__subcmd__ancestors_commands] )) ||
_dx__subcmd__help__subcmd__complete__subcmd__ancestors_commands() {
    local commands; commands=()
    _describe -t commands 'dx help complete ancestors commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__complete__subcmd__filesystem_commands] )) ||
_dx__subcmd__help__subcmd__complete__subcmd__filesystem_commands() {
    local commands; commands=()
    _describe -t commands 'dx help complete filesystem commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__complete__subcmd__frecents_commands] )) ||
_dx__subcmd__help__subcmd__complete__subcmd__frecents_commands() {
    local commands; commands=()
    _describe -t commands 'dx help complete frecents commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__complete__subcmd__paths_commands] )) ||
_dx__subcmd__help__subcmd__complete__subcmd__paths_commands() {
    local commands; commands=()
    _describe -t commands 'dx help complete paths commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__complete__subcmd__recents_commands] )) ||
_dx__subcmd__help__subcmd__complete__subcmd__recents_commands() {
    local commands; commands=()
    _describe -t commands 'dx help complete recents commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__complete__subcmd__stack_commands] )) ||
_dx__subcmd__help__subcmd__complete__subcmd__stack_commands() {
    local commands; commands=()
    _describe -t commands 'dx help complete stack commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__help_commands] )) ||
_dx__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'dx help help commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__init_commands] )) ||
_dx__subcmd__help__subcmd__init_commands() {
    local commands; commands=()
    _describe -t commands 'dx help init commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__menu_commands] )) ||
_dx__subcmd__help__subcmd__menu_commands() {
    local commands; commands=()
    _describe -t commands 'dx help menu commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__navigate_commands] )) ||
_dx__subcmd__help__subcmd__navigate_commands() {
    local commands; commands=()
    _describe -t commands 'dx help navigate commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__resolve_commands] )) ||
_dx__subcmd__help__subcmd__resolve_commands() {
    local commands; commands=()
    _describe -t commands 'dx help resolve commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__stack_commands] )) ||
_dx__subcmd__help__subcmd__stack_commands() {
    local commands; commands=(
'push:' \
'undo:' \
'redo:' \
    )
    _describe -t commands 'dx help stack commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__stack__subcmd__push_commands] )) ||
_dx__subcmd__help__subcmd__stack__subcmd__push_commands() {
    local commands; commands=()
    _describe -t commands 'dx help stack push commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__stack__subcmd__redo_commands] )) ||
_dx__subcmd__help__subcmd__stack__subcmd__redo_commands() {
    local commands; commands=()
    _describe -t commands 'dx help stack redo commands' commands "$@"
}
(( $+functions[_dx__subcmd__help__subcmd__stack__subcmd__undo_commands] )) ||
_dx__subcmd__help__subcmd__stack__subcmd__undo_commands() {
    local commands; commands=()
    _describe -t commands 'dx help stack undo commands' commands "$@"
}
(( $+functions[_dx__subcmd__init_commands] )) ||
_dx__subcmd__init_commands() {
    local commands; commands=()
    _describe -t commands 'dx init commands' commands "$@"
}
(( $+functions[_dx__subcmd__menu_commands] )) ||
_dx__subcmd__menu_commands() {
    local commands; commands=()
    _describe -t commands 'dx menu commands' commands "$@"
}
(( $+functions[_dx__subcmd__navigate_commands] )) ||
_dx__subcmd__navigate_commands() {
    local commands; commands=()
    _describe -t commands 'dx navigate commands' commands "$@"
}
(( $+functions[_dx__subcmd__resolve_commands] )) ||
_dx__subcmd__resolve_commands() {
    local commands; commands=()
    _describe -t commands 'dx resolve commands' commands "$@"
}
(( $+functions[_dx__subcmd__stack_commands] )) ||
_dx__subcmd__stack_commands() {
    local commands; commands=(
'push:' \
'undo:' \
'redo:' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'dx stack commands' commands "$@"
}
(( $+functions[_dx__subcmd__stack__subcmd__help_commands] )) ||
_dx__subcmd__stack__subcmd__help_commands() {
    local commands; commands=(
'push:' \
'undo:' \
'redo:' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'dx stack help commands' commands "$@"
}
(( $+functions[_dx__subcmd__stack__subcmd__help__subcmd__help_commands] )) ||
_dx__subcmd__stack__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'dx stack help help commands' commands "$@"
}
(( $+functions[_dx__subcmd__stack__subcmd__help__subcmd__push_commands] )) ||
_dx__subcmd__stack__subcmd__help__subcmd__push_commands() {
    local commands; commands=()
    _describe -t commands 'dx stack help push commands' commands "$@"
}
(( $+functions[_dx__subcmd__stack__subcmd__help__subcmd__redo_commands] )) ||
_dx__subcmd__stack__subcmd__help__subcmd__redo_commands() {
    local commands; commands=()
    _describe -t commands 'dx stack help redo commands' commands "$@"
}
(( $+functions[_dx__subcmd__stack__subcmd__help__subcmd__undo_commands] )) ||
_dx__subcmd__stack__subcmd__help__subcmd__undo_commands() {
    local commands; commands=()
    _describe -t commands 'dx stack help undo commands' commands "$@"
}
(( $+functions[_dx__subcmd__stack__subcmd__push_commands] )) ||
_dx__subcmd__stack__subcmd__push_commands() {
    local commands; commands=()
    _describe -t commands 'dx stack push commands' commands "$@"
}
(( $+functions[_dx__subcmd__stack__subcmd__redo_commands] )) ||
_dx__subcmd__stack__subcmd__redo_commands() {
    local commands; commands=()
    _describe -t commands 'dx stack redo commands' commands "$@"
}
(( $+functions[_dx__subcmd__stack__subcmd__undo_commands] )) ||
_dx__subcmd__stack__subcmd__undo_commands() {
    local commands; commands=()
    _describe -t commands 'dx stack undo commands' commands "$@"
}

if [ "$funcstack[1]" = "_dx" ]; then
    _dx "$@"
else
    compdef _dx dx
fi


compdef _dx_complete_paths cd
compdef _dx_complete_ancestors up
compdef _dx_complete_frecents cdf z
compdef _dx_complete_recents cdr
compdef _dx_complete_stack_back back 'cd-'
compdef _dx_complete_stack_forward forward 'cd+'

__dx_menu_widget() {
  if [[ "${DX_MENU:-}" == "0" ]] || ! (( $+commands[dx] )); then
    zle expand-or-complete
    return
  fi

  local __dx_first="${BUFFER%% *}"
  local __dx_menu_mode=""
  case "$__dx_first" in
    
    cd|up|cdf|z|cdr|back|forward|cd-|cd+ ) ;;
    *)
      zle expand-or-complete
      return
      ;;
  esac

  local __dx_json
  local __dx_cursor_bytes
  __dx_cursor_bytes="$(printf '%s' "${BUFFER[1,$CURSOR]}" | wc -c | tr -d ' ')"
  if [[ -n "$__dx_menu_mode" ]]; then
    __dx_json="$(dx menu --shell zsh --mode "$__dx_menu_mode" --buffer "$BUFFER" --cursor $__dx_cursor_bytes --cwd "$PWD" --session "${DX_SESSION:-}" </dev/tty 2>/dev/tty)"
  else
    __dx_json="$(dx menu --shell zsh --buffer "$BUFFER" --cursor $__dx_cursor_bytes --cwd "$PWD" --session "${DX_SESSION:-}" </dev/tty 2>/dev/tty)"
  fi
  local __dx_exit=$?

  # On runtime failure, leave the buffer unchanged and fall back
  # to native completion-equivalent behavior.
  if [[ $__dx_exit -ne 0 ]]; then
    zle expand-or-complete
    return
  fi

  local __dx_action_marker='"action":"'
  [[ "$__dx_json" == *$__dx_action_marker* ]] || { zle expand-or-complete; return }
  local __dx_action_rest="${__dx_json#*$__dx_action_marker}"
  local __dx_action="${__dx_action_rest%%\"*}"
  if [[ "$__dx_action" == "cancel" ]]; then
    CURSOR=${#BUFFER}
    zle reset-prompt
    return
  fi
  [[ "$__dx_action" == "replace" ]] || { zle expand-or-complete; return }

  local __dx_rs_marker='"replaceStart":'
  [[ "$__dx_json" == *$__dx_rs_marker* ]] || { zle expand-or-complete; return }
  local __dx_rs_rest="${__dx_json#*$__dx_rs_marker}"
  local __dx_rs="${__dx_rs_rest%%[^0-9]*}"
  [[ -n "$__dx_rs" ]] || { zle expand-or-complete; return }

  local __dx_re_marker='"replaceEnd":'
  [[ "$__dx_json" == *$__dx_re_marker* ]] || { zle expand-or-complete; return }
  local __dx_re_rest="${__dx_json#*$__dx_re_marker}"
  local __dx_re="${__dx_re_rest%%[^0-9]*}"
  [[ -n "$__dx_re" ]] || { zle expand-or-complete; return }

  (( __dx_re >= __dx_rs )) || { zle expand-or-complete; return }
  (( __dx_re <= ${#BUFFER} )) || { zle expand-or-complete; return }

  local __dx_value_marker='"value":"'
  [[ "$__dx_json" == *$__dx_value_marker* ]] || { zle expand-or-complete; return }
  local __dx_rest="${__dx_json#*$__dx_value_marker}"
  local __dx_value=""
  local __dx_i=1
  local __dx_len=${#__dx_rest}
  local __dx_escape=0
  local __dx_closed=0
  local __dx_ch

  while (( __dx_i <= __dx_len )); do
    __dx_ch="${__dx_rest[__dx_i]}"
    if (( __dx_escape )); then
      case "$__dx_ch" in
        '"'|'\\'|'/') __dx_value+="$__dx_ch" ;;
        *) zle expand-or-complete; return ;;
      esac
      __dx_escape=0
      (( __dx_i++ ))
      continue
    fi

    if [[ "$__dx_ch" == "\\" ]]; then
      __dx_escape=1
      (( __dx_i++ ))
      continue
    fi

    if [[ "$__dx_ch" == '"' ]]; then
      __dx_closed=1
      break
    fi

    __dx_value+="$__dx_ch"
    (( __dx_i++ ))
  done

  (( __dx_closed )) || { zle expand-or-complete; return }
  [[ -n "$__dx_value" ]] || { zle expand-or-complete; return }

  local __dx_terminal_marker="\"terminal\":\""
  [[ "$__dx_json" == *$__dx_terminal_marker* ]] || { zle expand-or-complete; return }
  local __dx_term_rest="${__dx_json#*$__dx_terminal_marker}"
  local __dx_terminal="${__dx_term_rest%%\"*}"
  [[ "$__dx_terminal" == "clean" || "$__dx_terminal" == "dirty" ]] || { zle expand-or-complete; return }

  BUFFER="${BUFFER[1,$__dx_rs]}${__dx_value}${BUFFER[$((${__dx_re}+1)),-1]}"
  CURSOR=$(( __dx_rs + ${#__dx_value} ))
  [[ "$__dx_terminal" == "dirty" ]] && zle reset-prompt
}

zle -N __dx_menu_widget
bindkey '^I' __dx_menu_widget

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
