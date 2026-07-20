# Menu Demo Recording

This directory contains the reproducible source for the animated menu demo.

## Files

- `setup.zsh` creates an isolated directory fixture and loads menu-enabled Zsh
  hooks.
- `menu-demo.tape` records a large scrollable two-column path menu, filtering,
  abbreviated `cd` commands, and menu-driven `cd-` / `cd+` traversal with VHS.
- The generated GIF is written to `docs/assets/menu-demo.gif`.

The fixture uses `${DX_DEMO_ROOT:-${TMPDIR:-/tmp}/dx-menu-demo}` and stores
session data under a separate `dx-menu-demo-state` directory. Both locations
are deleted and rebuilt for each recording. Set `DX_DEMO_ROOT` to a path ending
in `dx-menu-demo` when the generated animation should use a stable display path.

## Generate

Install [VHS](https://github.com/charmbracelet/vhs), build an optimized `dx`,
then run from the repository root:

```bash
cargo build --release
DX_DEMO_BIN=target/release/dx vhs docs/demo/menu-demo.tape
```

If `dx` is already installed on `PATH`, omit `DX_DEMO_BIN`:

```bash
vhs docs/demo/menu-demo.tape
```

Nix can run VHS without a global installation:

```bash
DX_DEMO_BIN=target/release/dx nix run nixpkgs#vhs -- docs/demo/menu-demo.tape
```

On macOS, `/Users/Shared` provides a stable path without exposing a username or
the `/private/tmp` alias in the recording:

```bash
DX_DEMO_BIN=target/release/dx \
  DX_DEMO_ROOT=/Users/Shared/dx-menu-demo \
  nix run nixpkgs#vhs -- docs/demo/menu-demo.tape
```

Optionally optimize the generated file with gifsicle:

```bash
gifsicle --batch -O3 docs/assets/menu-demo.gif
```

After generation, embed it with:

```markdown
![dx interactive menu filtering and selecting a directory](docs/assets/menu-demo.gif)
```
