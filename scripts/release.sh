#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

usage() {
  cat <<'EOF'
Usage: scripts/release.sh [--dry-run] <version> [remote]

Bumps all version strings, commits, tags, and pushes to trigger the
GitHub Actions release workflow.

Options:
  --dry-run   Update files and create commit/tag locally but do NOT push.

Arguments:
  version     Semver version without 'v' prefix (e.g. 1.0.4)
  remote      Git remote to push to (default: origin)

Examples:
  scripts/release.sh 1.0.4
  scripts/release.sh --dry-run 1.0.4
  scripts/release.sh 1.0.4 upstream
EOF
}

# --------------- parse flags ---------------
DRY_RUN=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=true; shift ;;
    -h|--help) usage; exit 0 ;;
    -*) echo "Unknown option: $1" >&2; usage; exit 1 ;;
    *) break ;;
  esac
done

if [[ $# -lt 1 || $# -gt 2 ]]; then
  usage
  exit 1
fi

VERSION="$1"
REMOTE="${2:-origin}"
TAG="v${VERSION}"

# --------------- validate version ---------------
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: version must be semver (e.g. 1.0.4), got '${VERSION}'" >&2
  exit 1
fi

# --------------- check preconditions ---------------
if [[ -n "$(git -C "$REPO_ROOT" status --porcelain)" ]]; then
  echo "Error: working tree is dirty; commit or stash changes first." >&2
  exit 1
fi

if git -C "$REPO_ROOT" rev-parse -q --verify "refs/tags/${TAG}" >/dev/null; then
  echo "Error: tag ${TAG} already exists." >&2
  exit 1
fi

# --------------- update versions ---------------
echo "Bumping versions to ${VERSION} ..."

# Cargo.toml files (match the first `version = "..."` line in each)
for toml in \
  "$REPO_ROOT/crates/taskbook-common/Cargo.toml" \
  "$REPO_ROOT/crates/taskbook-client/Cargo.toml" \
  "$REPO_ROOT/crates/taskbook-server/Cargo.toml"; do
  sed -i '' "s/^version = \"[^\"]*\"/version = \"${VERSION}\"/" "$toml"
  echo "  updated $(basename "$(dirname "$toml")")/Cargo.toml"
done

# overlay.nix
sed -i '' "s/version = \"[^\"]*\";/version = \"${VERSION}\";/" "$REPO_ROOT/overlay.nix"
echo "  updated overlay.nix"

# k8s/deployment.yaml (gitignored — update for local use)
DEPLOYMENT="$REPO_ROOT/k8s/deployment.yaml"
if [[ -f "$DEPLOYMENT" ]]; then
  sed -i '' "s|image: ghcr.io/alexanderdavidsen/taskbook-rs-server:[^ ]*|image: ghcr.io/alexanderdavidsen/taskbook-rs-server:${VERSION}|" "$DEPLOYMENT"
  echo "  updated k8s/deployment.yaml (local only, gitignored)"
fi

# --------------- verify build ---------------
echo "Running cargo check ..."
(cd "$REPO_ROOT" && cargo check --workspace)

# --------------- commit & tag ---------------
git -C "$REPO_ROOT" add \
  crates/taskbook-common/Cargo.toml \
  crates/taskbook-client/Cargo.toml \
  crates/taskbook-server/Cargo.toml \
  overlay.nix \
  Cargo.lock

git -C "$REPO_ROOT" commit -m "Release ${TAG}"
git -C "$REPO_ROOT" tag "${TAG}"

echo "Created commit and tag ${TAG}."

# --------------- push ---------------
if [[ "$DRY_RUN" == true ]]; then
  echo ""
  echo "Dry-run mode: skipping push."
  echo "To finish the release, run:"
  echo "  git push ${REMOTE} HEAD && git push ${REMOTE} ${TAG}"
else
  git -C "$REPO_ROOT" push "${REMOTE}" HEAD
  git -C "$REPO_ROOT" push "${REMOTE}" "${TAG}"
  echo "Pushed to ${REMOTE}. Release workflow should start shortly."
fi
