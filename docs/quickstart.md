# Quickstart

Welcome! This guide helps you get `dx` working quickly and run your first successful navigation flow.

## What you'll do

1. Confirm prerequisites.
2. Initialize shell integration.
3. Run a first command and verify expected output.
4. Continue to deeper guides as needed.

## Prerequisites

- A supported shell (Bash, Zsh, Fish, or PowerShell).
- `dx` installed and available on your `PATH`.
- If you use Nix, you can build or run the packaged app with:

```bash
nix build .#cdex
nix run .#dx -- --help
```

- If you use Homebrew, install from the separate tap with:

```bash
brew tap nickcox/dx
brew install nickcox/dx/cdex
dx --help
```

- Homebrew installs the `cdex` formula, but the command you run remains `dx`.

- If you prefer direct downloads, tagged GitHub Releases also publish raw binaries for Linux x86_64, Linux ARM64, macOS Intel, macOS Apple Silicon, and Windows x86_64. Download the binary for your platform, make it executable if needed, and place it on your `PATH`.

- A terminal session where you can run shell init commands.

## 1) Initialize shell integration

Set up your shell first so wrappers and completion behavior work as expected.

- Go to: [Shell Setup](./shell-setup.md)

## 2) Try a first command

After shell setup is loaded, run a simple command to confirm `dx` is available:

```bash
dx --help
```

Success looks like: help text prints without errors.

## 3) Use basic navigation flow

Once your shell is initialized, try one normal navigation command from your shell workflow.

`dx` path resolution supports shortened path segments, not just literal directory names. Examples:

```bash
cd pr/dx
cd cd-e
cd p..shell
cd proj/p..shell/s/.sdk
```

Shortening rules:
- Plain fragments still match from the start of a segment.
- Word delimiters `.`, `_`, and `-` can be used inside a segment, so queries like `cd-e` and `.sdk` match names around those literal delimiters.
- Doubled periods `..` act as an in-segment gap operator, so `p..shell` can match `PowerShell` and `s..32` can match `System32`.

If a command fails, revisit shell setup and confirm your shell initialization loaded successfully.

## Optional: enable menu completion for external commands

If you initialize with `dx init <shell> --menu`, you can map external commands to `dx`'s menu completion with `DX_MENU_COMMAND_MAPPINGS`:

```sh
export DX_MENU_COMMAND_MAPPINGS="ls=path,open=path,cat=file"
eval "$(dx init zsh --menu)"
```

The mapping format is `<command>=<mode>,...`. Valid modes are `path` for files and directories, `directory` for directories only, and `file` for regular files only. Re-run `dx init <shell> --menu` after changing mappings so regenerated hooks capture the new command set.

In PowerShell, dx menu mode binds `Tab` by default. Set `DX_PWSH_MENU_KEY` before `dx init pwsh --menu` to use a different PSReadLine key, for example `F12`. PowerShell fallback attempts to preserve the key's previous PSReadLine function; previous custom scriptblock handlers produce a warning because they cannot be replayed.

## What to read next

- Project overview: [README](../README.md)
- Setup details: [Shell Setup](./shell-setup.md)
- Implementation details: [Technical Docs](../tech-docs/)
