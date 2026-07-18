## Context

The resolver, completion sources, menu filesystem sources, labels, and shell replacement formatter independently identify path syntax with Unix string operations. These operations do not model Windows drive prefixes, UNC roots, native separators, or root-relative paths. They also trim query text and sometimes use lossy display strings as path identity.

`dx resolve` is an exact command with a strict output contract, while completion and menu candidate discovery are interactive and intentionally best-effort. Windows releases are produced, but the full Rust suite currently runs only on Ubuntu.

## Goals / Non-Goals

**Goals:**

- Establish one portable query interpretation boundary shared by resolver, completion, and mapped menu filesystem sources.
- Preserve native root structure and `PathBuf` identity through internal processing.
- Define explicit-resolution I/O failures without making interactive completion fallible.
- Test supported semantics on native Windows CI and retain Unix behavior.

**Non-Goals:**

- Implement the interactive TUI on Windows; its existing noop fallback remains unchanged.
- Add an arbitrary non-UTF path encoding to JSON or shell replacement actions.
- Support Windows drive-relative paths such as `C:work`.
- Provision a network share for live UNC traversal tests.

## Decisions

### Shared path-query classifier

Create a small internal resolver path-query module that classifies query text before resolution. It will identify home-relative, absolute, root-relative, explicit-relative, unsupported drive-relative, trailing-separator, and abbreviation-segment forms. It will expose native root anchors and separator-aware segment boundaries.

Use `Path`, `Component`, `Path::is_absolute`, `Path::has_root`, and `std::path::is_separator` rather than duplicated string tests. On Windows, both slash styles are accepted; on Unix, backslash remains an ordinary filename character.

Alternative: convert all input to `/`-separated syntax. Rejected because it would corrupt valid Unix backslash names and cannot represent Windows prefixes faithfully.

### Native path construction and normalization

Construct expanded paths with `PathBuf::join` and normalize lexical components while retaining `Prefix` and `RootDir`. Parent traversal cannot escape a Unix root, drive root, or UNC share root. Home expansion uses `dirs::home_dir()`.

Windows root-relative queries are resolved against the cwd's native drive/share root. Drive-relative paths are rejected because their meaning depends on per-drive process state that `dx` does not own.

Alternative: delegate all normalization to `canonicalize`. Rejected because it requires paths to exist and does not support fallback matching for missing prefixes.

### Error boundary

Exact resolver stages use fallible metadata and directory enumeration. `NotFound` remains a candidate miss where fallback applies; permission, invalid path, and other I/O failures surface as `ResolveError` diagnostics. Configured roots that are unavailable can be skipped independently so one optional search root does not disable unrelated resolution.

Completion and menu APIs remain infallible and skip unreadable or disappearing entries. This preserves their interactive no-output contract.

Alternative: make every completion API return `Result`. Rejected because menu and shell completion cannot usefully recover from per-entry failures and would change established empty-result behavior.

### Identity and text boundaries

Deduplication, ordering tie-breaks, and selection retain `PathBuf`/`OsStr` values. Human labels may be lossy, but duplicate labels must still select their corresponding path by index. Query and selector values are not trimmed; only a genuinely empty string is absent.

Plain, JSON, and shell action outputs remain UTF-8 string interfaces. If a selected path cannot be represented safely at that boundary, the operation reports an error or returns noop rather than emitting a lossy replacement.

### Native CI matrix

Run `cargo test --locked` on `ubuntu-latest` and `windows-latest`. Gate POSIX-shell integration tests on Unix and retain the PowerShell smoke job on native Windows. Platform-neutral tests derive paths from temp directories; Unix and Windows lexical-root tests use `cfg` gates.

Alternative: cross-compile Windows tests on Linux. Rejected because compilation cannot validate Windows path semantics, filesystem behavior, or PowerShell integration.

## Risks / Trade-offs

- [Risk] Shared parsing can change established Unix behavior. → Mitigation: retain Unix-specific tests, explicitly test backslash-as-filename behavior, and migrate callers incrementally.
- [Risk] Permission-error behavior can expose failures previously treated as misses. → Mitigation: apply propagation only to exact resolver operations and keep optional roots/completion best-effort.
- [Risk] UNC shares are unavailable in hosted CI. → Mitigation: test Windows UNC parsing and normalization lexically; do not require live share traversal.
- [Risk] Non-UTF paths cannot be sent through current JSON or shell string contracts. → Mitigation: preserve them internally and fail safely at the explicit text boundary.

## Migration Plan

1. Add the shared classifier and lexical normalization tests.
2. Migrate resolver direct resolution, fallback anchors, abbreviation parsing, and exact I/O errors.
3. Migrate completion filtering, labels, mapped menu sourcing, and shell replacement formatting.
4. Convert brittle Unix fixtures and enable the native Windows core-test job.
5. Run full Unix and Windows CI before release. Rollback is a normal code revert; no stored path data format changes are introduced.

## Open Questions

None. The Windows TUI and non-UTF external encoding are explicitly deferred.
