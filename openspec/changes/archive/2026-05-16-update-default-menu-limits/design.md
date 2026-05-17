## Context

The menu defaults for `DX_MENU_ITEM_MAX_LEN` (currently `usize::MAX` — no truncation cap) and `DX_MENU_MAX_ROWS` (currently `10`) are conservative. With larger terminals common and longer candidate lists, both defaults should be more practical out of the box.

## Goals / Non-Goals

**Goals:**
- Change default for `DX_MENU_ITEM_MAX_LEN` from no cap to `80`
- Change default for `DX_MENU_MAX_ROWS` from `10` to `20`

**Non-Goals:**
- No new configuration variables
- No structural or behavioral changes beyond default values
- No changes to how the env vars are parsed or validated

## Decisions

**Decision 1: Simple literal default swap**
Replace the default sentinel (`usize::MAX`) with `80` in `parse_menu_item_max_len()`, and replace the fallback (`10`) with `20` in `parse_menu_max_rows()`. No abstraction layer or config file indirection — keep the current env-var-first architecture.

## Risks / Trade-offs

- **Breaking for scripted users relying on no-cap behavior**: If someone depends on `DX_MENU_ITEM_MAX_LEN` being unset to get no truncation, they now get an 80-char cap. Mitigation: trivial — they set `DX_MENU_ITEM_MAX_LEN=9999` or unset env var in their shell config.
- **Test assertions on defaults break**: Any test that asserts default `DX_MENU_MAX_ROWS=10` or no-cap behavior for `DX_MENU_ITEM_MAX_LEN` needs updating.
