#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

usage() {
  cat <<'EOF'
Usage: scripts/release.sh [--dry-run] <version> [remote]

Bumps all version strings, creates a release branch, and opens a pull
request. After the PR is merged, tag the release with:

  git tag v<version> && git push origin v<version>

Options:
  --dry-run   Update files and create commit locally but do NOT push or
              create a pull request.

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

if ! command -v gh &>/dev/null; then
  echo "Error: 'gh' CLI is required to create pull requests." >&2
  exit 1
fi

BRANCH="release/${TAG}"

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
  sed -i '' "s|image: ghcr.io/alexanderdavidsen/taskbook-rs-server:[^ ]*|image: ghcr.io/alexanderdavidsen/taskbook-rs-server:${VERSION}|" "$DEPLOYMENT"
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
  echo "To finish the release, run:"
  echo "  git push ${REMOTE} ${BRANCH} && gh pr create --title 'Release ${TAG}' --body 'Bump version to ${VERSION}.'"
else
  git -C "$REPO_ROOT" push -u "${REMOTE}" "${BRANCH}"
  PR_URL=$(gh pr create \
    --title "Release ${TAG}" \
    --body "$(cat <<EOF
Bump version to ${VERSION}.

After merging, tag the release:
\`\`\`
git pull && git tag ${TAG} && git push ${REMOTE} ${TAG}
\`\`\`
EOF
)" \
    --repo "$(gh repo view --json nameWithOwner -q .nameWithOwner)" \
  )
  echo ""
  echo "Pull request created: ${PR_URL}"
  echo ""
  echo "After the PR is merged, tag the release:"
  echo "  git checkout master && git pull && git tag ${TAG} && git push ${REMOTE} ${TAG}"
fi
