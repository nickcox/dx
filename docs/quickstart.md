# Quickstart

This guide gets `dx` installed, loaded into your shell, and completing a first
navigation workflow.

## 1. Install dx

Homebrew:

```bash
brew install nickcox/dx/dx
```

Nix, from the repository:

```bash
nix build .#dx
nix run .#dx -- --help
```

The crate is published as `cdex` because `dx` was taken on crates.io; the binary
it installs is `dx`. `.#cdex` and `.#dx` build the same thing.

Tagged GitHub Releases also provide raw binaries for Linux, macOS, and Windows.
Place the downloaded binary on your `PATH` and make it executable when needed.

Confirm the command is available:

```bash
dx --help
```

## 2. Load the shell integration

Choose your shell and add the command to its profile.

### Bash

Add to `~/.bashrc`:

```bash
eval "$(dx init bash)"
```

### Zsh

Add to `~/.zshrc`:

```zsh
eval "$(dx init zsh)"
```

### Fish

Add to `~/.config/fish/config.fish`:

```fish
dx init fish | source
```

### PowerShell

Add to the profile shown by `$PROFILE`:

```powershell
Invoke-Expression ((& dx init pwsh | Out-String))
```

PowerShell must evaluate the generated output as one script block, which is why
the command uses `Out-String`.

On Windows, use `--native-menu` if you want an interactive completion menu; the
Rust TUI provided by `--menu` is Unix-only:

```powershell
Invoke-Expression ((& dx init pwsh --native-menu | Out-String))
```

Restart the shell or reload the profile after making the change.

## 3. Try abbreviated paths

`dx` first respects direct paths, then applies its path-shortening rules when a
literal path does not resolve.

```text
cd pr/dx
cd cd-e
cd P..Shell
```

The matching rules are:

- Plain fragments match from the start of a path segment.
- `.`, `_`, and `-` identify literal word boundaries within a segment.
- `..` inside a segment is a gap, so `P..Shell` can match `PowerShell`.
- A pure multi-dot token such as `...` keeps its step-up meaning.

Matching is case-sensitive by default. Set `DX_CASE_SENSITIVE=false` if you
prefer lowercase queries such as `p..shell` for mixed-case names.

Matches must be unambiguous. Use normal literal paths whenever you want to skip
abbreviation matching.

## 4. Bookmark a directory

Save a name for the current directory:

```bash
dx bookmarks add work
```

`dx` prints the absolute path it saved. Afterwards `cd work` jumps there, and
`cd wo` plus the completion key offers it alongside any matching directories:

```text
cd work
dx bookmarks
dx bookmarks remove work
```

## 5. Navigate history and ancestors

After changing directories a few times, try:

```text
up
back
forward
```

Aliases are also available:

```text
cd-     # same as back
cd+     # same as forward
```

Navigation commands accept an optional selector:

```text
up 3          # third ancestor
up project    # closest matching ancestor
back 2        # second item in back history
```

## 6. Optional frecent navigation

`z` and `cdf` use zoxide as their frecency provider:

```text
z project
cdf project
```

Install and use zoxide normally to populate its database. If zoxide is not
available, frecent queries simply produce no candidates.

## 7. Optional interactive menu

Add `--menu` to shell initialization to replace supported completion bindings
with the inline selector:

```zsh
eval "$(dx init zsh --menu)"
```

Equivalent forms are available for every supported shell. See
[Interactive Menu](./menu.md) for controls and customization.

## Next Steps

- [Shell Setup](./shell-setup.md)
- [Navigation Guide](./navigation.md)
- [Interactive Menu](./menu.md)
- [Configuration Reference](./configuration.md)
- [Scripting](./scripting.md)
- [Troubleshooting](./troubleshooting.md)
