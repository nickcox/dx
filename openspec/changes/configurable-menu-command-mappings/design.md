## Context

`dx menu` currently maps a fixed set of command contexts (`cd`, `up`, `cdf`, `z`, `cdr`, `back`, `forward`, aliases) to existing completion pipelines. The requested change adds menu-backed completion for non-dx commands while keeping current reliability guarantees (TTY-only interaction, structured JSON action boundary, and native fallback semantics in hooks). To keep `dx menu` focused on menu behavior rather than configuration policy, command mappings should be resolved at init time and compiled into generated shell registrations. The system must remain shell-portable and avoid introducing command-specific parsing complexity in v1.

## Goals / Non-Goals

**Goals:**
- Support configurable menu-backed mappings for external commands when menu mode is globally enabled.
- Provide explicit per-command completion modes: `path`, `directory`, `file`.
- Use `dx-smart` resolution for mapped external commands (`ls`, `open`, `cat` examples).
- Keep `dx menu` config-agnostic for mapped commands by passing explicit mode at invocation time.
- Keep v1 scoped to replacing only the current token under cursor.
- Deliver concrete, shell-by-shell registration behavior for Bash, Zsh, Fish, and PowerShell.
- Preserve an extension seam for a future config-file mapping layer.

**Non-Goals:**
- Full command-specific argument grammars (flags, positional rules, multi-token semantic parsing).
- Replacing shell-native completion for unmapped commands.
- Shipping the future config-file layer in this change.

## Decisions

### 1) Env var schema for command mappings
Use a single mapping variable with explicit grammar:

- `DX_MENU_COMMAND_MAPPINGS="<command>=<mode>,..."`
- `mode ∈ {path,directory,file}`

Examples:
- `DX_MENU_COMMAND_MAPPINGS="ls=path,open=path,cat=file"`

`DX_MENU_COMMAND_MAPPINGS` is consumed by `dx init <shell> --menu` when hook output is generated.

Rationale: compact, shell-friendly, and explicit enough for validation and shell registration generation.

Alternatives considered:
- Multiple env vars per command (`DX_MENU_MAP_LS=...`): rejected due to discoverability and scaling issues.
- JSON env var: rejected due to shell quoting friction across pwsh/zsh/bash/fish.

### 2) Init owns mapping parsing, validation, and registration
`dx init <shell> --menu` owns mapped-command configuration processing.
- Parse `DX_MENU_COMMAND_MAPPINGS` during init generation.
- Validate every mapping before emitting shell code.
- Fail init generation if any mapping entry is invalid.
- Emit shell registrations that capture the mapped command name and explicit mode.

`dx menu` SHALL NOT read mapping configuration at runtime for mapped-command support.

Rationale: separates configuration policy from menu runtime behavior and keeps shell registration explicit.

Alternatives considered:
- Parse env mappings inside `dx menu`: rejected because it couples config parsing to runtime dispatch and makes future config layering harder.

### 3) Future config-layer seam lives in init generation
Future config-file support should plug into the init-generation path, not the `dx menu` runtime path.
- Current source: env only
- Future shape: config-file source plus env override policy
- Generated hooks remain the artifact that drives mapped-command behavior

Rationale: preserves a future extension seam without forcing runtime config lookups into menu execution.

### 4) Mode semantics and candidate pipelines
Define mapped mode behavior:
- `path`: filesystem path candidates (files + directories)
- `directory`: directory-only candidates
- `file`: file-only candidates

Generated shell routing invokes `dx menu --mode <mode>` for mapped commands. Candidate generation reuses existing completion/resolution primitives where possible, but mapped modes require a filesystem candidate source that can surface both files and directories before mode filtering is applied.

Rationale: explicit user intent and predictable completion sets.

Alternatives considered:
- Single “path-like” mode with post-hoc shell filtering: rejected due to inconsistent cross-shell behavior.

### 5) Fixed mapped resolver policy (`dx-smart`)
Mapped commands use `dx-smart`.
- `dx-smart` applies dx path intelligence (abbreviation/fallback-aware behavior) before returning replacement text.
- If no suitable candidate or action exists, hooks fall back natively via existing noop/error pathways.

Rationale: matches user request and reuses dx’s core value proposition.

Alternatives considered:
- Native-first fallback: rejected because unmapped commands already preserve native shell completion, so a per-mapping native override would add complexity without meaningful benefit.

### 6) v1 token scope boundary
Only the token under cursor is parsed/replaced for mapped commands.
- No per-command arg-index interpretation.
- No attempt to infer semantic role of other tokens.

Rationale: sharply bounded complexity and consistent with existing replacement contract semantics.

### 7) Shell-by-shell registration design
When `dx init <shell> --menu` is generated and mappings are present, hooks register mapped commands as follows:

- **Bash**: extend `complete -F` registrations so mapped command names dispatch to shared menu completion function, which calls `dx menu --mode <mode>` with `COMP_LINE`/`COMP_POINT`.
- **Zsh**: extend the existing menu widget with a generated command→mode case table so mapped commands route through the shared widget and invoke `dx menu --mode <mode>` with `BUFFER`/`CURSOR`.
- **Fish**: extend the existing menu helper with a generated command→mode switch so mapped commands route through the shared helper and invoke `dx menu --mode <mode>`.
- **PowerShell**: extend the existing PSReadLine Tab handler with a generated command→mode lookup so mapped commands route through the shared handler and invoke `dx menu --mode <mode>`, with unchanged `TabCompleteNext` fallback.

Rationale: uses explicit init-owned mappings while preserving each shell’s most natural menu integration path and avoiding duplicated per-command handler logic where a shared widget/handler already exists.

Command-name matching follows each shell’s native registration semantics.

### 8) Mapping changes require explicit re-init
Changing `DX_MENU_COMMAND_MAPPINGS` does not change existing generated hooks.
- Users must re-run `dx init <shell> --menu` after mapping changes.
- Previously generated hooks continue using the mapping set captured when they were generated.

Rationale: init-owned mappings are simpler and keep `dx menu` free of runtime configuration concerns.

## Risks / Trade-offs

- **[Risk] Malformed mappings could produce partial or confusing hook output** → Mitigation: fail init generation on invalid mapping entries rather than emitting partial registrations.
- **[Risk] Cross-shell differences in cursor indexing/token boundaries** → Mitigation: normalize replacement spans in core logic; add per-shell integration tests for token-only replacement.
- **[Risk] `path` mode could be noisy for very large trees** → Mitigation: reuse existing candidate caps/ranking and preserve noop/native fallback path.
- **[Trade-off] Mapping changes require re-running init** → Mitigation: document this explicitly and fail fast on invalid mappings so regeneration is predictable.
- **[Trade-off] v1 skips command-aware parsing** → Mitigation: explicit mode registrations keep upgrade path open for v2 command-specific grammars.

## Migration Plan

1. Add mapping schema parser and validator in the `dx init --menu` generation path.
2. Add explicit `dx menu --mode <mode>` handling for mapped external-command invocations.
3. Implement file-aware mode semantics with fixed `dx-smart` candidate behavior for mapped commands.
4. Update `dx init --menu` generation for Bash direct mapped-command registrations and Zsh/Fish/PowerShell shared-handler mapped-command routing.
5. Add unit/integration coverage for parser, init-failure behavior, explicit mode dispatch, token-only replacement, file/directory filtering, and native fallback behavior.
6. Document env schema, examples, and the requirement to re-run `dx init` after mapping changes.

Rollback strategy: unset `DX_MENU_COMMAND_MAPPINGS` and re-run `dx init` (or run init without `--menu`) to disable mapped behavior.

## Answered Questions

- Q: Should invalid mapping entries produce stderr diagnostics by default, or only behind debug logging?
  A: Invalid mapping should cause init generation to fail
- Q: Should command name matching be case-sensitive on all shells, or shell-native semantics per platform?
  A: Follow shell-native command registration semantics
- Q: For `open` on macOS, should default suggested mode remain `file` or allow mixed `path` by recommendation in docs?
  A: If mentioned in the docs, `open` should be path mapped, since it can open a directory in Finder as well as opening files in their configured applications
