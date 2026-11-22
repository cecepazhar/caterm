#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
TAG="v2.1.44"

echo "=================================================="
echo "  Draft GitHub Release Creation (v2.1.44)"
echo "=================================================="

# 1. Check gh cli
if ! command -v gh &>/dev/null; then
  echo "ERROR: GitHub CLI ('gh') is not installed." >&2
  exit 1
fi

# 2. Verify files in dist
cd "$DIST_DIR"
echo "--> Verifying SHA256 checksums..."
sha256sum -c SHA256SUMS.txt

echo "--> Creating draft release on GitHub (cecepazhar/caterm)..."
gh release create "$TAG" \
  --repo cecepazhar/caterm \
  --draft \
  --title "CATerm v2.1.44 — GCC License v2, Device Management & Hardened Security" \
  --notes-file "$DIST_DIR/RELEASE_NOTES_v2.1.44.md" \
  "$DIST_DIR/CATerm_2.1.44_amd64.deb" \
  "$DIST_DIR/CATerm-2.1.44-1.x86_64.rpm" \
  "$DIST_DIR/CATerm_2.1.44_amd64.AppImage" \
  "$DIST_DIR/SHA256SUMS.txt"

echo "=================================================="
echo "  Draft release created successfully!"
echo "  Review on GitHub: https://github.com/cecepazhar/caterm/releases"
echo "=================================================="

