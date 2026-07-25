#!/usr/bin/env bash
set -euo pipefail

OWNER="${1:-CaptSinBad}"
REPO="${2:-ogunedo}"
VISIBILITY="${3:-public}"

if ! command -v gh >/dev/null 2>&1; then
  echo "GitHub CLI (gh) is required: https://cli.github.com/" >&2
  exit 1
fi

gh auth status

if [ ! -d .git ]; then
  git init -b main
fi

git add .
if ! git diff --cached --quiet; then
  git commit -m "feat: publish the Ogunedo proof system"
fi

case "$VISIBILITY" in
  public|private|internal) ;;
  *) echo "visibility must be public, private, or internal" >&2; exit 1 ;;
esac

gh repo create "$OWNER/$REPO" "--$VISIBILITY" --source=. --remote=origin --push

echo "Created https://github.com/$OWNER/$REPO"
