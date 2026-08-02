#!/usr/bin/env bash

# Verifies that every file carrying the crate version agrees, and optionally
# performs the mechanical half of a release: rewrite those files, run the
# checks CI enforces, then commit and tag.
#
# Pushing is deliberately left out unless asked for. Pushing the tag triggers
# .github/workflows/release.yml, which publishes a GitHub Release and notifies
# the Homebrew tap.

set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage:
  release.sh                verify the versions agree
  release.sh <version>      bump to <version>, verify, commit, tag

options:
  --no-verify   skip cargo fmt, clippy and test before committing
  --allow-empty-changelog
                release even though Unreleased records no entries
  --push        push master and the new tag once the release commit exists
USAGE
  exit 2
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo_toml="$repo_root/Cargo.toml"
cargo_lock="$repo_root/Cargo.lock"
flake_nix="$repo_root/flake.nix"
changelog_md="$repo_root/CHANGELOG.md"
changelog_sh="$repo_root/scripts/changelog.sh"

new_version=""
run_verify=1
do_push=0
require_changelog_entries=1

while (($# > 0)); do
  case "$1" in
    --no-verify) run_verify=0 ;;
    --allow-empty-changelog) require_changelog_entries=0 ;;
    --push) do_push=1 ;;
    -h | --help) usage ;;
    -*)
      echo "unknown option: $1" >&2
      usage
      ;;
    *)
      [[ -n "$new_version" ]] && usage
      new_version="$1"
      ;;
  esac
  shift
done

if [[ $do_push -eq 1 && -z "$new_version" ]]; then
  echo "--push needs a version to release" >&2
  usage
fi

for file in "$cargo_toml" "$cargo_lock" "$flake_nix" "$changelog_md"; do
  if [[ ! -f "$file" ]]; then
    echo "missing ${file#"$repo_root"/}" >&2
    exit 1
  fi
done

# The package version is the only line-anchored `version` in Cargo.toml;
# dependency versions are inline in their own tables.
read_cargo_toml_version() {
  python3 - "$cargo_toml" <<'PY'
import pathlib, re, sys
text = pathlib.Path(sys.argv[1]).read_text()
match = re.search(r'^version\s*=\s*"([^"]+)"', text, re.MULTILINE)
if not match:
    raise SystemExit("could not find version in Cargo.toml")
print(match.group(1))
PY
}

read_flake_version() {
  python3 - "$flake_nix" <<'PY'
import pathlib, re, sys
text = pathlib.Path(sys.argv[1]).read_text()
match = re.search(r'version\s*=\s*"([^"]+)";', text)
if not match:
    raise SystemExit("could not find version in flake.nix")
print(match.group(1))
PY
}

# Reads the version from the lockfile entry for this crate rather than the
# first `version` key, which belongs to whichever package sorts first.
read_cargo_lock_version() {
  python3 - "$cargo_lock" "$1" <<'PY'
import pathlib, re, sys
text = pathlib.Path(sys.argv[1]).read_text()
match = re.search(
    r'^name\s*=\s*"%s"\nversion\s*=\s*"([^"]+)"' % re.escape(sys.argv[2]),
    text,
    re.MULTILINE,
)
if not match:
    raise SystemExit("could not find %s in Cargo.lock" % sys.argv[2])
print(match.group(1))
PY
}

read_crate_name() {
  python3 - "$cargo_toml" <<'PY'
import pathlib, re, sys
text = pathlib.Path(sys.argv[1]).read_text()
match = re.search(r'^name\s*=\s*"([^"]+)"', text, re.MULTILINE)
if not match:
    raise SystemExit("could not find package name in Cargo.toml")
print(match.group(1))
PY
}

# Rewrites the first match only, so a later `version` key cannot be clobbered.
replace_first() {
  python3 - "$1" "$2" "$3" <<'PY'
import pathlib, re, sys
path = pathlib.Path(sys.argv[1])
text = path.read_text()
updated, count = re.subn(sys.argv[2], sys.argv[3], text, count=1, flags=re.MULTILINE)
if count != 1:
    raise SystemExit("could not rewrite version in %s" % path.name)
path.write_text(updated)
PY
}

verify_versions_agree() {
  local crate_name cargo_version flake_version lock_version
  crate_name="$(read_crate_name)"
  cargo_version="$(read_cargo_toml_version)"
  flake_version="$(read_flake_version)"
  lock_version="$(read_cargo_lock_version "$crate_name")"

  local disagreeing=()
  [[ "$flake_version" != "$cargo_version" ]] && disagreeing+=("flake.nix=$flake_version")
  [[ "$lock_version" != "$cargo_version" ]] && disagreeing+=("Cargo.lock=$lock_version")

  if ((${#disagreeing[@]} > 0)); then
    echo "version mismatch: Cargo.toml=$cargo_version ${disagreeing[*]}" >&2
    exit 1
  fi
}

if [[ -z "$new_version" ]]; then
  verify_versions_agree
  # The published release notes come from this section, so a tag whose version
  # has none is a failed release rather than a quiet one.
  "$changelog_sh" extract "$(read_cargo_toml_version)" >/dev/null
  echo "release readiness checks passed"
  exit 0
fi

if [[ ! "$new_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
  echo "not a valid version: $new_version" >&2
  exit 1
fi

tag="v$new_version"
verify_versions_agree
current_version="$(read_cargo_toml_version)"

if [[ "$new_version" == "$current_version" ]]; then
  echo "already at $current_version" >&2
  exit 1
fi

if [[ "$(printf '%s\n%s\n' "$current_version" "$new_version" | sort -V | head -1)" != "$current_version" ]]; then
  echo "$new_version is older than the current $current_version" >&2
  exit 1
fi

if git -C "$repo_root" rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
  echo "tag $tag already exists" >&2
  exit 1
fi

branch="$(git -C "$repo_root" branch --show-current)"
if [[ "$branch" != "master" ]]; then
  echo "releases are cut from master, not $branch" >&2
  exit 1
fi

if ! git -C "$repo_root" diff-index --quiet HEAD -- || [[ -n "$(git -C "$repo_root" ls-files --others --exclude-standard)" ]]; then
  echo "working tree is not clean" >&2
  exit 1
fi

if [[ $require_changelog_entries -eq 1 ]]; then
  "$changelog_sh" check-unreleased
fi

echo "bumping $current_version -> $new_version"
replace_first "$cargo_toml" '^version\s*=\s*"[^"]+"' "version = \"$new_version\""
replace_first "$flake_nix" 'version\s*=\s*"[^"]+";' "version = \"$new_version\";"
"$changelog_sh" release "$new_version" "$(date -u +%Y-%m-%d)"

# Rewrites this crate's entry in the lockfile without touching dependencies.
(cd "$repo_root" && cargo update --workspace --offline --quiet)

verify_versions_agree

if [[ $run_verify -eq 1 ]]; then
  echo "running the checks CI enforces"
  (
    cd "$repo_root"
    cargo fmt --check
    cargo clippy --all-targets --all-features --locked -- -D warnings
    cargo test --locked --quiet
  )
fi

git -C "$repo_root" add -- "$cargo_toml" "$cargo_lock" "$flake_nix" "$changelog_md"
git -C "$repo_root" commit -m "$tag"
git -C "$repo_root" tag -a "$tag" -m "$tag"
echo "committed and tagged $tag"

if [[ $do_push -eq 1 ]]; then
  git -C "$repo_root" push origin master
  git -C "$repo_root" push origin "$tag"
  echo "pushed $tag; the release workflow takes it from here"
else
  cat <<EOF
nothing has been pushed. to publish:
  git push origin master && git push origin $tag
EOF
fi
