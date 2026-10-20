#!/usr/bin/env bash
set -Eeuo pipefail

# Publish a versioned release using native GitHub Actions runners. This script
# only orchestrates remote builds; it does not build packages on this machine.
RELEASE_VERSION="${RELEASE_VERSION:-0.01}"
TAG="v${RELEASE_VERSION}"
WORKFLOW="release-builds.yml"
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

command -v git >/dev/null || { echo "git is required" >&2; exit 1; }
command -v gh >/dev/null || { echo "GitHub CLI (gh) is required" >&2; exit 1; }
gh auth status >/dev/null

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
if gh release view "$TAG" >/dev/null 2>&1; then
  echo "GitHub release $TAG already exists; refusing to overwrite it." >&2
  exit 1
fi

echo "Pushing $BRANCH and creating release tag $TAG..."
git push origin "$BRANCH"
git tag -a "$TAG" -m "Sublemon $RELEASE_VERSION"
git push origin "$TAG"
COMMIT="$(git rev-parse "$TAG^{commit}")"

echo "Waiting for the native release builds for $TAG..."
RUN_ID=""
for attempt in $(seq 1 60); do
  RUN_ID="$(gh run list --workflow "$WORKFLOW" --commit "$COMMIT" --json databaseId,headSha --jq "[.[] | select(.headSha == \"$COMMIT\")][0].databaseId // empty")"
  [[ -n "$RUN_ID" ]] && break
  sleep 5
done
if [[ -z "$RUN_ID" ]]; then
  echo "No $WORKFLOW run appeared for commit $COMMIT." >&2
  exit 1
fi
gh run watch "$RUN_ID" --exit-status

DIST="$ROOT/dist/release-$RELEASE_VERSION"
mkdir -p "$DIST/artifacts" "$DIST/packages"
gh run download "$RUN_ID" --dir "$DIST/artifacts"

# Keep all requested native packages and the Windows executable/archive.
while IFS= read -r -d '' artifact; do
  cp -f "$artifact" "$DIST/packages/"
done < <(find "$DIST/artifacts" -type f \( \
  -name '*.deb' -o -name '*.pkg.tar.zst' -o -name '*.rpm' -o \
  -name '*.exe' -o -name '*.zip' \) -print0)

shopt -s nullglob
ASSETS=("$DIST/packages"/*)
if (( ${#ASSETS[@]} == 0 )); then
  echo "The workflow completed but produced no release assets." >&2
  exit 1
fi

echo "Creating GitHub release $TAG with ${#ASSETS[@]} assets..."
gh release create "$TAG" "${ASSETS[@]}" \
  --title "Sublemon $RELEASE_VERSION" \
  --generate-notes
echo "Release published: $(gh release view "$TAG" --json url --jq .url)"
