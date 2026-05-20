pub const USER_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS search_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query TEXT NOT NULL,
    mode TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    selected_entry_id INTEGER
);

CREATE TABLE IF NOT EXISTS saved_words (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    english TEXT NOT NULL,
    chinese TEXT NOT NULL,
    pinyin TEXT NOT NULL DEFAULT '',
    definition TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS online_cache (
    query_hash TEXT PRIMARY KEY,
    payload_json TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

pub const CEDICT_SCHEMA: &str = r#"
CREATE TABLE entries (
    id INTEGER PRIMARY KEY,
    trad TEXT NOT NULL,
    simp TEXT NOT NULL,
    pinyin TEXT NOT NULL,
    pinyin_norm TEXT NOT NULL,
    definition TEXT NOT NULL
);

CREATE INDEX idx_entries_simp ON entries(simp);
CREATE INDEX idx_entries_trad ON entries(trad);
CREATE INDEX idx_entries_pinyin_norm ON entries(pinyin_norm);

CREATE VIRTUAL TABLE entries_fts USING fts5(
    definition,
    content='entries',
    content_rowid='id'
);

CREATE TABLE english_lemmas (
    lemma TEXT NOT NULL,
    entry_id INTEGER NOT NULL,
    PRIMARY KEY (lemma, entry_id)
);

CREATE INDEX idx_lemmas_prefix ON english_lemmas(lemma);
"#;
