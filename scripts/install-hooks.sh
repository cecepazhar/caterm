#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
GIT_HOOKS_DIR="$ROOT_DIR/.git/hooks"

echo "==> Installing CATerm Git pre-push hook (opt-in)..."

if [[ ! -d "$GIT_HOOKS_DIR" ]]; then
  echo "ERROR: .git/hooks directory not found at $GIT_HOOKS_DIR" >&2
  exit 1
fi

HOOK_SRC="$SCRIPT_DIR/hooks/pre-push"
HOOK_DST="$GIT_HOOKS_DIR/pre-push"

if [[ ! -f "$HOOK_SRC" ]]; then
  echo "ERROR: Source hook $HOOK_SRC not found!" >&2
  exit 1
fi

cp -f "$HOOK_SRC" "$HOOK_DST"
chmod +x "$HOOK_DST"

echo "==> Hook installed successfully at $HOOK_DST"
echo "==> To uninstall at any time, run: rm $HOOK_DST"

