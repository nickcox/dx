
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
__DX_BASH_MENU_MAPPING_CASE__
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
__DX_BASH_MENU_FALLBACK_CASE__
  esac
}

complete -o default -F _dx_menu_wrapper cd
complete -F _dx_menu_wrapper up
complete -F _dx_menu_wrapper cdf
complete -F _dx_menu_wrapper z
complete -F _dx_menu_wrapper cdr
complete -F _dx_menu_wrapper back
complete -F _dx_menu_wrapper cd-
complete -F _dx_menu_wrapper forward
complete -F _dx_menu_wrapper cd+
__DX_BASH_MAPPED_MENU_BINDINGS__
