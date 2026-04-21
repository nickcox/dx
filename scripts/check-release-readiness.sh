#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo_toml="$repo_root/Cargo.toml"
flake_nix="$repo_root/flake.nix"
release_workflow="$repo_root/.github/workflows/release.yml"

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

if ! grep -Eq '^name:\s*release$' "$release_workflow"; then
  echo "release workflow missing expected name: release" >&2
  exit 1
fi

if ! grep -Eq '^\s*tags:\s*$' "$release_workflow"; then
  echo "release workflow missing tag trigger section" >&2
  exit 1
fi

if ! grep -Eq '^\s*-\s*"v\*"\s*$' "$release_workflow"; then
  echo "release workflow missing expected tag pattern: \"v*\"" >&2
  exit 1
fi

if ! grep -Eq '^\s*build-release-binaries:\s*$' "$release_workflow"; then
  echo "release workflow missing build-release-binaries job" >&2
  exit 1
fi

if ! grep -Eq '^\s*publish-release:\s*$' "$release_workflow"; then
  echo "release workflow missing publish-release job" >&2
  exit 1
fi

if ! grep -Eq 'dx-linux-x86_64' "$release_workflow"; then
  echo "release workflow missing linux raw binary asset marker" >&2
  exit 1
fi

if ! grep -Eq 'dx-linux-arm64' "$release_workflow"; then
  echo "release workflow missing linux arm64 raw binary asset marker" >&2
  exit 1
fi

if ! grep -Eq 'dx-macos-x86_64' "$release_workflow"; then
  echo "release workflow missing macos raw binary asset marker" >&2
  exit 1
fi

if ! grep -Eq 'dx-macos-arm64' "$release_workflow"; then
  echo "release workflow missing macos arm64 raw binary asset marker" >&2
  exit 1
fi

if ! grep -Eq 'dx-windows-x86_64\.exe' "$release_workflow"; then
  echo "release workflow missing windows raw binary asset marker" >&2
  exit 1
fi

echo "release readiness checks passed"
