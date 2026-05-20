use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchMode {
    ZhToEn,
    EnToEn,
}

impl SearchMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            SearchMode::ZhToEn => "zh_to_en",
            SearchMode::EnToEn => "en_to_en",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "zh_to_en" => Some(SearchMode::ZhToEn),
            "en_to_en" => Some(SearchMode::EnToEn),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CedictEntry {
    pub id: i64,
    pub trad: String,
    pub simp: String,
    pub pinyin: String,
    pub pinyin_norm: String,
    pub definition: String,
}

#[derive(Debug, Clone)]
pub struct HistoryRecord {
    pub id: i64,
    pub query: String,
    pub mode: SearchMode,
    pub created_at: i64,
    pub selected_entry_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct SavedWord {
    pub id: i64,
    pub english: String,
    pub chinese: String,
    pub pinyin: String,
    pub definition: String,
    pub note: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppSettings {
    pub cedict_path: String,
    pub online_enabled: bool,
    pub online_api_url: String,
    pub online_api_key: String,
    pub online_score_threshold: f32,
    pub show_traditional: bool,
    /// UI locale: "en" or "zh"
    pub locale: String,
}
