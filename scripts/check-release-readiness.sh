#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo_toml="$repo_root/Cargo.toml"
flake_nix="$repo_root/flake.nix"

if [[ ! -f "$cargo_toml" ]]; then
  echo "missing Cargo.toml" >&2
  exit 1
fi

if [[ ! -f "$flake_nix" ]]; then
  echo "missing flake.nix" >&2
  exit 1
fi

cargo_version="$(python3 - "$cargo_toml" <<'PY'
import pathlib, re, sys
text = pathlib.Path(sys.argv[1]).read_text()
match = re.search(r'^version\s*=\s*"([^"]+)"', text, re.MULTILINE)
if not match:
    raise SystemExit("could not find version in Cargo.toml")
print(match.group(1))
PY
)"

flake_version="$(python3 - "$flake_nix" <<'PY'
import pathlib, re, sys
text = pathlib.Path(sys.argv[1]).read_text()
match = re.search(r'version\s*=\s*"([^"]+)";', text)
if not match:
    raise SystemExit("could not find version in flake.nix")
print(match.group(1))
PY
)"

if [[ "$cargo_version" != "$flake_version" ]]; then
  echo "version mismatch: Cargo.toml=$cargo_version flake.nix=$flake_version" >&2
  exit 1
fi

echo "release readiness checks passed"
