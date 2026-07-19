## Why

The generated PowerShell `Set-DxLocation` wrapper uses POSIX-style argument parsing that breaks native PowerShell parameter binding, and its custom `cd -` state is overwritten before use. Now that the wrapper has a stable module boundary and idiomatic command name, it can preserve dx navigation while behaving like a native `Set-Location` command.

## What Changes

- Replace remaining-argument parsing with a PowerShell advanced function that supports the native `Set-Location` parameter sets and pipeline behavior.
- Delegate no-argument home navigation, `-` and `+` location history, literal paths, named stacks, provider paths, and native output/error behavior to `Set-Location`.
- Apply `dx resolve` only to eligible filesystem `Path` arguments while preserving native fallback behavior.
- Record successful filesystem location transitions in the dx session stack without storing non-filesystem provider locations.
- Add runtime PowerShell integration coverage for parameter binding, history navigation, provider handling, pipeline input, output, failures, and stack updates.
- Clarify that POSIX `cd` flag passthrough requirements do not apply to PowerShell.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `shell-hooks`: Define native-compatible PowerShell location-wrapper behavior, filesystem-only dx augmentation, and shell-specific flag semantics.

## Impact

- Affects generated PowerShell hook code in `src/hooks/pwsh.rs` and hook-generation tests.
- Adds PowerShell runtime integration tests where `pwsh` is available.
- Changes the exported `Set-DxLocation` parameter contract from unstructured remaining arguments to native-style parameter sets.
- Does not change Bash, Zsh, Fish, the `dx resolve` CLI contract, or session-stack storage format.
