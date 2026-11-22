#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="${CATERM_ROOT_DIR:-$(cd "$SCRIPT_DIR/.." && pwd)}"

echo "==> Checking version synchronization across configuration files..."

# 1. Cargo.toml workspace version
CARGO_TOML="$ROOT_DIR/Cargo.toml"
CARGO_VER=$(awk -F'"' '/^[[:space:]]*version[[:space:]]*=/ {print $2; exit}' "$CARGO_TOML")

# 2. tauri.conf.json
TAURI_CONF="$ROOT_DIR/crates/caterm-app/tauri.conf.json"
TAURI_VER=$(awk -F'"' '/"version"[[:space:]]*:/ {print $4; exit}' "$TAURI_CONF")

# 3. frontend/package.json
PKG_JSON="$ROOT_DIR/frontend/package.json"
PKG_VER=$(awk -F'"' '/"version"[[:space:]]*:/ {print $4; exit}' "$PKG_JSON")

# 4. frontend/package-lock.json
LOCK_JSON="$ROOT_DIR/frontend/package-lock.json"
if [[ -f "$LOCK_JSON" ]]; then
  LOCK_VER=$(awk -F'"' '/"version"[[:space:]]*:/ {print $4; exit}' "$LOCK_JSON")
else
  LOCK_VER="$PKG_VER"
fi

echo "  Cargo.toml:                 $CARGO_VER"
echo "  tauri.conf.json:            $TAURI_VER"
echo "  frontend/package.json:      $PKG_VER"
echo "  frontend/package-lock.json: $LOCK_VER"

MISMATCH=0
if [[ "$CARGO_VER" != "$TAURI_VER" ]]; then
  echo "ERROR: Version mismatch between Cargo.toml ($CARGO_VER) and tauri.conf.json ($TAURI_VER)" >&2
  MISMATCH=1
fi

if [[ "$CARGO_VER" != "$PKG_VER" ]]; then
  echo "ERROR: Version mismatch between Cargo.toml ($CARGO_VER) and frontend/package.json ($PKG_VER)" >&2
  MISMATCH=1
fi

if [[ "$CARGO_VER" != "$LOCK_VER" ]]; then
  echo "ERROR: Version mismatch between Cargo.toml ($CARGO_VER) and frontend/package-lock.json ($LOCK_VER)" >&2
  MISMATCH=1
fi

if [[ $MISMATCH -ne 0 ]]; then
  echo "==> Version check FAILED. Please synchronize all version strings." >&2
  exit 1
fi

echo "==> All versions synchronized: v$CARGO_VER (OK)"
exit 0
