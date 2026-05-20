use dict_db::CedictDb;
use jieba_rs::Jieba;

use crate::normalize::{is_pinyin_input, normalize_query};
use crate::pinyin::normalize_pinyin;
use crate::rank::{MatchKind, RankContext, RankedCandidate, rank_candidates};
use crate::Result;

const MAX_RESULTS: usize = 50;

pub struct ZhToEnPipeline {
    jieba: Jieba,
}

impl Default for ZhToEnPipeline {
    fn default() -> Self {
        Self {
            jieba: Jieba::new(),
        }
    }
}

impl ZhToEnPipeline {
    pub fn search(
        &self,
        db: &CedictDb,
        query: &str,
        pinyin_mode: bool,
        use_trad: bool,
        boosts: Vec<(String, String)>,
    ) -> Result<Vec<RankedCandidate>> {
        let q = normalize_query(query);
        if q.is_empty() {
            return Ok(vec![]);
        }

        let mut raw = Vec::new();

        if pinyin_mode || is_pinyin_input(&q) {
            let norm = normalize_pinyin(&q);
            for e in db.lookup_pinyin(&norm)? {
                raw.push((e, MatchKind::Pinyin, q.clone()));
            }
        } else {
            self.collect_phrase_matches(db, &q, use_trad, &mut raw);
            if raw.is_empty() {
                let tokens: Vec<String> = self
                    .jieba
                    .cut(&q, false)
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect();
                for token in &tokens {
                    for e in db.lookup_exact(token, use_trad)? {
                        raw.push((e, MatchKind::TokenExact, token.clone()));
                    }
                }
                if tokens.len() > 1 {
                    for e in db.lookup_exact(&q, use_trad)? {
                        raw.push((e, MatchKind::PhraseExact, q.clone()));
                    }
                }
            }
        }

        let ctx = RankContext::new(&q, boosts);
        Ok(rank_candidates(raw, &ctx, MAX_RESULTS))
    }

    fn collect_phrase_matches(
        &self,
        db: &CedictDb,
        q: &str,
        use_trad: bool,
        raw: &mut Vec<(dict_db::CedictEntry, MatchKind, String)>,
    ) {
        if let Ok(entries) = db.lookup_exact(q, use_trad) {
            for e in entries {
                raw.push((e, MatchKind::PhraseExact, q.to_string()));
            }
        }
        let chars: Vec<char> = q.chars().collect();
        for win in (1..=4.min(chars.len())).rev() {
            if win == chars.len() {
                continue;
            }
            for i in 0..=chars.len().saturating_sub(win) {
                let slice: String = chars[i..i + win].iter().collect();
                if slice == q {
                    continue;
                }
                if let Ok(entries) = db.lookup_exact(&slice, use_trad) {
                    for e in entries {
                        raw.push((e, MatchKind::Substring, slice.clone()));
                    }
                }
            }
        }
    }
}
