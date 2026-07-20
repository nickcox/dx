#!/usr/bin/env zsh

dx_demo_setup() {
  emulate -L zsh
  setopt ERR_RETURN NO_UNSET

  local demo_root="${DX_DEMO_ROOT:-${TMPDIR:-/tmp}/dx-menu-demo}"
  local state_root="${TMPDIR:-/tmp}/dx-menu-demo-state"
  local dx_bin="${DX_DEMO_BIN:-${commands[dx]:-}}"

  if [[ -z "$dx_bin" ]]; then
    print -u2 "dx demo: dx is not available"
    return 1
  fi

  dx_bin="${dx_bin:A}"
  export PATH="${dx_bin:h}:$PATH"

  # Keep destructive cleanup constrained to these exact fixture names.
  [[ "$demo_root" == */dx-menu-demo ]] || return 1
  [[ "$state_root" == */dx-menu-demo-state ]] || return 1

  command rm -rf -- "$demo_root" "$state_root"
  command mkdir -p -- \
    "$demo_root"/applications \
    "$demo_root"/archives \
    "$demo_root"/benchmarks \
    "$demo_root"/builds \
    "$demo_root"/crates \
    "$demo_root"/documentation/api \
    "$demo_root"/documentation/guides \
    "$demo_root"/documentation-generation-pipeline \
    "$demo_root"/documentation-preview-environment \
    "$demo_root"/dotfiles \
    "$demo_root"/downloads \
    "$demo_root"/downloaded-release-artifacts \
    "$demo_root"/examples \
    "$demo_root"/experiments \
    "$demo_root"/fixtures \
    "$demo_root"/integrations \
    "$demo_root"/packages \
    "$demo_root"/playground \
    "$demo_root"/projects/dashboard \
    "$demo_root"/projects/design-system \
    "$demo_root"/projects/devtools \
    "$demo_root"/prototypes \
    "$demo_root"/releases \
    "$demo_root"/sandbox \
    "$demo_root"/scripts \
    "$demo_root"/services \
    "$demo_root"/tests \
    "$demo_root"/tools \
    "$demo_root"/website \
    "$state_root"/runtime

  export XDG_RUNTIME_DIR="$state_root/runtime"
  export DX_SESSION="menu-demo"
  export DX_SEARCH_ROOTS="$demo_root"
  export DX_MENU_BORDER=0
  export DX_MENU_MAX_ROWS=8
  export DX_MENU_ITEM_MAX_LEN=40
  unset DX_MENU_DEBUG
  unset DX_MENU_COMMAND_MAPPINGS

  # Keep the recording independent from the user's prompt configuration.
  precmd_functions=()
  preexec_functions=()
  PROMPT='%F{cyan}%1~%f %F{green}❯%f '
  RPROMPT=''

  autoload -Uz compinit
  compinit -d "$state_root/zcompdump"
  eval "$("$dx_bin" init zsh --menu)"
  builtin cd "$demo_root"
}

dx_demo_setup
unfunction dx_demo_setup
