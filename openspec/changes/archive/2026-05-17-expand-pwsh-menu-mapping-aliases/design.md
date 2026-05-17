## Context

`DX_MENU_COMMAND_MAPPINGS` is parsed during `dx init <shell> --menu` and rendered into generated shell hook code. PowerShell currently embeds a static array of `<command>=<mode>` entries and the global PSReadLine menu key handler compares the first buffer token against those literal command names.

PowerShell already maintains aliases and exposes one-way lookup by command definition through `Get-Alias -Definition <command>`. This lets generated PowerShell hook code expand a configured canonical command into the aliases visible in the current session without requiring Rust to inspect PowerShell state.

## Goals / Non-Goals

**Goals:**

- Let `DX_MENU_COMMAND_MAPPINGS="Get-ChildItem=path"` also match aliases such as `gci`, `dir`, or user-defined aliases whose definition is `Get-ChildItem`.
- Keep the mapping model one-way: configured command definition to aliases.
- Resolve PowerShell aliases once when generated hooks are loaded.
- Preserve direct mappings for configured commands whether or not aliases exist.
- Preserve behavior for Bash, Zsh, Fish, and the `dx menu` runtime.

**Non-Goals:**

- Inferring canonical commands from configured aliases such as `gci=path`.
- Re-resolving aliases on every Tab press.
- Adding a new environment variable, CLI flag, or cross-shell alias abstraction.
- Implementing PowerShell command-specific parsing beyond the existing first-token routing.

## Decisions

1. Expand aliases in generated PowerShell at hook load.

Rust will continue rendering the seed mappings from `DX_MENU_COMMAND_MAPPINGS`. The generated PowerShell script will convert those seeds into a lookup table when hooks are evaluated. This keeps alias discovery in the live PowerShell session, where user-defined aliases are available.

Alternative considered: expand aliases in Rust during `dx init`. That cannot see session-local aliases without launching PowerShell separately, and would not reflect aliases defined earlier in the user profile.

Alternative considered: expand aliases dynamically inside the Tab handler. That makes every completion pay alias lookup cost and allows behavior to shift during a session in ways that conflict with the existing frozen-registration model.

2. Use one-way lookup from configured command to aliases.

For each configured mapping, generated PowerShell code will map the configured command directly and call `Get-Alias -Definition <configured command> -ErrorAction SilentlyContinue` to add aliases that point to that definition. A mapping for `Get-ChildItem=path` can add `gci=path`; a mapping for `gci=path` maps only `gci` and does not infer `Get-ChildItem` or sibling aliases.

This keeps configuration explicit and avoids surprising expansion when a user intentionally maps a short alias differently from the canonical command.

3. Use a case-insensitive lookup table with explicit mappings taking precedence.

PowerShell hashtable lookup is case-insensitive by default, matching existing direct comparison behavior. The generated expansion should preserve configured command mappings first, then add derived aliases without overriding any explicit configured command mapping.

If two derived aliases collide, the first seed mapping wins. That avoids later mappings silently changing aliases created by earlier canonical mappings.

4. Store expanded mappings outside the per-key handler.

The generated hook should construct a global or script-scoped mapping table near menu setup, then the PSReadLine handler should perform a simple lookup by first token. This keeps the key handler fast and makes the frozen behavior clear.

## Risks / Trade-offs

- Alias definitions can differ by profile load order -> The behavior is frozen when dx hooks load; users who define aliases after dx init evaluation must reload the hooks.
- Some aliases may be intentionally different from their canonical command -> Only configured command definitions expand; explicitly configured commands take precedence over derived aliases.
- PowerShell hashtable casing can obscure duplicate spellings -> This matches existing PowerShell command-name comparison behavior and avoids case-sensitive surprises.
- User-defined alias expansion cannot be validated by Rust-only tests -> Unit tests can assert generated script structure and, where available, integration-style PowerShell tests can validate `Get-Alias -Definition` behavior.
