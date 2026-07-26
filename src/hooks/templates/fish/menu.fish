
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
__DX_FISH_MENU_MAPPING_CASES__
    case __DX_FISH_MENU_CASE_WORDS__
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
