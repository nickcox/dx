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

## Technical Docs

- [Technical docs index](./tech-docs/)
- [Configuration reference](./tech-docs/configuration.md)
- [Shell hook guarding strategy](./tech-docs/shell-hook-guarding.md)
