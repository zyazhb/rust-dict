#!/usr/bin/env bash
# Download CC-CEDICT (if missing) and build data/cedict.db
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DATA_DIR="$ROOT/data"
CEDICT_U8="$DATA_DIR/cedict_ts.u8"
CEDICT_ZIP="$DATA_DIR/cedict.zip"
CEDICT_DB="$DATA_DIR/cedict.db"
CEDICT_URL="https://www.mdbg.net/chinese/export/cedict/cedict_1_0_ts_utf-8_mdbg.zip"

mkdir -p "$DATA_DIR"

if [[ ! -f "$CEDICT_U8" ]]; then
  echo "Downloading CC-CEDICT from MDBG..."
  curl -fsSL -o "$CEDICT_ZIP" "$CEDICT_URL"
  unzip -o "$CEDICT_ZIP" -d "$DATA_DIR"
  if [[ ! -f "$CEDICT_U8" ]]; then
    echo "error: expected $CEDICT_U8 after unzip" >&2
    exit 1
  fi
  echo "Downloaded $(wc -l < "$CEDICT_U8" | tr -d ' ') lines -> $CEDICT_U8"
else
  echo "Using existing $CEDICT_U8"
fi

echo "Importing into SQLite..."
cargo run -p import_cedict --release -- \
  --input "$CEDICT_U8" \
  --output "$CEDICT_DB"

ls -lh "$CEDICT_DB"
echo "Done: $CEDICT_DB"
