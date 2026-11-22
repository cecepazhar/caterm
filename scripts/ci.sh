#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=================================================="
echo "  CATerm Local CI/CD Pipeline (v2.1)"
echo "=================================================="

# 1. Version Sync Check
echo "--> [1/5] Checking version synchronization..."
"$SCRIPT_DIR/check-version-sync.sh"

# 2. Rust Formatting Check
echo "--> [2/5] Running cargo fmt --check..."
(cd "$ROOT_DIR" && cargo fmt --check)

# 3. Rust Clippy (Zero Warnings Enforcement)
echo "--> [3/5] Running cargo clippy --workspace --all-targets -- -D warnings..."
(cd "$ROOT_DIR" && cargo clippy --workspace --all-targets -- -D warnings)

# 4. Rust Workspace Tests
echo "--> [4/5] Running cargo test --workspace..."
(cd "$ROOT_DIR" && cargo test --workspace)

# 5. Frontend Type & Lint Check
echo "--> [5/5] Running frontend type check (svelte-check)..."
(cd "$ROOT_DIR/frontend" && npm run check)

echo "=================================================="
echo "  ALL CI CHECKS PASSED (Green)!"
echo "=================================================="

