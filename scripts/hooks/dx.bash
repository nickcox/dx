#!/usr/bin/env bash

# Prototype Bash hook for dx resolve integration.

dx_cd() {
  if [[ $# -eq 0 ]]; then
    builtin cd
    return $?
  fi

  local query="$1"
  local resolved
  resolved="$(dx resolve "$query" 2>/dev/null)" || return $?

  if [[ -n "$resolved" ]]; then
    builtin cd "$resolved" || return $?
    return 0
  fi

  builtin cd "$query"
}

cd() {
  dx_cd "$@"
}

command_not_found_handle() {
  local cmd="$1"

  if [[ -n "${DX_RESOLVE_GUARD:-}" ]]; then
    printf "%s: command not found\n" "$cmd" >&2
    return 127
  fi

  if [[ "$cmd" != *"/"* && "$cmd" != *"."* && "$cmd" != "~"* && "$cmd" != "up" && "$cmd" != ...* ]]; then
    printf "%s: command not found\n" "$cmd" >&2
    return 127
  fi

  local resolved
  resolved="$(DX_RESOLVE_GUARD=1 dx resolve "$cmd" 2>/dev/null)" || {
    printf "%s: command not found\n" "$cmd" >&2
    return 127
  }

  if [[ -n "$resolved" ]]; then
    builtin cd "$resolved" || return $?
    return 0
  fi

  printf "%s: command not found\n" "$cmd" >&2
  return 127
}
