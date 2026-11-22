#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"

echo "=================================================="
echo "  CATerm v2 Official Release Pipeline"
echo "=================================================="

# Enforce TAURI_SIGNING_PRIVATE_KEY
if [[ -z "${TAURI_SIGNING_PRIVATE_KEY:-}" ]]; then
  echo "=====================================================================" >&2
  echo "ERROR: TAURI_SIGNING_PRIVATE_KEY environment variable is not set!" >&2
  echo "Release packaging requires owner private key for updater signing." >&2
  echo "Aborting release pipeline." >&2
  echo "=====================================================================" >&2
  exit 1
fi

# Step 1: Run Full CI
echo "--> Step 1/4: Running local CI pipeline..."
"$SCRIPT_DIR/ci.sh"

# Step 2: Build Linux Bundles
echo "--> Step 2/4: Building Linux release bundles..."
"$SCRIPT_DIR/build-all.sh" linux

# Step 3: Package Release & Generate Checksums
echo "--> Step 3/4: Packaging release artifacts..."
"$SCRIPT_DIR/package-release.sh" "$DIST_DIR"

# Step 4: Verify SHA256 Checksums
echo "--> Step 4/4: Verifying SHA256 checksums..."
(cd "$DIST_DIR" && sha256sum -c SHA256SUMS.txt)

echo "=================================================="
echo "  RELEASE PIPELINE COMPLETED SUCCESSFULLY!"
echo "=================================================="

