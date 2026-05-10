## 1. Menu Action Contract

- [x] 1.1 Extend the `dx menu` replace action model to include a required shell-facing `terminal` value of `clean` or `dirty` while preserving `action=replace`.
- [x] 1.2 Emit `terminal=clean` for single-candidate fast-path replacements before any TUI setup or rendering occurs.
- [x] 1.3 Emit `terminal=dirty` for replacements produced after interactive TUI selection.
- [x] 1.4 Add or update JSON contract tests for replace actions with both terminal states.

## 2. Shell Hook Redraw Handling

- [x] 2.1 Update Zsh menu hook generation to parse `terminal` and skip `zle reset-prompt` when `terminal=clean`.
- [x] 2.2 Update Fish menu hook generation to parse `terminal` and skip `commandline -f repaint` when `terminal=clean`.
- [x] 2.3 Update PowerShell menu hook generation to parse `terminal` and skip `PSConsoleReadLine::InvokePrompt()` when `terminal=clean`.
- [x] 2.4 Treat missing or unrecognized `terminal` values as invalid payloads that trigger native fallback.
- [x] 2.5 Review Bash menu completion handling and keep or adjust its existing carriage-return behavior according to the new terminal-state contract.

## 3. Verification and Documentation

- [x] 3.1 Add hook-generation tests proving Zsh, Fish, and PowerShell condition prompt redraw on the parsed `terminal` value.
- [x] 3.2 Add integration tests proving single-candidate menu replacement emits `terminal=clean`.
- [x] 3.3 Add integration tests or script-level checks proving interactive-selection replacement paths emit or handle `terminal=dirty`.
- [x] 3.4 Run `openspec validate distinguish-menu-replace-redraw --strict`.
- [x] 3.5 Run targeted menu and shell-hook tests, then the full Rust test suite.
