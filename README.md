# eng-dict

Lightweight English-learning dictionary for Chinese users. Desktop app (Rust + egui) with offline CC-CEDICT, Chinese→English and English→Chinese lookup, floating always-on-top widget, history, and saved words.

## Dictionary data (CC-CEDICT)

The app reads a local SQLite file `cedict.db` built from [CC-CEDICT](https://www.mdbg.net/chinese/dictionary?page=cc-cedict) (CC BY-SA 3.0). You need this file before running a **non-bundled** build.

### 1. Download

Official export (UTF-8, simplified + traditional):

```text
https://www.mdbg.net/chinese/export/cedict/cedict_1_0_ts_utf-8_mdbg.zip
```

Or use the helper script (downloads + unpacks automatically):

```bash
./scripts/prepare-dict.sh
```

This produces:

| File | Description |
|------|-------------|
| `data/cedict_ts.u8` | Raw CC-CEDICT text (~9 MB) |
| `data/cedict.db` | SQLite database used by the app (~50 MB) |

### 2. Process (import to SQLite)

The import tool parses `cedict_ts.u8`, builds indexes, English lemma table, FTS5, and fuzzy pinyin keys (`pinyin_fuzzy`).

Manual import:

```bash
cargo run -p import_cedict --release -- \
  --input data/cedict_ts.u8 \
  --output data/cedict.db
```

`prepare-dict.sh` runs the same import step after download.

### 3. Run (development build)

Requires `data/cedict.db` on disk (not embedded):

```bash
cargo run -p dict-app --release
```

User settings and `user.db` are stored under the OS app data directory (e.g. `~/Library/Application Support/eng-dict/` on macOS).

Point to another dictionary path in **Settings** if needed.

## Portable one-file build

To ship a **single executable** with the dictionary embedded (no separate `cedict.db` beside the binary):

```bash
./scripts/build-portable.sh
```

This will:

1. Run `prepare-dict.sh` (download + import if needed)
2. Build `dict-app` with the `bundled-dict` feature
3. Copy the binary to `dist/eng-dict-<os>-<arch>`

On first launch, the app extracts the bundled database once to:

- macOS: `~/Library/Application Support/eng-dict/cedict-bundled.db`
- Linux: `~/.local/share/eng-dict/cedict-bundled.db`

The executable is large (~55–60 MB) because it contains the full dictionary.

### Build portable manually

```bash
./scripts/prepare-dict.sh
cargo build --release -p dict-app --features bundled-dict
```

## Project layout

```text
crates/dict-core/   Lookup pipelines (ZH→EN, EN→CN, ranking)
crates/dict-db/     SQLite schema and user data
crates/dict-app/    egui desktop UI
tools/import_cedict Import CC-CEDICT .u8 → cedict.db
scripts/            prepare-dict.sh, build-portable.sh
data/               Dictionary files (gitignored)
```

## Tests

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

## License

Application code: MIT (see repository). Dictionary data: [CC-CEDICT](https://cc-cedict.org/wiki/) © MDBG, [CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/).
