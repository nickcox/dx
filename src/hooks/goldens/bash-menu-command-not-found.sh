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
  local __dx_dest=""
  local __dx_origin="$PWD"
  if [[ -n "$__dx_selector" ]]; then
    local __dx_target
    __dx_target="$(__dx_stack_run navigate "$__dx_op" "$__dx_selector")" || return 1
    [[ -n "$__dx_target" ]] || return 1
    __dx_dest="$(__dx_stack_run stack "$__dx_op" --preview --target "$__dx_target")" || return 1
  else
    __dx_dest="$(__dx_stack_run stack "$__dx_op" --preview)" || return 1
  fi

  [[ -n "$__dx_dest" ]] || return 1
  __dx_cd_native "$__dx_dest" || return $?
  __dx_stack_run stack "$__dx_op" --target "$__dx_dest" >/dev/null || {
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

cdr() {
  __dx_jump_mode recents "${1:-}"
}

cdf() {
  __dx_jump_mode frecents "${1:-}"
}

z() {
  cdf "$@"
}

_dx_complete_paths() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  COMPREPLY=()
  command -v dx >/dev/null 2>&1 || return 1
  local line
   while IFS= read -r line; do
     [[ -n "$line" ]] && COMPREPLY+=("$line")
   done < <(dx complete paths "$cur" 2>/dev/null)
}

_dx_complete_ancestors() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  COMPREPLY=()
  command -v dx >/dev/null 2>&1 || return 1
  local line
   while IFS= read -r line; do
     [[ -n "$line" ]] && COMPREPLY+=("$line")
   done < <(dx complete ancestors "$cur" 2>/dev/null)
}

_dx_complete_frecents() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  COMPREPLY=()
  command -v dx >/dev/null 2>&1 || return 1
  local line
   while IFS= read -r line; do
     [[ -n "$line" ]] && COMPREPLY+=("$line")
   done < <(dx complete frecents "$cur" 2>/dev/null)
}

_dx_complete_recents() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  COMPREPLY=()
  command -v dx >/dev/null 2>&1 || return 1
  local line
   while IFS= read -r line; do
     [[ -n "$line" ]] && COMPREPLY+=("$line")
   done < <(dx complete recents "$cur" 2>/dev/null)
}

_dx_complete_stack_back() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  COMPREPLY=()
  command -v dx >/dev/null 2>&1 || return 1
  local line
   while IFS= read -r line; do
     [[ -n "$line" ]] && COMPREPLY+=("$line")
   done < <(dx complete stack --direction back "$cur" 2>/dev/null)
}

_dx_complete_stack_forward() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  COMPREPLY=()
  command -v dx >/dev/null 2>&1 || return 1
  local line
   while IFS= read -r line; do
     [[ -n "$line" ]] && COMPREPLY+=("$line")
   done < <(dx complete stack --direction forward "$cur" 2>/dev/null)
}

_dx() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="dx"
                ;;
            dx,bookmarks)
                cmd="dx__subcmd__bookmarks"
                ;;
            dx,complete)
                cmd="dx__subcmd__complete"
                ;;
            dx,help)
                cmd="dx__subcmd__help"
                ;;
            dx,init)
                cmd="dx__subcmd__init"
                ;;
            dx,menu)
                cmd="dx__subcmd__menu"
                ;;
            dx,navigate)
                cmd="dx__subcmd__navigate"
                ;;
            dx,resolve)
                cmd="dx__subcmd__resolve"
                ;;
            dx,stack)
                cmd="dx__subcmd__stack"
                ;;
            dx__subcmd__bookmarks,add)
                cmd="dx__subcmd__bookmarks__subcmd__add"
                ;;
            dx__subcmd__bookmarks,help)
                cmd="dx__subcmd__bookmarks__subcmd__help"
                ;;
            dx__subcmd__bookmarks,list)
                cmd="dx__subcmd__bookmarks__subcmd__list"
                ;;
            dx__subcmd__bookmarks,prune)
                cmd="dx__subcmd__bookmarks__subcmd__prune"
                ;;
            dx__subcmd__bookmarks,remove)
                cmd="dx__subcmd__bookmarks__subcmd__remove"
                ;;
            dx__subcmd__bookmarks__subcmd__help,add)
                cmd="dx__subcmd__bookmarks__subcmd__help__subcmd__add"
                ;;
            dx__subcmd__bookmarks__subcmd__help,help)
                cmd="dx__subcmd__bookmarks__subcmd__help__subcmd__help"
                ;;
            dx__subcmd__bookmarks__subcmd__help,list)
                cmd="dx__subcmd__bookmarks__subcmd__help__subcmd__list"
                ;;
            dx__subcmd__bookmarks__subcmd__help,prune)
                cmd="dx__subcmd__bookmarks__subcmd__help__subcmd__prune"
                ;;
            dx__subcmd__bookmarks__subcmd__help,remove)
                cmd="dx__subcmd__bookmarks__subcmd__help__subcmd__remove"
                ;;
            dx__subcmd__complete,ancestors)
                cmd="dx__subcmd__complete__subcmd__ancestors"
                ;;
            dx__subcmd__complete,filesystem)
                cmd="dx__subcmd__complete__subcmd__filesystem"
                ;;
            dx__subcmd__complete,frecents)
                cmd="dx__subcmd__complete__subcmd__frecents"
                ;;
            dx__subcmd__complete,help)
                cmd="dx__subcmd__complete__subcmd__help"
                ;;
            dx__subcmd__complete,paths)
                cmd="dx__subcmd__complete__subcmd__paths"
                ;;
            dx__subcmd__complete,recents)
                cmd="dx__subcmd__complete__subcmd__recents"
                ;;
            dx__subcmd__complete,stack)
                cmd="dx__subcmd__complete__subcmd__stack"
                ;;
            dx__subcmd__complete__subcmd__help,ancestors)
                cmd="dx__subcmd__complete__subcmd__help__subcmd__ancestors"
                ;;
            dx__subcmd__complete__subcmd__help,filesystem)
                cmd="dx__subcmd__complete__subcmd__help__subcmd__filesystem"
                ;;
            dx__subcmd__complete__subcmd__help,frecents)
                cmd="dx__subcmd__complete__subcmd__help__subcmd__frecents"
                ;;
            dx__subcmd__complete__subcmd__help,help)
                cmd="dx__subcmd__complete__subcmd__help__subcmd__help"
                ;;
            dx__subcmd__complete__subcmd__help,paths)
                cmd="dx__subcmd__complete__subcmd__help__subcmd__paths"
                ;;
            dx__subcmd__complete__subcmd__help,recents)
                cmd="dx__subcmd__complete__subcmd__help__subcmd__recents"
                ;;
            dx__subcmd__complete__subcmd__help,stack)
                cmd="dx__subcmd__complete__subcmd__help__subcmd__stack"
                ;;
            dx__subcmd__help,bookmarks)
                cmd="dx__subcmd__help__subcmd__bookmarks"
                ;;
            dx__subcmd__help,complete)
                cmd="dx__subcmd__help__subcmd__complete"
                ;;
            dx__subcmd__help,help)
                cmd="dx__subcmd__help__subcmd__help"
                ;;
            dx__subcmd__help,init)
                cmd="dx__subcmd__help__subcmd__init"
                ;;
            dx__subcmd__help,menu)
                cmd="dx__subcmd__help__subcmd__menu"
                ;;
            dx__subcmd__help,navigate)
                cmd="dx__subcmd__help__subcmd__navigate"
                ;;
            dx__subcmd__help,resolve)
                cmd="dx__subcmd__help__subcmd__resolve"
                ;;
            dx__subcmd__help,stack)
                cmd="dx__subcmd__help__subcmd__stack"
                ;;
            dx__subcmd__help__subcmd__bookmarks,add)
                cmd="dx__subcmd__help__subcmd__bookmarks__subcmd__add"
                ;;
            dx__subcmd__help__subcmd__bookmarks,list)
                cmd="dx__subcmd__help__subcmd__bookmarks__subcmd__list"
                ;;
            dx__subcmd__help__subcmd__bookmarks,prune)
                cmd="dx__subcmd__help__subcmd__bookmarks__subcmd__prune"
                ;;
            dx__subcmd__help__subcmd__bookmarks,remove)
                cmd="dx__subcmd__help__subcmd__bookmarks__subcmd__remove"
                ;;
            dx__subcmd__help__subcmd__complete,ancestors)
                cmd="dx__subcmd__help__subcmd__complete__subcmd__ancestors"
                ;;
            dx__subcmd__help__subcmd__complete,filesystem)
                cmd="dx__subcmd__help__subcmd__complete__subcmd__filesystem"
                ;;
            dx__subcmd__help__subcmd__complete,frecents)
                cmd="dx__subcmd__help__subcmd__complete__subcmd__frecents"
                ;;
            dx__subcmd__help__subcmd__complete,paths)
                cmd="dx__subcmd__help__subcmd__complete__subcmd__paths"
                ;;
            dx__subcmd__help__subcmd__complete,recents)
                cmd="dx__subcmd__help__subcmd__complete__subcmd__recents"
                ;;
            dx__subcmd__help__subcmd__complete,stack)
                cmd="dx__subcmd__help__subcmd__complete__subcmd__stack"
                ;;
            dx__subcmd__help__subcmd__stack,back)
                cmd="dx__subcmd__help__subcmd__stack__subcmd__back"
                ;;
            dx__subcmd__help__subcmd__stack,forward)
                cmd="dx__subcmd__help__subcmd__stack__subcmd__forward"
                ;;
            dx__subcmd__help__subcmd__stack,push)
                cmd="dx__subcmd__help__subcmd__stack__subcmd__push"
                ;;
            dx__subcmd__stack,back)
                cmd="dx__subcmd__stack__subcmd__back"
                ;;
            dx__subcmd__stack,forward)
                cmd="dx__subcmd__stack__subcmd__forward"
                ;;
            dx__subcmd__stack,help)
                cmd="dx__subcmd__stack__subcmd__help"
                ;;
            dx__subcmd__stack,push)
                cmd="dx__subcmd__stack__subcmd__push"
                ;;
            dx__subcmd__stack__subcmd__help,back)
                cmd="dx__subcmd__stack__subcmd__help__subcmd__back"
                ;;
            dx__subcmd__stack__subcmd__help,forward)
                cmd="dx__subcmd__stack__subcmd__help__subcmd__forward"
                ;;
            dx__subcmd__stack__subcmd__help,help)
                cmd="dx__subcmd__stack__subcmd__help__subcmd__help"
                ;;
            dx__subcmd__stack__subcmd__help,push)
                cmd="dx__subcmd__stack__subcmd__help__subcmd__push"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        dx)
            opts="-h -V --help --version resolve init complete navigate bookmarks stack menu help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__bookmarks)
            opts="-h --json --help add remove list prune help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__bookmarks__subcmd__add)
            opts="-h --json --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__bookmarks__subcmd__help)
            opts="add remove list prune help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__bookmarks__subcmd__help__subcmd__add)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__bookmarks__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__bookmarks__subcmd__help__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__bookmarks__subcmd__help__subcmd__prune)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__bookmarks__subcmd__help__subcmd__remove)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__bookmarks__subcmd__list)
            opts="-h --json --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__bookmarks__subcmd__prune)
            opts="-h --json --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__bookmarks__subcmd__remove)
            opts="-h --json --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__complete)
            opts="-h --help paths ancestors frecents recents stack filesystem help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__complete__subcmd__ancestors)
            opts="-h --json --limit --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__complete__subcmd__filesystem)
            opts="-h --json --limit --help path directory file"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__complete__subcmd__frecents)
            opts="-h --json --limit --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__complete__subcmd__help)
            opts="paths ancestors frecents recents stack filesystem help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__complete__subcmd__help__subcmd__ancestors)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__complete__subcmd__help__subcmd__filesystem)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__complete__subcmd__help__subcmd__frecents)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__complete__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__complete__subcmd__help__subcmd__paths)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__complete__subcmd__help__subcmd__recents)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__complete__subcmd__help__subcmd__stack)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__complete__subcmd__paths)
            opts="-h --json --limit --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__complete__subcmd__recents)
            opts="-h --session --json --limit --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --session)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__complete__subcmd__stack)
            opts="-h --direction --session --json --limit --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --direction)
                    COMPREPLY=($(compgen -W "back forward" -- "${cur}"))
                    return 0
                    ;;
                --session)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help)
            opts="resolve init complete navigate bookmarks stack menu help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__bookmarks)
            opts="add remove list prune"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__bookmarks__subcmd__add)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__bookmarks__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__bookmarks__subcmd__prune)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__bookmarks__subcmd__remove)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__complete)
            opts="paths ancestors frecents recents stack filesystem"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__complete__subcmd__ancestors)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__complete__subcmd__filesystem)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__complete__subcmd__frecents)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__complete__subcmd__paths)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__complete__subcmd__recents)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__complete__subcmd__stack)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__init)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__menu)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__navigate)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__resolve)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__stack)
            opts="push back forward"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__stack__subcmd__back)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__stack__subcmd__forward)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__help__subcmd__stack__subcmd__push)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__init)
            opts="-h --command-not-found --menu --native-menu --help bash zsh fish pwsh"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__menu)
            opts="-h --buffer --cursor --cwd --session --prompt-row --mode --shell --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --buffer)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cwd)
                    COMPREPLY=()
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o plusdirs
                    fi
                    return 0
                    ;;
                --session)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --prompt-row)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --mode)
                    COMPREPLY=($(compgen -W "path directory file" -- "${cur}"))
                    return 0
                    ;;
                --shell)
                    COMPREPLY=($(compgen -W "bash zsh fish pwsh" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__navigate)
            opts="-h --session --help up back forward"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --session)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__resolve)
            opts="-h --list --json --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__stack)
            opts="-h --list --clear --direction --json --session --help push back forward help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --direction)
                    COMPREPLY=($(compgen -W "back forward both" -- "${cur}"))
                    return 0
                    ;;
                --session)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__stack__subcmd__back)
            opts="-h --session --target --preview --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --session)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --target)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__stack__subcmd__forward)
            opts="-h --session --target --preview --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --session)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --target)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__stack__subcmd__help)
            opts="push back forward help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__stack__subcmd__help__subcmd__back)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__stack__subcmd__help__subcmd__forward)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__stack__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__stack__subcmd__help__subcmd__push)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dx__subcmd__stack__subcmd__push)
            opts="-h --session --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --session)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _dx -o nosort -o bashdefault -o default dx
else
    complete -F _dx -o bashdefault -o default dx
fi


complete -o default -F _dx_complete_paths cd
complete -F _dx_complete_ancestors up
complete -F _dx_complete_frecents cdf
complete -F _dx_complete_frecents z
complete -F _dx_complete_recents cdr
complete -F _dx_complete_stack_back back
complete -F _dx_complete_stack_back cd-
complete -F _dx_complete_stack_forward forward
complete -F _dx_complete_stack_forward cd+

__dx_try_menu() {
  local __dx_mode_override="${1:-}"
  [[ "${DX_MENU:-}" == "0" ]] && return 1
  command -v dx >/dev/null 2>&1 || return 1

  __dx_json_extract_string() {
    local __dx_key="$1"
    local __dx_json_input="$2"
    local __dx_marker="\"$__dx_key\":\""
    [[ "$__dx_json_input" == *"$__dx_marker"* ]] || return 1

    local __dx_rest="${__dx_json_input#*"$__dx_marker"}"
    local __dx_i=0
    local __dx_len=${#__dx_rest}
    local __dx_escape=0
    local __dx_ch
    local __dx_out=""

    while (( __dx_i < __dx_len )); do
      __dx_ch="${__dx_rest:__dx_i:1}"
      if (( __dx_escape )); then
        case "$__dx_ch" in
          '"'|'\\'|'/') __dx_out+="$__dx_ch" ;;
          *) return 1 ;;
        esac
        __dx_escape=0
        ((__dx_i++))
        continue
      fi

      if [[ "$__dx_ch" == "\\" ]]; then
        __dx_escape=1
        ((__dx_i++))
        continue
      fi

      if [[ "$__dx_ch" == '"' ]]; then
        printf '%s' "$__dx_out"
        return 0
      fi

      __dx_out+="$__dx_ch"
      ((__dx_i++))
    done

    return 1
  }

  __dx_json_extract_uint() {
    local __dx_key="$1"
    local __dx_json_input="$2"
    local __dx_marker="\"$__dx_key\":"
    [[ "$__dx_json_input" == *"$__dx_marker"* ]] || return 1

    local __dx_rest="${__dx_json_input#*"$__dx_marker"}"
    local __dx_num="${__dx_rest%%[^0-9]*}"
    [[ -n "$__dx_num" ]] || return 1
    printf '%s' "$__dx_num"
  }

  local __dx_json
  if [[ -n "$__dx_mode_override" ]]; then
    __dx_json="$(dx menu --shell bash --mode "$__dx_mode_override" --buffer "$COMP_LINE" --cursor "$COMP_POINT" --cwd "$PWD" --session "${DX_SESSION:-}" </dev/tty 2>/dev/tty)" || return 1
  else
    __dx_json="$(dx menu --shell bash --buffer "$COMP_LINE" --cursor "$COMP_POINT" --cwd "$PWD" --session "${DX_SESSION:-}" </dev/tty 2>/dev/tty)" || return 1
  fi

  local __dx_action
  __dx_action="$(__dx_json_extract_string action "$__dx_json")" || return 1
  [[ "$__dx_action" == "cancel" ]] && return 0
  [[ "$__dx_action" == "replace" ]] || return 1

  local __dx_rs __dx_re
  __dx_rs="$(__dx_json_extract_uint replaceStart "$__dx_json")" || return 1
  __dx_re="$(__dx_json_extract_uint replaceEnd "$__dx_json")" || return 1
  (( __dx_re >= __dx_rs )) || return 1

  local __dx_value
  __dx_value="$(__dx_json_extract_string value "$__dx_json")" || return 1
  [[ -n "$__dx_value" ]] || return 1

  local __dx_terminal
  __dx_terminal="$(__dx_json_extract_string terminal "$__dx_json")" || return 1
  [[ "$__dx_terminal" == "clean" || "$__dx_terminal" == "dirty" ]] || return 1
  __dx_menu_terminal="$__dx_terminal"

  COMPREPLY=("$__dx_value")
  return 0
}

_dx_menu_wrapper() {
  local __dx_cmd="${COMP_WORDS[0]:-${COMP_LINE%% *}}"
  local __dx_menu_mode=""
  __dx_menu_terminal=""
  case "$__dx_cmd" in

  esac

  if [[ -n "$__dx_menu_mode" ]]; then
    if __dx_try_menu "$__dx_menu_mode"; then
      [[ "$__dx_menu_terminal" == "dirty" && -t 1 ]] && printf '\r' >/dev/tty
      return 0
    fi
  fi
  if __dx_try_menu; then
    [[ "$__dx_menu_terminal" == "dirty" && -t 1 ]] && printf '\r' >/dev/tty
    return 0
  fi
  case "$__dx_cmd" in
    cd) _dx_complete_paths ;;
    up) _dx_complete_ancestors ;;
    cdf|z) _dx_complete_frecents ;;
    cdr) _dx_complete_recents ;;
    back|cd-) _dx_complete_stack_back ;;
    forward|cd+) _dx_complete_stack_forward ;;
  esac
}

complete -o default -F _dx_menu_wrapper cd
complete -F _dx_menu_wrapper up
complete -F _dx_menu_wrapper cdf
complete -F _dx_menu_wrapper z
complete -F _dx_menu_wrapper cdr
complete -F _dx_menu_wrapper back
complete -F _dx_menu_wrapper forward
complete -F _dx_menu_wrapper cd-
complete -F _dx_menu_wrapper cd+


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
