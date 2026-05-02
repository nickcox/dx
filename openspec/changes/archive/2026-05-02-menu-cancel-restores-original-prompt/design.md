## Context

`dx menu` currently has three final outcome categories at the Rust layer: selected candidate, cancelled-with-query-change, and cancelled-without-query-change. Those are collapsed into just two wire actions for shell hooks: `replace` and `noop`.

That encoding worked while cancel without edits was expected to fall back into ordinary shell completion behavior. The current UX expectation is different: Escape means the menu consumed the keypress and should leave the prompt exactly as it was before the menu session started. That requires separating explicit cancel from generic fallback/noop conditions, because shell hooks currently treat non-`replace` menu output as permission to run native completion.

## Goals / Non-Goals

**Goals:**
- Make Escape cancel restore the original prompt token regardless of any typed in-menu refinement.
- Ensure explicit cancel does not trigger native completion insertion in any supported shell.
- Preserve existing fallback behavior for non-interactive, parse-failure, no-candidate, and runtime-error paths.
- Keep the action protocol simple and deterministic across Bash, Zsh, Fish, and PowerShell.

**Non-Goals:**
- Redesigning live filtering itself or removing the initial-query clamp.
- Introducing per-shell configurable cancel behavior.
- Reworking selection replacement semantics or prompt-range calculation.

## Decisions

### D1: Add an explicit final `cancel` action to the menu protocol

`dx menu` will continue to emit `replace` for successful selection and `noop` for non-handled fallback paths, but explicit user cancellation will emit a distinct `cancel` action.

Rationale: this keeps cancel semantics visible at the protocol boundary and lets shell hooks distinguish "user cancelled; do nothing" from "menu could not handle this; fall back natively".

Alternatives considered:
- Reuse `noop` and special-case shell behavior based on other fields or exit status. Rejected because it blurs fallback vs explicit-cancel semantics and makes hook logic brittle.
- Emit a `replace` that rewrites the original token back into place on cancel. Rejected because the desired result is no visible edit, and forcing a replacement adds unnecessary shell-buffer churn.

### D2: Cancel always restores the original prompt-derived query state

If the user presses Escape or Ctrl+C, the final menu action will reflect the original parsed query token, not any typed in-menu refinement.

Rationale: this matches the requested improvement and makes cancel behavior easy to explain: selection applies, cancel abandons session-local edits.

Alternatives considered:
- Preserve typed refinement on cancel when it differs from the initial query. Rejected because that is the behavior being intentionally removed.

### D3: Shell hooks handle `cancel` as terminal no-op, not native fallback

Each shell integration will treat `cancel` as a successful, handled outcome that leaves the current buffer unchanged and returns without invoking native completion.

Rationale: this directly fixes the observed "Escape inserts first completion item" behavior.

Alternatives considered:
- Keep hook logic unchanged and try to suppress native completion through shell-specific return-value tricks. Rejected because the hook layer already has a structured action channel; using it is cleaner and more portable.

## Risks / Trade-offs

- [Risk] Introducing a new action requires updating all menu-enabled hooks together -> Mitigation: capture the change in spec deltas and add generated-hook contract coverage for every shell.
- [Risk] Existing tests and docs may over-assume `noop` is the only non-replace final action -> Mitigation: update action serialization tests, CLI mapping tests, and hook contract assertions in the same change.
- [Trade-off] Cancel no longer preserves typed refinement that some users may have found useful -> acceptable because the change is intentionally redefining cancel as "discard interactive edits".

## Migration Plan

1. Update `dx-menu`, `dx-menu-filtering`, and `shell-hooks` specs to define explicit cancel behavior.
2. Add the new `cancel` menu action in Rust and map cancel outcomes to it from `dx menu`.
3. Update Bash, Zsh, Fish, and PowerShell menu hook handling so `cancel` returns without buffer mutation or native fallback.
4. Add regression tests for menu-action mapping and generated hook contracts.
5. Rollback strategy: remove the dedicated cancel action and restore the previous cancel-to-noop/replace behavior.

## Open Questions

- None at proposal time; the desired cancel behavior and fallback semantics are now explicit.
