## 1. Native PowerShell Wrapper

- [x] 1.1 Replace `Set-DxLocation` remaining-argument parsing with native-style `Path`, `LiteralPath`, and `StackName` parameter sets, `PassThru`, aliases, pipeline attributes, and `CmdletBinding`.
- [x] 1.2 Forward bound parameters and pipeline lifecycle to the fully qualified native `Set-Location` cmdlet with a steppable pipeline.
- [x] 1.3 Remove custom old-directory state and delegate no-argument, `-`, `+`, provider, wildcard, output, and error behavior to native `Set-Location`.

## 2. dx Filesystem Augmentation

- [x] 2.1 Add eligibility checks so only direct FileSystem `Path` arguments without history tokens, provider qualification, or wildcards are sent to `dx resolve`.
- [x] 2.2 Preserve the original `Path` for native fallback when dx is unavailable or resolution fails, while suppressing resolver fallback diagnostics.
- [x] 2.3 Record completed FileSystem transitions after native navigation by pushing the eligible origin and destination without storing provider locations.
- [x] 2.4 Keep stack updates fire-and-forget and prevent stack output or failures from changing native `Set-Location` output and success behavior.

## 3. Generated Hook Tests

- [x] 3.1 Update generated-output assertions for advanced-function parameter sets, native cmdlet qualification, steppable forwarding, resolution eligibility, and removal of `__dx_oldpwd` and POSIX-style PowerShell flag parsing.
- [x] 3.2 Add runtime `pwsh` tests for no-argument navigation, native `-`/`+` history, named and literal paths, `PassThru`, pipeline input, provider paths, wildcard fallback, and native failure behavior.
- [x] 3.3 Add runtime `pwsh` tests proving successful FileSystem transitions update dx history, failed transitions do not, provider destinations are excluded, and stack failures do not alter navigation results.
- [x] 3.4 Isolate runtime session variables in the spawned PowerShell process and canonicalize temporary paths before macOS path comparisons.

## 4. Verification

- [x] 4.1 Run PowerShell-focused tests and the complete Rust test suite, then fix any regressions.
- [x] 4.2 Run formatting and lint checks for the final implementation.
