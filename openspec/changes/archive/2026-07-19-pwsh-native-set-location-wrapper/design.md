## Context

`Set-DxLocation` currently accepts all input as remaining string arguments, separates tokens that look like POSIX flags, and forwards an array to `Set-Location`. PowerShell array splatting does not reinterpret strings such as `-LiteralPath` or `-PassThru` as named parameters, so native parameter forms fail. The wrapper also assigns the current path to its saved old-directory variable immediately before handling `cd -`, preventing previous-location navigation.

PowerShell's native `Set-Location` already provides no-argument home navigation, `-` and `+` history, three parameter sets, pipeline input, provider paths, wildcards, named location stacks, and `PassThru`. The dx resolver and session stack are filesystem-oriented, so native compatibility requires a clear boundary between native location behavior and dx augmentation.

## Goals / Non-Goals

**Goals:**

- Make `Set-DxLocation` bind and forward the current native `Set-Location` parameter contract.
- Preserve dx resolution for direct, eligible filesystem path arguments.
- Preserve native output, error, pipeline, history, wildcard, and provider behavior.
- Keep non-filesystem provider locations out of dx session-stack storage.
- Add runtime tests that exercise generated hooks in `pwsh`, rather than relying only on generated-text assertions.

**Non-Goals:**

- Adding cd-extras features such as path-part replacement, numbered undo/redo spellings, or configurable no-argument destinations.
- Replacing dx undo/redo with PowerShell's native location history or named stacks.
- Teaching `dx resolve` or dx session stacks about non-filesystem PowerShell providers.
- Changing Bash, Zsh, or Fish wrapper behavior beyond clarifying the shell-specific flag requirement.
- Supporting Windows PowerShell-specific transaction parameters that are absent from current `pwsh` `Set-Location` metadata.

## Decisions

1. Model `Set-DxLocation` as a native-style advanced function.

The wrapper will declare `CmdletBinding` and the native `Path`, `LiteralPath`, and `StackName` parameter sets, including `PassThru`, `PSPath`/`LP` aliases, pipeline attributes, and property-name binding. It will resolve the fully qualified `Microsoft.PowerShell.Management\Set-Location` cmdlet and forward bound parameters through a steppable pipeline. This follows the useful proxy-command structure in cd-extras while retaining dx-specific policy around resolution and stack recording.

Alternative considered: continue parsing remaining string arguments and special-case known parameter names. This would reproduce PowerShell's binder incompletely, mishandle common parameters and pipeline input, and drift as native behavior evolves.

2. Delegate native location semantics instead of maintaining parallel state.

No-argument calls and the `-` and `+` path values will be forwarded to native `Set-Location`. The module-scoped `__dx_oldpwd` state will be removed. This makes location history include changes performed through other aliases or direct `Set-Location` calls and eliminates the current overwrite bug.

Alternative considered: repair `__dx_oldpwd`. A private one-entry history would still diverge from PowerShell's native bidirectional history and would not observe location changes outside `Set-DxLocation`.

3. Limit dx resolution to direct filesystem `Path` augmentation.

The wrapper will consider dx resolution only for a directly bound `Path` value when the starting provider is FileSystem. It will bypass resolution for pipeline input, `LiteralPath`, `StackName`, `-` and `+`, wildcard paths, and provider-qualified paths. A successful dx result replaces the bound `Path`; a failed result leaves the original value for native fallback. Resolver diagnostics remain suppressed during fallback, matching current wrapper behavior.

Alternative considered: attempt dx resolution for every `Path`. That could reinterpret provider-relative paths, wildcard expressions, or native history tokens as filesystem queries and violate native semantics.

4. Forward through the native cmdlet with a steppable pipeline.

The wrapper will use the proxy-command pattern of resolving the cmdlet by module-qualified name, constructing a script command with `@PSBoundParameters`, and forwarding `Begin`, `Process`, and `End`. This preserves pipeline behavior, `PassThru` output, common parameters, and native diagnostics while avoiding recursion through the `cd` alias.

Alternative considered: call the cmdlet directly from a simple `process` block. That is shorter but is less faithful to native pipeline lifecycle and output/error behavior.

5. Record only completed filesystem transitions.

The wrapper will capture the starting `PathInfo`, complete native forwarding, and compare the final provider and path. If the destination changed and is FileSystem, it will push the starting path first when the start was also FileSystem, then push the destination. This initializes an empty dx session with both sides of the first transition. If the start was another provider, only the filesystem destination is pushed. A non-filesystem destination and a call such as `-StackName` that does not change location are not recorded.

Stack commands will have all output suppressed and remain fire-and-forget. Their failures will not replace native output or turn a successful `Set-Location` into failure.

Alternative considered: keep pushing before native navigation. That initializes history but mutates the dx stack even when parameter binding or navigation fails. A single atomic transition operation in the Rust CLI would avoid two pushes, but it expands this change into a new stack API without being necessary for correctness.

6. Verify behavior in a real PowerShell process.

Generated-text tests will continue to cover stable structural markers, while integration tests will evaluate `dx init pwsh` as one script block in `pwsh` and exercise parameter metadata and behavior. Tests that mutate `DX_SESSION` or session storage environment variables will use the shared global environment lock, and temporary paths will be canonicalized on macOS before comparison.

Alternative considered: add only string assertions for parameter declarations. Those assertions cannot detect binding, pipeline, provider, output, or history regressions.

## Risks / Trade-offs

- Static proxy parameters can drift from a future `Set-Location` contract -> Match the supported `pwsh` contract explicitly and cover parameter metadata in runtime tests.
- Pipeline input may visit multiple locations while stack recording sees only the initial and final locations -> Treat one pipeline invocation as one effective transition, matching the start/end tracking model used by cd-extras.
- Entering a non-filesystem provider leaves the last filesystem location as dx's tracked cwd -> Skip unsupported provider paths and resynchronize when a filesystem destination is reached.
- Two fire-and-forget pushes can partially update history if storage fails between them -> Preserve successful navigation and defer an atomic transition CLI operation unless partial writes prove problematic.
- Provider availability differs by platform -> Use a cross-platform provider such as `Env:` where supported and isolate Windows-only provider cases.

## Migration Plan

The generated hook changes the next time users evaluate `dx init pwsh`; no persisted-data migration is required. Re-evaluating older generated output or installing the previous binary restores the former wrapper behavior if rollback is needed.

## Open Questions

None.
