#!/usr/bin/env bash

# Reads and rewrites CHANGELOG.md around the `## [Unreleased]` section.
#
# Split out from release.sh so that script stays about the files carrying the
# crate version. The two meet in release.sh, which calls `check-unreleased`
# before rewriting anything and `release` once the version files are updated.
#
# Parsing lives in python3 for the same reason release.sh uses it: the format is
# line-anchored markdown and sed portability across BSD and GNU is not worth the
# fight.

set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage:
  changelog.sh check-unreleased      fail when the Unreleased section is empty
  changelog.sh release <version> <date>
                                     retitle Unreleased as <version> - <date>
                                     and open a fresh Unreleased section
  changelog.sh extract <version>     print one version's notes to stdout
USAGE
  exit 2
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
changelog_md="$repo_root/CHANGELOG.md"

require_changelog() {
  if [[ ! -f "$changelog_md" ]]; then
    echo "changelog.sh: $changelog_md does not exist" >&2
    exit 1
  fi
}

# Empty means no entries, not merely no text: a section holding nothing but
# `### Added` headings has still recorded nothing.
check_unreleased() {
  python3 - "$changelog_md" <<'PY'
import pathlib, re, sys

text = pathlib.Path(sys.argv[1]).read_text()
match = re.search(r"^## \[Unreleased\]\s*$(.*?)(?=^## |\Z)", text, re.MULTILINE | re.DOTALL)
if not match:
    raise SystemExit("changelog.sh: no '## [Unreleased]' section in CHANGELOG.md")

body = [line for line in match.group(1).splitlines() if line.strip()]
if not any(not line.lstrip().startswith("###") for line in body):
    raise SystemExit(
        "changelog.sh: the Unreleased section has no entries.\n"
        "Add them, or pass --allow-empty-changelog to release.sh."
    )
PY
}

release() {
  local version="$1" date="$2"
  python3 - "$changelog_md" "$version" "$date" <<'PY'
import pathlib, re, sys

path, version, date = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3]
text = path.read_text()

if re.search(r"^## \[%s\]" % re.escape(version), text, re.MULTILINE):
    raise SystemExit("changelog.sh: CHANGELOG.md already has a %s section" % version)

# Matches only spaces and tabs, never the newline, so the blank line that
# follows the heading survives into the new section.
updated, count = re.subn(
    r"^## \[Unreleased\][ \t]*$",
    "## [Unreleased]\n\n## [%s] - %s" % (version, date),
    text,
    count=1,
    flags=re.MULTILINE,
)
if count != 1:
    raise SystemExit("changelog.sh: no '## [Unreleased]' section in CHANGELOG.md")
path.write_text(updated)
PY
}

extract() {
  local version="$1"
  python3 - "$changelog_md" "$version" <<'PY'
import pathlib, re, sys

path, version = pathlib.Path(sys.argv[1]), sys.argv[2]
text = path.read_text()

match = re.search(
    r"^## \[%s\][^\n]*$(.*?)(?=^## |\Z)" % re.escape(version),
    text,
    re.MULTILINE | re.DOTALL,
)
if not match:
    raise SystemExit("changelog.sh: CHANGELOG.md has no section for %s" % version)

body = match.group(1).strip()
if not body:
    raise SystemExit("changelog.sh: the %s section is empty" % version)
print(body)
PY
}

(($# > 0)) || usage
require_changelog

case "$1" in
  check-unreleased)
    (($# == 1)) || usage
    check_unreleased
    ;;
  release)
    (($# == 3)) || usage
    release "$2" "$3"
    ;;
  extract)
    (($# == 2)) || usage
    extract "$2"
    ;;
  -h | --help) usage ;;
  *) usage ;;
esac
