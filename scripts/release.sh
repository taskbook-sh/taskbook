#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

usage() {
  cat <<'EOF'
Usage: scripts/release.sh [options] <version> [remote]
       scripts/release.sh --tag <version> [remote]

Creates a release pull request, or tags a merged release.

Commands:
  (default)     Create a release branch, bump versions, and open a PR.
  --tag         After the PR is merged, tag the release on master and push
                the tag to trigger the GitHub Actions release workflow.

Options:
  --dry-run     Perform changes locally but do NOT push or create a PR/tag.

Arguments:
  version       Semver version without 'v' prefix (e.g. 1.0.4)
  remote        Git remote to push to (default: origin)

Examples:
  scripts/release.sh 1.0.6              # Create release PR
  scripts/release.sh --dry-run 1.0.6    # Preview locally
  scripts/release.sh --tag 1.0.6        # Tag after PR is merged
EOF
}

# --------------- parse flags ---------------
DRY_RUN=false
TAG_MODE=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=true; shift ;;
    --tag)     TAG_MODE=true; shift ;;
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
BRANCH="release/${TAG}"

# --------------- validate version ---------------
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: version must be semver (e.g. 1.0.4), got '${VERSION}'" >&2
  exit 1
fi

# ===============================================================
#  --tag mode: tag a merged release on master
# ===============================================================
if [[ "$TAG_MODE" == true ]]; then
  if git -C "$REPO_ROOT" rev-parse -q --verify "refs/tags/${TAG}" >/dev/null; then
    echo "Error: tag ${TAG} already exists." >&2
    exit 1
  fi

  echo "Switching to master and pulling latest changes ..."
  git -C "$REPO_ROOT" checkout master
  git -C "$REPO_ROOT" pull "${REMOTE}" master

  # Sanity check: the version bump should be present on master
  if ! grep -q "^version = \"${VERSION}\"" "$REPO_ROOT/crates/taskbook-client/Cargo.toml"; then
    echo "Error: version ${VERSION} not found in crates/taskbook-client/Cargo.toml." >&2
    echo "Has the release PR been merged?" >&2
    exit 1
  fi

  git -C "$REPO_ROOT" tag "${TAG}"
  echo "Created tag ${TAG}."

  if [[ "$DRY_RUN" == true ]]; then
    echo ""
    echo "Dry-run mode: skipping push."
    echo "To finish: git push ${REMOTE} ${TAG}"
  else
    git -C "$REPO_ROOT" push "${REMOTE}" "${TAG}"
    echo "Pushed tag ${TAG} to ${REMOTE}. Release workflow should start shortly."
  fi

  # Clean up the release branch if it still exists
  if git -C "$REPO_ROOT" rev-parse -q --verify "refs/heads/${BRANCH}" >/dev/null; then
    git -C "$REPO_ROOT" branch -d "${BRANCH}"
    echo "Deleted local branch ${BRANCH}."
  fi
  if git -C "$REPO_ROOT" ls-remote --exit-code --heads "${REMOTE}" "${BRANCH}" >/dev/null 2>&1; then
    if [[ "$DRY_RUN" == false ]]; then
      git -C "$REPO_ROOT" push "${REMOTE}" --delete "${BRANCH}"
      echo "Deleted remote branch ${BRANCH}."
    fi
  fi

  exit 0
fi

# ===============================================================
#  Default mode: create release PR
# ===============================================================

# --------------- check preconditions ---------------
if [[ -n "$(git -C "$REPO_ROOT" status --porcelain)" ]]; then
  echo "Error: working tree is dirty; commit or stash changes first." >&2
  exit 1
fi

if git -C "$REPO_ROOT" rev-parse -q --verify "refs/tags/${TAG}" >/dev/null; then
  echo "Error: tag ${TAG} already exists." >&2
  exit 1
fi

if ! command -v gh &>/dev/null; then
  echo "Error: 'gh' CLI is required to create pull requests." >&2
  exit 1
fi

if git -C "$REPO_ROOT" rev-parse -q --verify "refs/heads/${BRANCH}" >/dev/null; then
  echo "Error: branch ${BRANCH} already exists." >&2
  exit 1
fi

# --------------- create release branch ---------------
git -C "$REPO_ROOT" checkout -b "${BRANCH}"
echo "Created branch ${BRANCH}."

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
  sed -i '' "s|image: ghcr.io/taskbook-sh/taskbook-server:[^ ]*|image: ghcr.io/taskbook-sh/taskbook-server:${VERSION}|" "$DEPLOYMENT"
  echo "  updated k8s/deployment.yaml (local only, gitignored)"
fi

# --------------- verify build ---------------
echo "Running cargo check ..."
(cd "$REPO_ROOT" && cargo check --workspace)

# --------------- commit ---------------
git -C "$REPO_ROOT" add \
  crates/taskbook-common/Cargo.toml \
  crates/taskbook-client/Cargo.toml \
  crates/taskbook-server/Cargo.toml \
  overlay.nix \
  Cargo.lock

git -C "$REPO_ROOT" commit -m "Release ${TAG}"

echo "Created commit for ${TAG}."

# --------------- push & create PR ---------------
if [[ "$DRY_RUN" == true ]]; then
  echo ""
  echo "Dry-run mode: skipping push and PR creation."
  echo "To finish the release:"
  echo "  git push -u ${REMOTE} ${BRANCH}"
  echo "  gh pr create --title 'Release ${TAG}' --body 'Bump version to ${VERSION}.'"
else
  git -C "$REPO_ROOT" push -u "${REMOTE}" "${BRANCH}"
  PR_URL=$(gh pr create \
    --title "Release ${TAG}" \
    --body "$(cat <<EOF
Bump version to ${VERSION}.

After merging, finish the release:
\`\`\`
scripts/release.sh --tag ${VERSION}
\`\`\`
EOF
)" \
  )
  echo ""
  echo "Pull request created: ${PR_URL}"
  echo ""
  echo "After the PR is merged, run:"
  echo "  scripts/release.sh --tag ${VERSION}"
fi
