
__dx_menu_widget() {
  if [[ "${DX_MENU:-}" == "0" ]] || ! (( $+commands[dx] )); then
    zle expand-or-complete
    return
  fi

  local __dx_first="${BUFFER%% *}"
  local __dx_menu_mode=""
  case "$__dx_first" in
    __DX_ZSH_MENU_MAPPING_CASE__
    __DX_ZSH_MENU_CASE__ ) ;;
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
