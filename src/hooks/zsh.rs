pub fn generate(command_not_found: bool) -> String {
    let mut script = String::from(
        r#"if [[ -z "${DX_SESSION:-}" ]]; then
  export DX_SESSION="$$"
fi

__dx_is_path_like() {
  local __dx_cmd="$1"
  [[ "$__dx_cmd" == */* || "$__dx_cmd" == .* || "$__dx_cmd" == ~* || "$__dx_cmd" == ...* ]]
}

__dx_push_pwd() {
  command -v dx >/dev/null 2>&1 || return 0
  dx push "$PWD" >/dev/null 2>&1 || true
}

__dx_cd_native() {
  builtin cd "$@"
}

cd() {
  local __dx_status=0

  if [[ $# -eq 0 ]]; then
    __dx_cd_native
    __dx_status=$?
    if [[ $__dx_status -eq 0 ]]; then
      __dx_push_pwd
    fi
    return $__dx_status
  fi

  if [[ "$1" == "-" && $# -eq 1 ]]; then
    __dx_cd_native -
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
    __dx_cd_native "$@"
    return $?
  fi

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
"#,
    );

    if command_not_found {
        script.push_str(
            r#"
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

  __dx_cd_native "$__dx_resolved" || return $?
  __dx_push_pwd
  return 0
}
"#,
        );
    }

    script
}
