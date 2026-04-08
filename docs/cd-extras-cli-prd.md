# Product Requirements Document: dx Navigation Contracts (Cross-Shell)

## 1. Overview

This PRD describes the current, implemented command and shell-integration contract for `dx` across Bash, Zsh, Fish, and PowerShell.

### Objective

- Keep shell wrappers thin and local-directory-changing.
- Keep path resolution, candidate selection, and stack state semantics in the Rust binary.
- Keep the menu boundary machine-readable and dependency-free.

### Non-Goals (Phase 1)

- No runtime behavior changes in this documentation phase.
- No frecency-store architecture changes.

## 2. Architecture Contract

`dx` uses a two-layer architecture:

1. **Core CLI binary** (Rust): resolves paths, computes candidates, applies stack transitions, and emits machine-readable outputs.
2. **Shell hooks** (Bash/Zsh/Fish/PowerShell): call CLI commands, apply local shell buffer updates and directory changes, and route completion/menu interactions.

### 2.1 Core contract boundaries

- `dx resolve` returns one resolved path on success.
- `dx complete paths` and peer completion modes return completion candidates for shell consumption.
- `dx navigate` performs selector resolution for `up|back|forward` in Rust.
- `dx stack` handles stack push/undo/redo operations.
- `dx menu` returns JSON actions for shell buffer replacement logic.

## 3. Current CLI Surface

## 3.1 Path resolution

- `dx resolve <query>`
- For queries starting with `/`, `./`, `../`, `~`, or `~/`, resolution first uses filesystem/direct-path semantics; if that pass has a match, it is used. If not, the prefix is stripped and resolution continues through root-based abbreviation/fallback and bookmark lookup.
- If prefix stripping leaves an empty query (for example `~/` when HOME target is missing), resolution remains unresolved.

## 3.2 Completions

- `dx complete paths [query]`
- For `paths` queries starting with `/`, `./`, `../`, `~`, or `~/`, completion first uses filesystem/direct-path semantics; if that pass has matches, they are returned. If not, the prefix is stripped and completion continues through root-based abbreviation/fallback.
- If prefix stripping leaves an empty query (for example `~/` when HOME target is missing), `paths` completion returns no candidates.
- `dx complete ancestors [query]`
- `dx complete frecents [query]`
- `dx complete recents [query]`
- `dx complete stack --direction <back|forward> [query]`

## 3.3 Navigation and stack

- `dx navigate <up|back|forward> [selector]`
- `dx stack push <path>`
- `dx stack undo [--target <absolute-path>]`
- `dx stack redo [--target <absolute-path>]`

## 3.4 Menu boundary

- `dx menu --buffer <line> --cursor <byte-offset> [--cwd <path>] [--session <id>] [--prompt-row <row>]`
- stdout emits machine-readable JSON action payloads (`noop` or `replace`).

## 3.5 Initialization

- `dx init <bash|zsh|fish|pwsh> [--menu] [--command-not-found]`

## 4. Shell Hook Integration

## 4.1 Thin-wrapper model

- Hooks call native shell primitives (`builtin cd` / `Set-Location`) for directory changes.
- Hooks call `dx stack push` around successful transitions where required by mode semantics.
- `back`/`forward` style transitions call `dx stack undo` / `dx stack redo` and optional `--target` when selector-based.

## 4.2 Selector and completion model

- Completion mode routing is explicit and command-bound.
- Selector resolution for navigation remains in Rust via `dx navigate`.

## 4.3 Menu split I/O model

- stdout is reserved for machine-readable JSON action payloads.
- Interactive menu rendering/input runs through tty/dev-tty paths on POSIX shells and PSReadLine paths on PowerShell.
- Shell wrappers may parse payloads differently per shell, but must remain dependency-free and adhere to the same action contract.

## 5. PowerShell Initialization Contract

PowerShell init output must be evaluated as a single script block, using `Out-String`:

```powershell
Invoke-Expression ((& dx init pwsh | Out-String))
```

For menu mode:

```powershell
Invoke-Expression ((& dx init pwsh --menu | Out-String))
```

## 6. Documentation Guardrails

- This PRD is current-contract focused.
- Historical background may be summarized only in paraphrased form.
- Verbatim obsolete command spellings are intentionally excluded to keep verification enforceable.
