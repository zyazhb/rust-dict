#!/usr/bin/env bash
# Build a single portable binary with CC-CEDICT embedded (~50 MB dictionary inside the exe).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/dist"
NAME="eng-dict"

cd "$ROOT"

"$ROOT/scripts/prepare-dict.sh"

mkdir -p "$DIST"

echo "Building release binary with bundled-dict feature..."
cargo build --release -p dict-app --features bundled-dict

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
OUT="$DIST/${NAME}-${OS}-${ARCH}"
cp "$ROOT/target/release/dict-app" "$OUT"
chmod +x "$OUT"

ls -lh "$OUT"
echo ""
echo "Portable binary: $OUT"
echo "First run extracts the dictionary to your app data folder; the exe is self-contained."
