## Context

`dx menu` currently returns the same `replace` JSON action for two different paths:

- A single-candidate fast path that returns before terminal size lookup, raw-mode setup, cursor hiding, row reservation, or TUI rendering.
- An interactive TUI path that may draw menu rows, clear terminal lines, hide/show the cursor, and move the cursor during cleanup.

Zsh, Fish, and PowerShell hooks currently redraw/repaint the prompt after every successful replacement. That is conservative for interactive TUI selection, but unnecessary and visually janky for the fast path where `dx menu` never touched the terminal.

## Goals / Non-Goals

**Goals:**

- Let `dx menu` communicate whether terminal presentation is clean or dirty when returning a replacement.
- Avoid prompt redraw for single-candidate fast-path replacements where no TUI was rendered.
- Preserve redraw for interactive selections and explicit cancellation paths where the TUI may have changed terminal state.
- Preserve existing native fallback behavior for noop, invalid payloads, command failure, and non-replace actions.
- Keep the JSON action contract simple for shell parsers.

**Non-Goals:**

- Redesign the menu action protocol beyond replacement terminal-state metadata.
- Change candidate sourcing, filtering, ranking, or replacement formatting.
- Remove terminal cleanup for interactive TUI paths.
- Change Bash completion insertion semantics beyond parsing/ignoring the redraw metadata where appropriate.

## Decisions

### Decision: Keep `action: "replace"` and add `terminal: "clean" | "dirty"`

Replacement actions SHALL remain `action=replace`, with an additional required field describing terminal presentation state when `dx menu` returns.

`terminal=clean` means `dx menu` did not render interactive UI or otherwise disturb terminal presentation, so shell hooks can apply the buffer edit without prompt redraw. `terminal=dirty` means `dx menu` may have rendered or cleaned up interactive UI, so shell hooks should refresh prompt display after applying the edit.

The menu action protocol is private between the `dx` binary and generated shell hooks, so compatibility with external JSON consumers is not a design constraint. The reason to keep one `replace` action is semantic consistency: `replaceStart`, `replaceEnd`, and `value` always describe the buffer edit, while `terminal` describes the terminal state that determines whether a prompt redraw is needed.

Alternative considered: introduce `action: "replaceAndRedraw"`. This makes dispatch explicit but either duplicates the replacement payload shape or creates an asymmetric protocol. Since replacement semantics are identical in clean and dirty cases, terminal state is better represented as required metadata on the replacement action.

### Decision: Fast-path replacements use `terminal: "clean"`

The single-candidate fast path SHALL emit replacements with `terminal=clean` because it exits before interactive terminal setup and rendering.

Interactive selections SHALL emit replacements with `terminal=dirty` because the TUI may have drawn below the prompt and cleanup may move or clear terminal rows.

### Decision: Cancellation remains redraw-capable through existing `cancel`

Explicit cancellation after an interactive menu remains a separate `cancel` action. Shell hooks may continue to redraw/reset prompt for cancellation, because cancellation only happens after interactive TUI entry.

No `terminal` field is needed for `cancel` unless future behavior introduces non-interactive cancel paths.

### Decision: Shell hooks branch redraw from terminal state after applying replacement

Zsh, Fish, and PowerShell hooks SHALL parse the `terminal` field and only run `zle reset-prompt`, `commandline -f repaint`, or `PSConsoleReadLine::InvokePrompt()` when it is `dirty`.

If the field is absent or not exactly `clean` or `dirty`, hooks SHALL treat the payload as invalid and use native fallback. This keeps the private protocol strict and avoids silently guessing about terminal state.

Bash may ignore the terminal-state field for buffer insertion because it delegates replacement insertion to Readline completion via `COMPREPLY`; however, any Bash-specific terminal cleanup hacks should be reviewed under the same distinction.

## Risks / Trade-offs

- [Risk] Some shells may not repaint their command line after programmatic buffer mutation unless explicitly asked. → Mitigation: add targeted shell-level tests/manual checks for fast-path replacement in Zsh, Fish, and PowerShell before skipping redraw in each shell.
- [Risk] POSIX string parsing of JSON fields can be brittle. → Mitigation: use a simple top-level string field with exact emitted values `clean` or `dirty`, and treat missing/unrecognized values as invalid payloads that trigger native fallback.
- [Risk] The name `terminal` could be too broad. → Mitigation: document it narrowly as terminal presentation state after `dx menu` returns; `clean` means no prompt redraw required, `dirty` means prompt redraw required.
