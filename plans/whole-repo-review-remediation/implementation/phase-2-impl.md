---
type: planning
entity: implementation-plan
plan: "whole-repo-review-remediation"
phase: 2
status: completed
created: "2026-04-16"
updated: "2026-04-16"
---

# Implementation Plan: Phase 2 - Fix Menu CWD and Flagged `cd` Parsing

> Implements [Phase 2](../phases/phase-2.md) of [whole-repo-review-remediation](../plan.md)

## Approach

Generalize path-completion sourcing to accept an explicit cwd from `dx menu --cwd` via a shared resolver seam, then teach menu-buffer parsing to isolate only the path argument region for a bounded set of flagged `cd` forms.

Parser behavior is shell-aware at the integration boundary: POSIX `cd -L/-P/--` forms are supported for Bash/Zsh menu flows, while PowerShell `--psreadline-mode` must preserve current wrapper semantics by treating those POSIX forms as fallback/non-intervention (noop/native completion path), not as supported PowerShell grammar.

## Affected Modules

| Module | Change Type | Description |
|--------|-------------|-------------|
| `src/cli` | modify | Preserve `--cwd` intake and ensure menu command forwards effective cwd through sourcing path. |
| `src/menu` | modify | Use explicit cwd in `paths` candidate sourcing and implement flagged-`cd` query/replacement boundaries. |
| `src/resolve` | modify | Add completion API path that accepts injected cwd instead of reading process cwd directly. |
| `tests` | modify/create | Add regression coverage for `--cwd` contract, approved flags, and unsupported-flag fallback behavior. |

## Required Context

| File | Why |
|------|-----|
| `src/cli/menu.rs` | CLI already computes effective cwd and passes it into candidate sourcing (`src/cli/menu.rs:156-184`, `:212-219`). |
| `src/menu/mod.rs` | `CompletionMode::Paths` currently ignores provided cwd at sourcing (`src/menu/mod.rs:49-53`); dedup path handling differs by mode (`:68-71`). |
| `src/resolve/completion.rs` | Path completion hard-reads process cwd and reuses it in multiple internal paths (`src/resolve/completion.rs:44-52`, `:64`, `:84-89`). |
| `src/menu/buffer.rs` | Current parser treats all post-command text as query (`src/menu/buffer.rs:141-170`). |
| `tests/menu_cli.rs` | Existing generated-hook/menu fallback contract anchors that must remain stable while parser behavior changes. |
| `src/hooks/pwsh.rs` | PowerShell Tab handler calls `dx menu ... --psreadline-mode`; parser changes must not reinterpret POSIX flags as native pwsh grammar (`src/hooks/pwsh.rs:266-286`). |
| `src/hooks/mod.rs` | Existing cross-shell fallback marker tests (`all_shells_freeze_menu_fallback_contract_markers`) anchor noop/native behavior expectations. |
| `docs/review-remediation-impact-2026-04-16.md` | Phase intent and grammar decision gates (`...:75-83`, `...:132-137`). |
| `plans/whole-repo-review-remediation/verification/shell-smoke-matrix.md` | Required phase-local smoke scenarios and shells (`...:9-13`). |

## Implementation Steps

### Step 1: Thread explicit cwd through paths completion

- **What**: Introduce an explicit resolver API seam for completion cwd, using one concrete shape:
  - add a new method variant (for example `collect_completion_candidates_with_limit_and_cwd(raw_query, limit, cwd: Option<&Path>)`) and keep existing methods delegating with `None` for backward compatibility.
  - update `collect_completion_candidates_impl` to accept `cwd: Option<&Path>` and resolve an effective cwd once.
  - redirect **all current internal cwd usages** in `src/resolve/completion.rs` to that effective cwd (`std::env::current_dir()` fallback site, `expand_filesystem_prefix` call, `FallbackPolicy::from_query_context`).
- **Where**: `src/resolve/completion.rs` (`collect_completion_candidates_*` path), `src/menu/mod.rs` (`source_candidates_with_meta` paths branch), and callsites in `src/cli/menu.rs`.
- **Why**: Current CLI contract accepts `--cwd`, but paths completion ignores it.
- **Considerations**: Keep non-path modes unchanged; when injected cwd is `None`, behavior must remain current-dir compatible.

### Step 2: Implement approved flagged-`cd` grammar in menu buffer parser

- **What**: Extend `parse_buffer` for command `cd` so replacement/query span only the path token for approved POSIX forms:
  - `cd -L <path>`
  - `cd -P <path>`
  - `cd -- <path>`
  and explicitly handle interactive pre-path states:
  - `cd -P ` / `cd -- ` (no path typed yet) should keep replacement start after flag token with `query=None` and `needs_space_prefix=false`.
- **Where**: `src/menu/buffer.rs` parser logic + parser tests.
- **Why**: Current parser incorrectly includes flags in replacement region.
- **Considerations**: Preserve quoting/drill-in handling (`unquote_shell_quoted`) and keep non-`cd` command parsing unchanged.

### Step 3: Enforce unsupported/grouped-flag fallback behavior

- **What**: For unsupported forms, parser should return no replaceable path region so menu integration falls back to noop/native completion behavior:
  - unknown/grouped flags: `cd -Q foo`, `cd -LP foo`, `cd -abc foo`
  - lone previous-dir form: `cd -`
  - PowerShell `--psreadline-mode`: POSIX `-L/-P/--` inputs are treated as non-intervention/fallback forms to preserve existing pwsh wrapper semantics.
- **Where**: `src/menu/buffer.rs` decision branches and menu CLI regression tests (`tests/menu_cli.rs` and/or parser tests).
- **Why**: Plan requires explicit support/fallback semantics rather than silent mis-parsing.
- **Considerations**: Keep fallback consistent with current noop contract (`dx menu` returns noop and shell wrappers invoke native fallback).

### Step 4: Record phase-local smoke expectations

- **What**: Update matrix rows for Phase 2 scenarios after implementation verification.
- **Where**: `plans/whole-repo-review-remediation/verification/shell-smoke-matrix.md` rows for Bash/Zsh/Fish/PowerShell cwd behavior and approved flagged forms (`:9-13`).
- **Why**: Phase acceptance explicitly requires shell evidence or `Not Feasible` rationale.
- **Considerations**: Fish may be `Not Feasible` if environment cannot run interactive scenario; rationale must be explicit.

## Testing Plan

| Test Type | What to Test | Expected Outcome |
|-----------|-------------|-----------------|
| Unit (parser) | `parse_buffer` on `cd -L`, `cd -P`, `cd --`, `cd -`, unknown/grouped forms, quoted flagged paths | Approved forms isolate path span only; unsupported/lone-dash forms return fallback/noop-compatible result |
| Integration (menu CLI) | `dx menu --cwd <path>` in `paths` mode | Candidate roots reflect provided cwd, not process cwd |
| Integration (pwsh-mode non-intervention) | `dx menu --psreadline-mode` with POSIX flagged forms | Returns noop/fallback behavior for POSIX-only flagged grammar in pwsh mode |
| Shell smoke | Matrix Phase 2 scenarios (`shell-smoke-matrix.md`) | Bash/Zsh/PowerShell evidence recorded; Fish pass/not-feasible annotated |

**Verify command:** `cargo test --lib -- --list | rg -q 'menu::buffer::tests::cd_flagged_forms_isolate_path_token' && cargo test --lib -- --list | rg -q 'menu::buffer::tests::cd_unsupported_and_lone_dash_forms_fall_back' && cargo test --test menu_cli -- --list | rg -q 'menu_paths_mode_honors_explicit_cwd' && cargo test --test menu_cli -- --list | rg -q 'menu_psreadline_mode_keeps_posix_flagged_cd_as_fallback' && cargo test --lib menu::buffer::tests::cd_flagged_forms_isolate_path_token -- --exact && cargo test --lib menu::buffer::tests::cd_unsupported_and_lone_dash_forms_fall_back -- --exact && cargo test --test menu_cli menu_paths_mode_honors_explicit_cwd -- --exact && cargo test --test menu_cli menu_psreadline_mode_keeps_posix_flagged_cd_as_fallback -- --exact && cargo test`

### Test Integrity Constraints

- Existing non-interactive noop contract tests in `tests/menu_cli.rs` must remain unchanged in semantics.
- Existing quoting/drill-in tests in `src/menu/buffer.rs` must still pass; flagged support must not break `cd '/path with space'/` behavior.
- Existing completion-path tests in `src/resolve/completion.rs` that rely on process cwd must be updated only where the new explicit-cwd API is intentionally used; default CLI completion behavior must remain backward compatible.

## Rollback Strategy

Rollback parser and completion-api changes together if regressions appear. Do not keep partial parser support without cwd threading, or cwd threading without fallback-safe flag parsing, because either half reintroduces contract drift.

## Open Decisions

| Decision | Options | Chosen | Rationale |
|----------|---------|--------|-----------|
| `cd -L <path>` | Support vs fallback | Support | Common native `cd` flag form; parser can cleanly isolate subsequent path token. |
| `cd -P <path>` | Support vs fallback | Support | Common native `cd` flag form; same replacement semantics as `-L`. |
| `cd -- <path>` | Support vs fallback | Support | Explicit end-of-options should map naturally to path token boundary. |
| `cd -P ` / `cd -- ` before path token | Menu intervention vs fallback | Intervention (empty query region after flags) | Enables expected completion at path-argument position once user presses Tab after flag token. |
| `cd -` (lone dash) | Special-case support vs fallback | Fallback | Preserves native previous-directory semantics and avoids path-completion misparse. |
| Quoted paths after approved flags | Support vs fallback | Support | Preserve existing quote/drill-in behavior for real path arguments (`cd -P '/tmp/with space'/`). |
| Unknown/grouped flags (`-Q`, `-LP`, `-abc`) | Attempt best-effort parse vs fallback | Fallback | Avoid incorrect replacements across shell-specific/ambiguous flag grammars; preserve native behavior. |
| PowerShell `--psreadline-mode` handling of POSIX `-L/-P/--` forms | Support vs fallback | Fallback | Preserve current PowerShell wrapper semantics; Phase 2 avoids redefining pwsh grammar. |
| CWD injection scope | Menu-only patch vs shared completion API seam | Shared completion API seam | Keeps future embeddability and aligns with phase gate on contract correctness. |

## Reality Check

### Code Anchors Used

| File | Symbol/Area | Why it matters |
|------|-------------|----------------|
| `src/cli/menu.rs:156-184` | effective cwd and `source_candidates_with_meta` call | Confirms CLI already computes cwd and passes it down. |
| `src/menu/mod.rs:49-53` | `CompletionMode::Paths` branch | Confirms paths sourcing currently ignores provided cwd. |
| `src/menu/mod.rs:68-71` | mode-specific canonical-cwd handling | Confirms non-path dedup behavior and prevents accidental over-fix scope. |
| `src/resolve/completion.rs:44-52` | `std::env::current_dir()` usage | Primary root cause of `--cwd` contract drift. |
| `src/resolve/completion.rs:64` + `:84-89` | filesystem expansion + fallback policy cwd usage | Must both consume injected effective cwd to avoid partial fixes. |
| `src/menu/buffer.rs:141-170` | query/replacement region derivation | Root cause of flagged-`cd` parsing defect. |
| `src/hooks/pwsh.rs:266-286` | PSReadLine `dx menu --psreadline-mode` callsite | Grounds shell-specific fallback requirement for POSIX-only forms. |
| `tests/menu_cli.rs:309-380` | hook fallback contract tests | Anchors noop/native fallback expectations for parser changes. |
| `plans/whole-repo-review-remediation/verification/shell-smoke-matrix.md:9-13` | Phase 2 required scenarios | Defines mandatory shell evidence to close this phase. |

### Mismatches / Notes

- Current parser has no formal flag grammar for `cd`; this phase introduces deliberately bounded, shell-aware behavior (POSIX support for Bash/Zsh, fallback for pwsh POSIX forms).
- Existing repo tests do not currently assert explicit `--cwd` behavior for menu `paths` mode; this phase adds that missing contract coverage.

## Execution Outcome

- **Status:** Completed.
- **Main implementation landed in:**
  - `src/resolve/completion.rs`
  - `src/menu/mod.rs`
  - `src/menu/buffer.rs`
  - `src/cli/menu.rs`
  - `tests/menu_cli.rs`
- **Follow-up slice landed to close initial review blockers / test-quality gaps:**
  - added fallback coverage for approved flags without trailing space
  - added quoted `-L` parser coverage parity
  - strengthened explicit-cwd assertions to validate concrete selected-path identity
  - added higher-level flagged replace-span CLI coverage (`menu_flagged_cd_replace_span_starts_at_path_token`)
- **Approach deviation:** None required; execution remained aligned with the approved implementation plan.
- **Final acceptance record:** `../reviews/impl-review-phase-2-collated-2026-04-16.md`.
