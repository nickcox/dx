---
type: planning
entity: implementation-plan
plan: "whole-repo-review-remediation"
phase: 1
status: completed
created: "2026-04-16"
updated: "2026-04-16"
---

# Implementation Plan: Phase 1 - Harden Persistence Writes

> Implements [Phase 1](../phases/phase-1.md) of [whole-repo-review-remediation](../plan.md)

## Approach

Replace the current delete-and-retry fallback in `write_atomic_replace` with a no-data-loss replacement contract: write temp in the target directory, attempt one replace operation, and if replacement fails, keep the existing target untouched while cleaning best-effort temp artifacts.

For deterministic cross-platform failure testing, use a **narrow test-only replace-failure seam in `src/common/mod.rs`** (around the replace syscall boundary) and validate durability at **caller level** inside bookmark/session storage tests. This avoids brittle permission/locking approaches while still exercising real `write_store`/`write_session` call paths.

## Affected Modules

| Module | Change Type | Description |
|--------|-------------|-------------|
| `src/common` | modify | Harden `write_atomic_replace` error path so existing targets are never deleted pre-replace. |
| `src/bookmarks` | modify | Preserve storage error mapping while validating safer replacement behavior in bookmark persistence tests. |
| `src/stacks` | modify | Preserve storage error mapping while validating safer replacement behavior in session persistence tests. |
| `tests` | modify/create | Add focused durability coverage for replacement-failure behavior (bookmark + session callers). |

## Required Context

| File | Why |
|------|-----|
| `src/common/mod.rs` | Current unsafe fallback is in `write_atomic_replace` (`src/common/mod.rs:46-63`). |
| `src/bookmarks/storage.rs` | Bookmark storage writes through atomic helper and maps helper errors (`src/bookmarks/storage.rs:95-126`). |
| `src/stacks/storage.rs` | Session storage writes through atomic helper and maps helper errors (`src/stacks/storage.rs:73-99`). |
| `src/bookmarks/mod.rs` | Caller-level storage tests may need store-construction helpers during durability assertions. |
| `src/stacks/mod.rs` | Caller-level session construction (`SessionStack`) may be needed for replace-failure tests. |
| `docs/reviews/whole-repo-review-2026-04-16.md` | Source review finding and severity for persistence defect (`...:42-43`, `...:61-63`). |
| `docs/review-remediation-impact-2026-04-16.md` | Scope and risk grounding for durability remediation (`...:7-17`, `...:66-74`). |

## Implementation Steps

### Step 1: Replace unsafe atomic fallback contract

- **What**: Refactor `write_atomic_replace` to remove `target.exists() -> remove_file(target) -> rename(temp,target)` logic and return `AtomicWriteError::Replace` immediately when replacement fails.
- **Where**: `src/common/mod.rs` (`write_atomic_replace`, currently `:46-63`).
- **Why**: Current flow can delete last-known-good data if second rename fails.
- **Considerations**: Keep same-directory temp usage intact; retain best-effort temp cleanup without mutating existing target on replace failures.

### Step 2: Preserve and sharpen caller error mapping

- **What**: Treat this as a verification-first step: confirm existing `AtomicWriteError::{Write,Replace}` mappings in bookmark/session storage remain correct after helper hardening; make caller code changes only if a concrete mismatch is found.
- **Where**: `src/bookmarks/storage.rs:114-125`, `src/stacks/storage.rs:88-98`.
- **Why**: Caller contracts should stay stable while underlying durability semantics improve.
- **Considerations**: Expected default is **no functional caller mapping change**; do not widen scope into storage format/serialization changes.

### Step 3: Add replacement-failure durability tests

- **What**: Add targeted caller-level tests that force replace failure via the Step 1 test seam and assert old target content survives for both bookmark and session persistence paths.
- **Where**: `src/bookmarks/storage.rs` test module and `src/stacks/storage.rs` test module (single chosen placement; no separate integration file in this phase).
- **Why**: Phase acceptance explicitly requires proving old target survivability.
- **Considerations**:
  - Use the test-only replace seam as the deterministic mechanism (not locking; not permissions toggling).
  - Keep test path through real `write_store` / `write_session` callers.
  - In session tests, keep `cleanup_stale` interaction explicit: ensure any test fixtures do not rely on files that `cleanup_stale` may touch before write; `.tmp` seam artifacts remain outside `is_session_file` matching.

## Testing Plan

| Test Type | What to Test | Expected Outcome |
|-----------|-------------|-----------------|
| Caller-level durability (unit) | Replacement failure in `write_store` and `write_session` preserves old file contents and returns replace error | Old on-disk payload remains intact for both callers |
| Regression | Existing read/write round-trip and cleanup tests still pass | No regression in normal persistence behavior |

**Verify command:** `cargo test --lib -- --list | rg -q 'bookmarks::storage::tests::write_store_replace_failure_preserves_last_known_good_target' && cargo test --lib -- --list | rg -q 'stacks::storage::tests::write_session_replace_failure_preserves_last_known_good_target' && cargo test --lib bookmarks::storage::tests::write_store_replace_failure_preserves_last_known_good_target -- --exact && cargo test --lib stacks::storage::tests::write_session_replace_failure_preserves_last_known_good_target -- --exact && cargo test`

### Test Integrity Constraints

- `src/bookmarks/storage.rs` existing tests (e.g., `write_then_read_round_trip_preserves_bookmarks`) must remain valid and should not be weakened.
- `src/stacks/storage.rs` existing tests (e.g., `write_then_read_round_trip_succeeds`) must remain valid and should not be weakened.
- Any newly added failure-injection seam must not bypass production write paths.

## Rollback Strategy

If post-change failures appear, revert helper + caller test changes as a single unit. Do not keep partial test-only assertions without the helper contract change, and do not ship a rollback that reintroduces target deletion before successful replacement.

## Open Decisions

| Decision | Options | Chosen | Rationale |
|----------|---------|--------|-----------|
| Failure behavior when replace step fails | (A) delete target then retry, (B) keep target and fail | B | Matches plan DoD and eliminates known data-loss path. |
| Failure-injection mechanism | (A) permissions/locking, (B) narrow test-only replace seam | B | Deterministic on macOS/Linux and CI; avoids locking non-effect and permission/root fragility. |
| Failure-injection placement | (A) storage module tests, (B) dedicated integration file | A | Keeps caller-level assertions close to `write_store`/`write_session` and avoids test target naming drift. |

## Reality Check

### Code Anchors Used

| File | Symbol/Area | Why it matters |
|------|-------------|----------------|
| `src/common/mod.rs:46-63` | `write_atomic_replace` | Contains unsafe delete-and-retry behavior that must be removed. |
| `src/bookmarks/storage.rs:95-126` | `write_store` | Direct caller path for bookmark durability. |
| `src/stacks/storage.rs:73-99` | `write_session` + `cleanup_stale` pre-write call | Session durability tests must account for pre-write cleanup behavior. |
| `docs/reviews/whole-repo-review-2026-04-16.md:42-43` | Major finding #1 | Confirms severity and concrete defect location. |
| `docs/review-remediation-impact-2026-04-16.md:9-15` | Impact mapping | Confirms bookmark/session coupling to atomic helper. |

### Mismatches / Notes

- Current repo tests do not yet explicitly force replace failure for both callers; this phase adds that missing coverage via a narrow test-only seam.
- `cleanup_stale` runs before `write_session`; tests must keep fixtures compatible with this pre-write behavior so replace-failure assertions stay deterministic.

## Execution Outcome

- Phase 1 implementation was completed and accepted in implementation review.
- No approach deviation from this implementation plan was required.
- Review outcome and deferred non-blocking follow-ups are captured in [Phase 1 collated review](../reviews/impl-review-phase-1-collated-2026-04-16.md).
