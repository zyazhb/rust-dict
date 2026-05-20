use rusqlite::{params, Connection, OpenFlags};

use crate::error::{DbError, Result};
use crate::models::CedictEntry;
use crate::schema::CEDICT_SCHEMA;

pub struct CedictDb {
    conn: Connection,
}

impl CedictDb {
    pub fn open_readonly(path: &str) -> Result<Self> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        Ok(Self { conn })
    }

    pub fn open_readwrite(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(CEDICT_SCHEMA)?;
        Ok(Self { conn })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn lookup_exact(&self, hanzi: &str, use_trad: bool) -> Result<Vec<CedictEntry>> {
        let col = if use_trad { "trad" } else { "simp" };
        let sql = format!(
            "SELECT id, trad, simp, pinyin, pinyin_norm, definition FROM entries WHERE {col} = ?1 LIMIT 50"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![hanzi], row_to_entry)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(DbError::from)
    }

    pub fn lookup_pinyin(&self, keys: &[String]) -> Result<Vec<CedictEntry>> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        let sql = "SELECT id, trad, simp, pinyin, pinyin_norm, definition FROM entries
             WHERE pinyin_norm = ?1 OR pinyin_norm LIKE ?2
                OR pinyin_fuzzy = ?3 OR pinyin_fuzzy LIKE ?4
             LIMIT 50";
        let mut stmt = self.conn.prepare(sql)?;
        for key in keys {
            if key.is_empty() {
                continue;
            }
            let norm_prefix = format!("{key}%");
            let fuzzy = key
                .chars()
                .filter(|c| c.is_ascii_alphabetic())
                .collect::<String>();
            if fuzzy.is_empty() {
                continue;
            }
            let fuzzy_prefix = format!("{fuzzy}%");
            let rows = stmt.query_map(
                params![key, norm_prefix, fuzzy, fuzzy_prefix],
                row_to_entry,
            )?;
            for row in rows {
                let entry = row?;
                if seen.insert(entry.id) {
                    out.push(entry);
                }
            }
            if out.len() >= 50 {
                break;
            }
        }
        Ok(out)
    }

    pub fn lookup_english_prefix(&self, prefix: &str) -> Result<Vec<CedictEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.trad, e.simp, e.pinyin, e.pinyin_norm, e.definition
             FROM english_lemmas l
             JOIN entries e ON e.id = l.entry_id
             WHERE l.lemma LIKE ?1
             ORDER BY l.lemma
             LIMIT 50",
        )?;
        let pattern = format!("{}%", prefix.to_lowercase());
        let rows = stmt.query_map(params![pattern], row_to_entry)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(DbError::from)
    }

    pub fn lookup_english_fts(&self, query: &str) -> Result<Vec<CedictEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.trad, e.simp, e.pinyin, e.pinyin_norm, e.definition
             FROM entries_fts f
             JOIN entries e ON e.id = f.rowid
             WHERE entries_fts MATCH ?1
             LIMIT 50",
        )?;
        let fts_query = format!("{query}*");
        let rows = stmt.query_map(params![fts_query], row_to_entry)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(DbError::from)
    }

    pub fn related_lemmas(&self, entry_id: i64) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT lemma FROM english_lemmas WHERE entry_id = ?1")?;
        let rows = stmt.query_map(params![entry_id], |row| row.get(0))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(DbError::from)
    }
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<CedictEntry> {
    Ok(CedictEntry {
        id: row.get(0)?,
        trad: row.get(1)?,
        simp: row.get(2)?,
        pinyin: row.get(3)?,
        pinyin_norm: row.get(4)?,
        definition: row.get(5)?,
    })
}
