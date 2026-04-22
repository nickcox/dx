# Shell Setup

Use this page to enable `dx` in your shell and verify it is active.

## Before you start

- Make sure `dx` is installed and available on your `PATH`.
- If you use Nix, you can build or run `dx` before shell setup with `nix build .#cdex` or `nix run .#dx -- --help`.
- If you use Homebrew, install from the separate tap with `brew tap nickcox/dx` and `brew install nickcox/dx/cdex` before shell setup.
- Homebrew installs the `cdex` formula, but the executable you run in your shell is still `dx`.
- Choose the setup instructions for your shell.

## Bash

Add `dx` init output to your Bash startup config, then restart your terminal or reload the file.

## Zsh

Add `dx` init output to your Zsh startup config, then restart your terminal or reload the file.

## Fish

Add `dx` init output to your Fish config, then restart your terminal or reload the file.

## PowerShell

Add `dx` init output to your PowerShell profile, then restart your terminal or reload the profile.

## Verify setup

Run:

```bash
dx --help
```

Success looks like: help output appears and no shell errors occur during startup.

## Related docs

- Project overview: [README](../README.md)
- Start here: [Quickstart](./quickstart.md)
- Implementation details: [Technical Docs](../tech-docs/)
