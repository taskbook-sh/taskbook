#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

usage() {
  cat <<'EOF'
Usage: scripts/release.sh [options] <version> [remote]

Bumps versions, commits to master, tags the release, and pushes to trigger
the GitHub Actions release workflow.

Options:
  --dry-run     Perform changes locally but do NOT push or create a tag.

Arguments:
  version       Semver version without 'v' prefix (e.g. 1.0.4)
  remote        Git remote to push to (default: origin)

Examples:
  scripts/release.sh 1.0.6              # Full release
  scripts/release.sh --dry-run 1.0.6    # Preview locally
EOF
}

# --------------- helper functions ---------------
run_sed() {
  local pattern="$1"
  local file="$2"
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "$pattern" "$file"
  else
    sed -i "$pattern" "$file"
  fi
}

# --------------- parse flags ---------------
DRY_RUN=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=true; shift ;;
    -h|--help) usage; exit 0 ;;
    -*)        echo "Unknown option: $1" >&2; usage; exit 1 ;;
    *)         break ;;
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

echo "Detecting default branch..."
TARGET_BRANCH=$(git -C "$REPO_ROOT" remote show "${REMOTE}" | grep 'HEAD branch' | cut -d' ' -f5)
TARGET_BRANCH="${TARGET_BRANCH:-master}"
echo "  Default branch is '${TARGET_BRANCH}'"

CURRENT_BRANCH=$(git -C "$REPO_ROOT" rev-parse --abbrev-ref HEAD)
if [[ "$CURRENT_BRANCH" != "$TARGET_BRANCH" ]]; then
  echo "Error: must be on $TARGET_BRANCH branch (currently on '${CURRENT_BRANCH}')." >&2
  exit 1
fi

# --------------- pull latest ---------------
if [[ "$DRY_RUN" == true ]]; then
  echo "Dry-run: skipping pull."
else
  echo "Pulling latest $TARGET_BRANCH ..."
  git -C "$REPO_ROOT" pull "${REMOTE}" "$TARGET_BRANCH"
fi

# --------------- update versions ---------------
echo "Bumping versions to ${VERSION} ..."

for toml in \
  "$REPO_ROOT/crates/taskbook-common/Cargo.toml" \
  "$REPO_ROOT/crates/taskbook-client/Cargo.toml" \
  "$REPO_ROOT/crates/taskbook-server/Cargo.toml"; do
  run_sed "s/^version = \"[^\"]*\"/version = \"${VERSION}\"/" "$toml"
  echo "  updated $(basename "$(dirname "$toml")")/Cargo.toml"
done

run_sed "s/version = \"[^\"]*\";/version = \"${VERSION}\";/" "$REPO_ROOT/overlay.nix"
echo "  updated overlay.nix"

# k8s/deployment.yaml (gitignored — update for local use)
DEPLOYMENT="$REPO_ROOT/k8s/deployment.yaml"
if [[ -f "$DEPLOYMENT" ]]; then
  run_sed "s|image: ghcr.io/taskbook-sh/taskbook-server:[^ ]*|image: ghcr.io/taskbook-sh/taskbook-server:${VERSION}|" "$DEPLOYMENT"
  echo "  updated k8s/deployment.yaml (local only, gitignored)"
fi

# --------------- verify build ---------------
echo "Updating Cargo.lock and verifying build ..."
(cd "$REPO_ROOT" && cargo metadata --format-version 1 >/dev/null)
(cd "$REPO_ROOT" && cargo check --workspace)

# --------------- commit and tag ---------------
if [[ "$DRY_RUN" == true ]]; then
  echo "Dry-run: skipping commit and tag."
  
  # Clean up changes to Cargo.lock, Cargo.toml, overlay.nix
  echo "Dry-run: reverting changes..."
  git -C "$REPO_ROOT" checkout \
    crates/taskbook-common/Cargo.toml \
    crates/taskbook-client/Cargo.toml \
    crates/taskbook-server/Cargo.toml \
    overlay.nix \
    Cargo.lock
    
  echo "Dry-run completed successfully. No changes were applied."
  exit 0
fi

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
git -C "$REPO_ROOT" push "${REMOTE}" "${TARGET_BRANCH}" "${TAG}"
echo "Pushed ${TARGET_BRANCH} and tag ${TAG} to ${REMOTE}. Release workflow should start shortly."

# --------------- update overlay.nix hashes ---------------
echo ""
echo "Waiting for release workflow to start ..."
RUN_ID=""
for _ in $(seq 1 30); do
  RUN_ID=$(gh run list --workflow release.yml --json databaseId,headBranch \
    -q "[.[] | select(.headBranch == \"${TAG}\")][0].databaseId" --limit 10 2>/dev/null)
  if [[ -n "$RUN_ID" ]]; then
    break
  fi
  sleep 5
done

if [[ -z "$RUN_ID" ]]; then
  echo "Warning: could not find release workflow run. Skipping overlay hash update." >&2
  echo "Update overlay.nix hashes manually after the release is published."
  exit 0
fi

echo "Watching workflow run ${RUN_ID} ..."
if ! gh run watch "$RUN_ID" --exit-status; then
  echo "Error: release workflow failed." >&2
  exit 1
fi

echo "Downloading release assets and updating overlay.nix hashes ..."
HASH_TMPDIR=$(mktemp -d)
trap "rm -rf $HASH_TMPDIR" EXIT

TARBALLS=(
  "tb-linux-x86_64.tar.gz"
  "tb-linux-aarch64.tar.gz"
  "tb-darwin-x86_64.tar.gz"
  "tb-darwin-aarch64.tar.gz"
)

for tarball in "${TARBALLS[@]}"; do
  gh release download "${TAG}" --pattern "${tarball}" --dir "$HASH_TMPDIR"
  hash="sha256-$(openssl dgst -sha256 -binary "${HASH_TMPDIR}/${tarball}" | base64 | tr -d '\n')"
  
  # Update hash in overlay.nix using a temp file
  awk -v name="${tarball}" -v newhash="${hash}" '
    $0 ~ name { found=1 }
    found && /hash = "/ {
      sub(/hash = "[^"]*"/, "hash = \"" newhash "\"")
      found=0
    }
    { print }
  ' "$REPO_ROOT/overlay.nix" > "$REPO_ROOT/overlay.nix.tmp"
  mv "$REPO_ROOT/overlay.nix.tmp" "$REPO_ROOT/overlay.nix"
  echo "  ${tarball}: ${hash}"
done

git -C "$REPO_ROOT" add overlay.nix
git -C "$REPO_ROOT" commit -m "Update overlay.nix hashes for ${TAG}"
git -C "$REPO_ROOT" push "${REMOTE}" "${TARGET_BRANCH}"
echo "Pushed updated overlay.nix hashes to ${TARGET_BRANCH}."
