## Context

Generated PowerShell init now loads an in-memory `dx` module. The module boundary makes exported command names visible through PowerShell-native tooling such as `Get-Command -Module dx`. That shifts the naming concern from only interactive convenience to module API presentation.

The current module exports short function names and aliases, for example `back`, `forward`, `cdf`, `cdr`, and `up`. In PowerShell style, those short spellings are better treated as aliases over approved verb-noun commands.

## Goals / Non-Goals

**Goals:**

- Expose PowerShell-style primary functions for dx navigation commands.
- Preserve all existing short command names as aliases for interactive use.
- Add `..` as an alias for upward navigation to match `cd-extras`-style ergonomics.
- Keep existing navigation behavior, stack behavior, completions, menu handling, and command-not-found handling unchanged.
- Ensure `Remove-Module dx` restores or removes every alias that dx installs.

**Non-Goals:**

- Changing command names for Bash, Zsh, or Fish.
- Changing the `dx` Rust CLI command names.
- Rewriting `Set-DxLocation` as a full native `Set-Location` replacement.
- Removing compatibility aliases such as `back`, `forward`, `cdf`, or `cdr`.

## Decisions

1. Primary PowerShell functions use approved-style names.

The generated module should define and export `Step-Up`, `Undo-Location`, `Redo-Location`, `Set-FrecentLocation`, and `Set-RecentLocation`. These names make module introspection feel native while keeping behavior centralized in one function per operation.

Alternative considered: keep short function names and add separate long aliases. That preserves implementation shape but does not improve `Get-Command -Module dx` output because the primary exported functions remain short names.

2. Short interactive names become aliases.

The module should install aliases from short command names to primary functions: `up` and `..` to `Step-Up`; `back` and `cd-` to `Undo-Location`; `forward` and `cd+` to `Redo-Location`; `cdf` and `z` to `Set-FrecentLocation`; and `cdr` to `Set-RecentLocation`.

Alternative considered: only provide idiomatic names and remove short names. That would break existing user muscle memory and existing shell completion/menu command contexts.

3. Completion and menu command contexts continue to bind short names.

Existing completion registrations for `up`, `back`, `forward`, `cdf`, `cdr`, `cd-`, `cd+`, and `z` should continue. PowerShell aliases should be enough for command invocation, but generated completer registration should continue covering the familiar command names. Adding completion bindings for the long primary names is optional if it can be done without changing existing behavior.

Alternative considered: switch completions only to the long names. That would make current aliases less useful interactively.

4. Module cleanup tracks the expanded alias set.

Because dx will additionally install `..` and retarget aliases for short names, the module import should capture previous alias targets for all dx-installed aliases. `OnRemove` should restore prior targets or remove aliases that did not exist before import.

Alternative considered: only clean up `cd`, `z`, `cd-`, and `cd+` as today. That leaves `up`, `back`, `forward`, `cdf`, `cdr`, and `..` behind or inconsistent after module unload.

## Risks / Trade-offs

- `..` may already exist as a user alias -> Capture and restore previous target on unload.
- Some PowerShell verbs may not be approved exactly as chosen -> `Set`, `Undo`, and `Redo` are common enough for this shell integration; `Step-Up` intentionally follows existing `cd-extras` naming even if less common than `Set-Location`.
- Exporting fewer short functions could affect tests that inspect exported command types -> Update tests to expect functions for primary names and aliases for short names.
- Completion registration for aliases and primary names can drift -> Keep behavior tests focused on generated binding strings for the supported command names.
