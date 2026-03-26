#!/usr/bin/env zsh

# Prototype Zsh hook for dx resolve integration.

function dx_cd() {
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

function cd() {
  dx_cd "$@"
}

function command_not_found_handler() {
  local cmd="$1"

  if [[ -n "${DX_RESOLVE_GUARD:-}" ]]; then
    print -u2 -- "zsh: command not found: $cmd"
    return 127
  fi

  if [[ "$cmd" != */* && "$cmd" != *.* && "$cmd" != ~* && "$cmd" != up && "$cmd" != ...* ]]; then
    print -u2 -- "zsh: command not found: $cmd"
    return 127
  fi

  local resolved
  resolved="$(DX_RESOLVE_GUARD=1 dx resolve "$cmd" 2>/dev/null)" || {
    print -u2 -- "zsh: command not found: $cmd"
    return 127
  }

  if [[ -n "$resolved" ]]; then
    builtin cd "$resolved" || return $?
    return 0
  fi

  print -u2 -- "zsh: command not found: $cmd"
  return 127
}
