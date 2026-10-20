#!/usr/bin/env bash
set -Eeuo pipefail

# Publish using native GitHub Actions runners. This script only pushes Git refs
# and waits for CI; it does not build packages on this machine.
RELEASE_VERSION="${RELEASE_VERSION:-0.01}"
TAG="v${RELEASE_VERSION}"
REPO="AcronixLabs/Sublemon-Text"
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

command -v git >/dev/null || { echo "git is required" >&2; exit 1; }
command -v curl >/dev/null || { echo "curl is required" >&2; exit 1; }
command -v python3 >/dev/null || { echo "python3 is required" >&2; exit 1; }

BRANCH="$(git branch --show-current)"
if [[ -z "$BRANCH" ]]; then
  echo "Refusing to release from a detached HEAD." >&2
  exit 1
fi
if [[ -n "$(git status --porcelain)" ]]; then
  echo "Commit or stash all changes before releasing." >&2
  git status --short >&2
  exit 1
fi
if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
  echo "Local tag $TAG already exists; release tags are immutable." >&2
  exit 1
fi
if git ls-remote --exit-code --tags origin "refs/tags/$TAG" >/dev/null 2>&1; then
  echo "Remote tag $TAG already exists; release tags are immutable." >&2
  exit 1
fi

echo "Pushing $BRANCH and creating release tag $TAG over HTTPS..."
git push origin "$BRANCH"
git tag -a "$TAG" -m "Sublemon $RELEASE_VERSION"
git push origin "$TAG"

echo "Waiting for GitHub Actions to build packages and publish $TAG..."
API_URL="https://api.github.com/repos/$REPO/releases/tags/$TAG"
for attempt in $(seq 1 90); do
  if curl --fail --silent --max-time 15 "$API_URL" 2>/dev/null \
    | python3 -c 'import json,sys; d=json.load(sys.stdin); sys.exit(0 if d.get("html_url") and not d.get("draft", True) else 1)'; then
    echo "Release published: https://github.com/$REPO/releases/tag/$TAG"
    exit 0
  fi
  sleep 20
done

echo "The refs were pushed, but the release did not appear within 30 minutes." >&2
echo "Check the Actions run for tag $TAG: https://github.com/$REPO/actions" >&2
exit 1
