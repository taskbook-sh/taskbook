#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/release.sh <version> [remote]

Updates overlay.nix version, commits, tags, and pushes.

Examples:
  scripts/release.sh 0.1.2
  scripts/release.sh 0.1.2 upstream
EOF
}

if [[ $# -lt 1 || $# -gt 2 ]]; then
  usage
  exit 1
fi

version="$1"
remote="${2:-origin}"
tag="v${version}"

if [[ -n "$(git status --porcelain)" ]]; then
  echo "Working tree is dirty; please commit or stash changes first." >&2
  exit 1
fi

if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
  echo "Tag ${tag} already exists." >&2
  exit 1
fi

perl -0pi -e "s/version = \"[^\"]+\";/version = \"${version}\";/" overlay.nix

if git diff --quiet -- overlay.nix; then
  echo "overlay.nix already at version ${version}." >&2
  exit 1
fi

git add overlay.nix
git commit -m "Release ${tag}"
git tag "${tag}"

git push "${remote}" HEAD
git push "${remote}" "${tag}"
