use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};

use crate::error::{DbError, Result};
use crate::models::{AppSettings, HistoryRecord, SavedWord, SearchMode};
use crate::schema::USER_SCHEMA;

pub struct UserDb {
    conn: Connection,
}

impl UserDb {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(USER_SCHEMA)?;
        Ok(Self { conn })
    }

    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }

    pub fn add_history(&self, query: &str, mode: SearchMode, selected_entry_id: Option<i64>) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO search_history (query, mode, created_at, selected_entry_id) VALUES (?1, ?2, ?3, ?4)",
            params![query, mode.as_str(), Self::now(), selected_entry_id],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_history(&self, limit: usize) -> Result<Vec<HistoryRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, query, mode, created_at, selected_entry_id FROM search_history
             ORDER BY created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let mode_str: String = row.get(2)?;
            Ok(HistoryRecord {
                id: row.get(0)?,
                query: row.get(1)?,
                mode: SearchMode::parse(&mode_str).unwrap_or(SearchMode::ZhToEn),
                created_at: row.get(3)?,
                selected_entry_id: row.get(4)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(DbError::from)
    }

    pub fn save_word(
        &self,
        english: &str,
        chinese: &str,
        pinyin: &str,
        definition: &str,
        note: &str,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO saved_words (english, chinese, pinyin, definition, note, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![english, chinese, pinyin, definition, note, Self::now()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_saved(&self) -> Result<Vec<SavedWord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, english, chinese, pinyin, definition, note, created_at FROM saved_words
             ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SavedWord {
                id: row.get(0)?,
                english: row.get(1)?,
                chinese: row.get(2)?,
                pinyin: row.get(3)?,
                definition: row.get(4)?,
                note: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(DbError::from)
    }

    pub fn delete_saved(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM saved_words WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_note(&self, id: i64, note: &str) -> Result<()> {
        self.conn
            .execute("UPDATE saved_words SET note = ?1 WHERE id = ?2", params![note, id])?;
        Ok(())
    }

    pub fn saved_english_boosts(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT chinese, english FROM saved_words")?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get(1)?)))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(DbError::from)
    }

    pub fn get_settings(&self) -> Result<AppSettings> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get(1)?)))?;
        let mut settings = AppSettings::default();
        for row in rows {
            let (k, v) = row?;
            match k.as_str() {
                "cedict_path" => settings.cedict_path = v,
                "online_enabled" => settings.online_enabled = v == "true",
                "online_api_url" => settings.online_api_url = v,
                "online_api_key" => settings.online_api_key = v,
                "online_score_threshold" => {
                    settings.online_score_threshold = v.parse().unwrap_or(0.3);
                }
                "show_traditional" => settings.show_traditional = v == "true",
                "locale" => settings.locale = v,
                "compact_hotkey" => settings.compact_hotkey = v,
                _ => {}
            }
        }
        Ok(settings)
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        let pairs = [
            ("cedict_path", settings.cedict_path.clone()),
            ("online_enabled", settings.online_enabled.to_string()),
            ("online_api_url", settings.online_api_url.clone()),
            ("online_api_key", settings.online_api_key.clone()),
            (
                "online_score_threshold",
                settings.online_score_threshold.to_string(),
            ),
            ("show_traditional", settings.show_traditional.to_string()),
            ("locale", settings.locale.clone()),
            ("compact_hotkey", settings.compact_hotkey.clone()),
        ];
        for (key, value) in pairs {
            self.conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )?;
        }
        Ok(())
    }

    pub fn get_online_cache(&self, query_hash: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload_json FROM online_cache WHERE query_hash = ?1")?;
        let mut rows = stmt.query(params![query_hash])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn set_online_cache(&self, query_hash: &str, payload_json: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO online_cache (query_hash, payload_json, created_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(query_hash) DO UPDATE SET payload_json = excluded.payload_json,
             created_at = excluded.created_at",
            params![query_hash, payload_json, Self::now()],
        )?;
        Ok(())
    }
}
