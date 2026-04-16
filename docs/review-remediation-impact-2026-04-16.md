# Review Remediation Impact (2026-04-16)

Grounding artifact for planning follow-up work from `docs/reviews/whole-repo-review-2026-04-16.md`, aligned with current shell/CLI contracts in `tech-docs/cd-extras-cli-prd.md`, `tech-docs/shell-hook-guarding.md`, and lessons from `plans/architectural-doc-code-alignment/plan.md`.

## 1. Impacted modules/files and why they matter

### A) Atomic write safety (Major)

- `src/common/mod.rs:46-63` (`write_atomic_replace`)
  - Current fallback removes the target before retrying `rename`; a second failure can leave no persisted file.
- `src/bookmarks/storage.rs:95-126`
  - Bookmarks durability depends on the above helper; failure can drop user bookmark state.
- `src/stacks/storage.rs:73-99`
  - Session stack durability also depends on the same helper; failure can lose navigation history.

Why it matters: this is a correctness/data-loss boundary affecting persistent user state.

### B) `dx menu --cwd` honoring (Major)

- `src/cli/menu.rs:156-184`
  - CLI resolves effective cwd and passes it into menu candidate sourcing.
- `src/menu/mod.rs:41-53`
  - Paths mode still calls resolver completion without forwarding the provided cwd.
- `src/resolve/completion.rs:44-52`
  - Completion currently hard-reads process cwd (`std::env::current_dir()`), creating contract drift for `--cwd`.

Why it matters: `dx menu` exposes a cwd override contract (`tech-docs/cd-extras-cli-prd.md:64-66`) that paths mode currently violates.

### C) Legacy hook prototype removal / guard test alignment (Major)

- `tests/shell_hook_guard.rs:20-23`, `:46-49`, `:88-90`
  - Tests source legacy script `scripts/hooks/dx.bash` directly.
- `scripts/hooks/dx.bash`, `scripts/hooks/dx.zsh`
  - Prototype hooks still exist and can be mistaken as authoritative.
- `src/hooks/{bash,zsh,fish,pwsh}.rs` + `src/hooks/mod.rs`
  - These are declared source of truth by current docs (`tech-docs/shell-hook-guarding.md:46-56`).

Why it matters: safety-net drift can validate the wrong implementation and block confident cleanup.

### D) Flagged `cd` parsing at menu boundary (Major)

- `src/menu/buffer.rs:100-170`
  - Query region is everything after command token; flagged forms like `cd -P foo` / `cd -- foo` are parsed as query text rather than path argument.
- `src/cli/menu.rs:134-171`
  - Downstream logic assumes parsed query maps to a path-like token.

Why it matters: this is a user-facing shell-boundary correctness gap in interactive completion/menu flows.

### Supporting hygiene areas (Minor, only where enabling above)

- `src/resolve/mod.rs` (`Resolver.config` visibility) and `src/resolve/pipeline.rs` (unused mode parameter) from review findings.
- `src/test_support.rs` and hook-marker tests in `src/hooks/mod.rs` for clarity/diagnostics.

Why they matter: reduce incidental complexity while touching resolver/menu/hook seams.

## 2. Cross-cutting risks and dependencies

- **Contract baseline risk**: `tech-docs/` is current contract; remediation should keep behavior aligned with `cd-extras-cli-prd.md` and `shell-hook-guarding.md` before deleting legacy artifacts.
- **Single-boundary change risk**: menu/cd behavior spans Rust parser (`src/menu/buffer.rs`) and generated shell hooks (`src/hooks/*`). Partial fixes can create cross-shell divergence.
- **Persistence semantics risk**: replacing atomic-write strategy must preserve same-directory replacement guarantees on macOS and avoid introducing non-atomic cross-device moves.
- **Test architecture dependency**: removing `scripts/hooks/*` requires first migrating guard assertions to generated-hook paths (`dx init ...` output or hook generator tests) to avoid coverage regression.
- **PowerShell coupling risk**: PSReadLine Tab handler and `ConvertFrom-Json` path (`src/hooks/pwsh.rs:241-292`) are stable today; changing parse or wrapper strategy can ripple through fallback semantics and smoke feasibility.

## 3. Candidate phase breakdown (2-4 phases) with rationale

### Phase 1 — Data durability remediation (atomic writes)

Scope:
- Replace unsafe delete-and-retry behavior in `write_atomic_replace`.
- Add targeted failure-mode tests for bookmarks/stacks persistence paths.

Rationale:
- Highest user impact (possible data loss) and least shell-coupled; should be isolated first.

### Phase 2 — Menu cwd + flagged `cd` correctness in core parser/pipeline

Scope:
- Thread effective cwd explicitly through paths completion (menu path).
- Update `parse_buffer` to identify path argument region for supported flagged `cd` forms (`-P`, `-L`, `--`).
- Keep non-`cd` command parsing unchanged.

Rationale:
- Resolves two major shell-boundary correctness issues in one cohesive core slice before shell-script cleanup.

### Phase 3 — Hook authority cleanup and legacy prototype retirement

Scope:
- Repoint `tests/shell_hook_guard.rs` (or equivalent guard coverage) to generated hooks / init output.
- Remove or quarantine `scripts/hooks/dx.bash` and `scripts/hooks/dx.zsh` from active test pathways.
- Update docs pointers if file movement/removal occurs.

Rationale:
- Prevents test drift and enables explicit legacy deletion requested by user, after replacement coverage exists.

### Phase 4 — Shell parity hardening + minor hygiene bundle

Scope:
- Address minor follow-ups that de-risk main fixes (resolver API cleanup, test diagnostics/documentation).
- Validate PowerShell flag detection approach decision (keep current wrapper parsing vs ProxyCommand-based proxy).

Rationale:
- Keeps core remediation focused while still closing enabling hygiene debt in the same planning stream.

## 4. Verification implications (automated + shell smoke)

### Automated tests

- **Atomic-write tests**
  - Add failure-injection tests around replacement failures and assert old target survivability.
  - Cover both bookmark and stack store callers.
- **Menu cwd tests**
  - Add CLI/integration tests proving `dx menu --cwd <X>` paths mode candidate roots from `<X>` rather than process cwd.
- **Flagged `cd` parse tests**
  - Expand `src/menu/buffer.rs` tests with `cd -P foo`, `cd -- foo`, mixed spacing, and quoted-path forms.
- **Hook authority tests**
  - Ensure hook guard behavior is tested through generated scripts (`dx init <shell>`) not legacy prototype files.

### Shell smoke evidence (feasible subset)

- Reuse the matrix style from `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md`.
- Minimum expected evidence:
  - Bash/Zsh init + menu-noop fallback still pass after parser/cwd changes.
  - PowerShell init (`Out-String` single-script-block form) and menu fallback still pass.
  - Fish/interactive scenarios marked pass or not-feasible with explicit environment rationale.

## 5. Open design questions that the plan should explicitly gate

1. **Atomic replace contract gate**
   - What exact replacement semantics are accepted on macOS/Linux for “never delete old target unless replacement succeeded”?
   - Should this helper be limited to same-directory temp files only, with explicit invariant checks?

2. **`--cwd` threading gate**
   - Should cwd be injected only for menu paths mode, or generalized so resolver completion APIs can accept explicit cwd for broader embeddability?

3. **Flagged `cd` grammar gate**
   - Which flagged forms are officially supported in menu parsing (`-P`, `-L`, `--`, grouped/unknown flags), and when should parser fall back to noop/native completion?

4. **Legacy hook deletion gate**
   - Are `scripts/hooks/*` removed outright, moved to archival docs, or retained but excluded from tests/tooling? (User preference currently favors deletion consideration.)

5. **PowerShell parsing/flag-detection mechanism gate**
   - Current approach: explicit wrapper parsing + JSON action handling (`ConvertFrom-Json`) in generated script.
   - Alternative to evaluate: generating a proxy wrapper via `[System.Management.Automation.ProxyCommand]::Create(...)` to inherit `Set-Location` parameter binding/flags.
   - Decision criteria for justification:
     - measurable correctness gains on flagged/path edge cases,
     - no regression to menu fallback and PSReadLine integration,
     - maintainability/readability of generated hook output,
     - no new runtime dependency burden.
   - Default planning posture: keep current explicit wrapper unless proxy-based approach demonstrates clear net benefit with tests/smoke evidence.
