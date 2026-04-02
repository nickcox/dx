## 1. Effective Root Set Construction

- [x] 1.1 Update resolver/config root assembly to include cwd as an implicit root by default.
- [x] 1.2 Normalize and deduplicate effective roots so cwd is not duplicated when already configured.
- [x] 1.3 Preserve configured-root ordering while appending implicit cwd fallback behavior.

## 2. Completion and Resolution Integration

- [x] 2.1 Ensure `dx complete paths` root-based candidate sourcing uses the effective root set including cwd.
- [x] 2.2 Ensure root-based resolution stages (abbreviation/fallback) use the same effective root set.
- [x] 2.3 Verify direct-path and step-up precedence behavior remains unchanged.

## 3. Verification and Documentation

- [x] 3.1 Add tests for no-config environments showing cwd-root-based completions/resolution working by default.
- [x] 3.2 Add tests for dedup behavior when cwd is already explicitly configured.
- [x] 3.3 Update docs/config references to describe implicit cwd root behavior.
- [x] 3.4 Run full test suite and adjust any baseline expectations affected by new default root inclusion.
