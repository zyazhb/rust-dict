use dict_db::CedictEntry;
use serde::{Deserialize, Serialize};

use crate::rank::{MatchKind, RankBadge, RankedCandidate};
use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineCandidateDto {
    pub english: String,
    pub chinese: String,
    pub pinyin: String,
    pub definition: String,
}

pub trait OnlineProvider: Send + Sync {
    fn search_zh_to_en(&self, query: &str) -> Result<Vec<OnlineCandidateDto>>;
}

pub struct MockOnlineProvider;

impl OnlineProvider for MockOnlineProvider {
    fn search_zh_to_en(&self, query: &str) -> Result<Vec<OnlineCandidateDto>> {
        Ok(vec![OnlineCandidateDto {
            english: format!("(online) translation for {query}"),
            chinese: query.to_string(),
            pinyin: String::new(),
            definition: "Mock online result — configure API in Settings".into(),
        }])
    }
}

pub struct HttpOnlineProvider {
    pub base_url: String,
    pub api_key: String,
}

impl OnlineProvider for HttpOnlineProvider {
    fn search_zh_to_en(&self, query: &str) -> Result<Vec<OnlineCandidateDto>> {
        if self.base_url.is_empty() {
            return Err(crate::CoreError::Online(
                "online API URL not configured".into(),
            ));
        }
        let url = format!(
            "{}/translate?q={}",
            self.base_url.trim_end_matches('/'),
            urlencoding_query(query)
        );
        let mut req = ureq::get(&url);
        if !self.api_key.is_empty() {
            req = req.set("Authorization", &format!("Bearer {}", self.api_key));
        }
        let resp = req
            .call()
            .map_err(|e| crate::CoreError::Online(e.to_string()))?;
        let body: OnlineApiResponse = resp
            .into_json()
            .map_err(|e| crate::CoreError::Online(e.to_string()))?;
        Ok(body
            .results
            .into_iter()
            .map(|r| OnlineCandidateDto {
                english: r.english,
                chinese: r.chinese.unwrap_or_else(|| query.to_string()),
                pinyin: r.pinyin.unwrap_or_default(),
                definition: r.definition.unwrap_or_default(),
            })
            .collect())
    }
}

#[derive(Debug, Deserialize)]
struct OnlineApiResponse {
    results: Vec<OnlineApiResult>,
}

#[derive(Debug, Deserialize)]
struct OnlineApiResult {
    english: String,
    chinese: Option<String>,
    pinyin: Option<String>,
    definition: Option<String>,
}

fn urlencoding_query(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
                c.to_string()
            } else {
                format!("%{:02X}", c as u32)
            }
        })
        .collect()
}

pub fn dto_to_ranked(dtos: Vec<OnlineCandidateDto>, query: &str) -> Vec<RankedCandidate> {
    let mut id = -1i64;
    dtos.into_iter()
        .map(|d| {
            id -= 1;
            RankedCandidate {
                entry: CedictEntry {
                    id,
                    trad: d.chinese.clone(),
                    simp: d.chinese.clone(),
                    pinyin: d.pinyin.clone(),
                    pinyin_norm: String::new(),
                    definition: d.definition.clone(),
                },
                english: d.english.clone(),
                sense: d.definition.clone(),
                score: 0.4,
                match_kind: MatchKind::Online,
                badges: vec![RankBadge::Online],
                matched_chinese: query.to_string(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_provider_returns_result() {
        let p = MockOnlineProvider;
        let r = p.search_zh_to_en("你好").unwrap();
        assert_eq!(r.len(), 1);
        assert!(r[0].english.contains("你好"));
    }

    #[test]
    fn dto_to_ranked_marks_online() {
        let dtos = vec![OnlineCandidateDto {
            english: "hello".into(),
            chinese: "你好".into(),
            pinyin: "ni3 hao3".into(),
            definition: "greeting".into(),
        }];
        let ranked = dto_to_ranked(dtos, "你好");
        assert_eq!(ranked[0].badges.len(), 1);
    }
}
