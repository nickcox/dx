## Why

Shell hooks (`dx init`) now handle `cd` wrapping and session recording, but there is no completion contract for the different navigation intents users actually invoke (`cd`, `up`, `z`/`cdf`, `cdr`, `cd-`, `cd+`). A single inferred completion mode would be ambiguous and brittle. We need explicit completion modes so shell hooks can request the right candidate set for each action.

## What Changes

- Replace the generic `dx complete <query>` shape with mode-specific completion subcommands:
  - `dx complete paths <query>`
  - `dx complete ancestors [query]`
  - `dx complete frecents [query]`
  - `dx complete recents [query]`
  - `dx complete stack --direction back|forward [query]`
- Define shell-to-mode dispatch explicitly (no intent inference):
  - `cd` -> `paths`
  - `up` -> `ancestors`
  - `z`/`cdf` -> `frecents`
  - `cdr` -> `recents`
  - `cd-`/`cd+` -> `stack` with `back`/`forward`
- Introduce a `FrecencyProvider` trait with a `ZoxideProvider` implementation for frecents-mode candidates.
- Implement mode-aware candidate sourcing and ranking, with per-mode providers and deduplication rules.
- Add shell completion registration code to `dx init` output for all four shells:
  - Bash: function-based completion bindings that route each action to the correct `dx complete <mode>` call.
  - Zsh: `compdef` wrappers that route by command/alias to mode-specific completion calls.
  - Fish: `complete` entries per command/alias mapped to mode-specific completion calls.
  - PowerShell: `Register-ArgumentCompleter` handlers mapped to mode-specific completion calls.
- The completion bindings are always included in `dx init` output (not gated behind a flag).

## Capabilities

### New Capabilities
- `completions`: Mode-specific completion subcommands, mode-aware candidate sourcing/ranking, frecency provider abstraction, and shell completion dispatch wiring.

### Modified Capabilities
- `shell-hooks`: `dx init` output gains shell-specific completion registration blocks that map each navigation action to the matching completion mode.

## Impact

- **CLI surface**: New `dx complete` mode tree with shared `--json` support.
- **New modules**: `src/complete/mod.rs` (mode routing, candidate shaping, dedup), `src/frecency/mod.rs` (`FrecencyProvider` trait + `ZoxideProvider`), `src/cli/complete.rs`.
- **Modified modules**: `src/cli/mod.rs` (new `Complete` variant), `src/hooks/{bash,zsh,fish,pwsh}.rs` (mode-aware completion registration and command/alias dispatch).
- **External dependency**: Optional runtime dependency on `zoxide` binary for frecents mode. `dx complete frecents` degrades gracefully if `zoxide` is not installed.
- **No new crate dependencies**: `zoxide` is invoked via `std::process::Command`, not linked.
