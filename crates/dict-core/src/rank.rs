use std::collections::HashMap;

use dict_db::CedictEntry;

use crate::frequency::frequency_rank;
use crate::gloss::{extract_headword, split_english_senses};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchKind {
    PhraseExact,
    TokenExact,
    Substring,
    Pinyin,
    EnglishPrefix,
    EnglishFts,
    Online,
}

#[derive(Debug, Clone)]
pub enum RankBadge {
    ExactMatch,
    CommonWord,
    SavedBefore,
    Online,
}

#[derive(Debug, Clone)]
pub struct RankedCandidate {
    pub entry: CedictEntry,
    pub english: String,
    pub sense: String,
    pub score: f32,
    pub match_kind: MatchKind,
    pub badges: Vec<RankBadge>,
    pub matched_chinese: String,
}

pub struct RankContext {
    pub user_boosts: HashMap<String, String>,
    #[allow(dead_code)]
    pub query: String,
}

impl RankContext {
    pub fn new(query: &str, boosts: Vec<(String, String)>) -> Self {
        let user_boosts = boosts.into_iter().collect();
        Self {
            query: query.to_string(),
            user_boosts,
        }
    }
}

pub fn rank_candidates(
    entries: Vec<(CedictEntry, MatchKind, String)>,
    ctx: &RankContext,
    limit: usize,
) -> Vec<RankedCandidate> {
    let mut out = Vec::new();
    for (entry, kind, matched_zh) in entries {
        for sense in split_english_senses(&entry.definition) {
            let english = extract_headword(&sense);
            if english.is_empty() {
                continue;
            }
            let mut score = kind_score(kind);
            let freq = frequency_rank(&english);
            score += freq * 0.35;
            if english.len() < 40 {
                score += 0.1;
            } else {
                score -= 0.1;
            }
            let mut badges = Vec::new();
            if matches!(kind, MatchKind::PhraseExact | MatchKind::TokenExact) {
                badges.push(RankBadge::ExactMatch);
            }
            if freq > 0.5 {
                badges.push(RankBadge::CommonWord);
            }
            if ctx
                .user_boosts
                .get(&matched_zh)
                .is_some_and(|e| e.eq_ignore_ascii_case(&english))
            {
                score += 0.5;
                badges.push(RankBadge::SavedBefore);
            }
            out.push(RankedCandidate {
                entry: entry.clone(),
                english: english.clone(),
                sense,
                score,
                match_kind: kind,
                badges,
                matched_chinese: matched_zh.clone(),
            });
        }
    }
    out.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out.dedup_by(|a, b| a.entry.id == b.entry.id && a.english == b.english);
    out.truncate(limit);
    out
}

fn kind_score(kind: MatchKind) -> f32 {
    match kind {
        MatchKind::PhraseExact => 1.0,
        MatchKind::TokenExact => 0.85,
        MatchKind::Pinyin => 0.8,
        MatchKind::Substring => 0.5,
        MatchKind::EnglishPrefix => 0.7,
        MatchKind::EnglishFts => 0.55,
        MatchKind::Online => 0.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dict_db::CedictEntry;

    fn entry(def: &str) -> CedictEntry {
        CedictEntry {
            id: 1,
            trad: "學習".into(),
            simp: "学习".into(),
            pinyin: "xué xí".into(),
            pinyin_norm: "xue2 xi2".into(),
            definition: def.into(),
        }
    }

    #[test]
    fn prefers_common_word() {
        let ctx = RankContext::new("学习", vec![]);
        let ranked = rank_candidates(
            vec![(entry("/to study/learning/"), MatchKind::PhraseExact, "学习".into())],
            &ctx,
            10,
        );
        assert!(!ranked.is_empty());
        assert!(ranked[0].english.contains("study") || ranked[0].english.contains("learning"));
    }
}
