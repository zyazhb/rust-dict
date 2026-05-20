use dict_db::CedictDb;

use crate::normalize::{is_pinyin_input, normalize_query};
use crate::pinyin::pinyin_search_keys;
use crate::rank::{MatchKind, RankContext, RankedCandidate, rank_candidates};
use crate::Result;

const MAX_RESULTS: usize = 50;

#[derive(Debug, Default, Clone, Copy)]
pub struct ZhToEnPipeline;

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
            let keys = pinyin_search_keys(&q);
            for e in db.lookup_pinyin(&keys)? {
                raw.push((e, MatchKind::Pinyin, q.clone()));
            }
        } else {
            self.collect_phrase_matches(db, &q, use_trad, &mut raw);
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
