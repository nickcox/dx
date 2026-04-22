# dx

`dx` is a directory navigation CLI and shell integration toolkit focused on fast path resolution, stack-style navigation, and shell-friendly workflows.

## Start Here

- [Quickstart](./docs/quickstart.md)
- [Shell Setup](./docs/shell-setup.md)

Nix users can run `dx` directly from the flake:

```bash
nix build .#cdex
nix run .#dx -- --help
```

Homebrew users can install from the separate tap:

```bash
brew tap nickcox/dx
brew install nickcox/dx/cdex
dx --help
```

Upgrade with:

```bash
brew upgrade nickcox/dx/cdex
```

Distribution identity is `cdex` (package/formula), while the installed command remains `dx`. Homebrew release updates are maintained via manual PRs to `nickcox/homebrew-dx`.

## Direct Downloads

Tagged GitHub Releases also publish raw `dx` binaries for:

- Linux x86_64: `dx-linux-x86_64`
- Linux ARM64: `dx-linux-arm64`
- macOS Intel: `dx-macos-x86_64`
- macOS Apple Silicon: `dx-macos-arm64`
- Windows x86_64: `dx-windows-x86_64.exe`

Download the binary for your platform from the project's GitHub Releases page, mark it executable if needed, and place it somewhere on your `PATH`.

## Technical Docs

- [Technical docs index](./tech-docs/)
- [Configuration reference](./tech-docs/configuration.md)
- [Shell hook guarding strategy](./tech-docs/shell-hook-guarding.md)
