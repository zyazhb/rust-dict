use std::collections::HashSet;

use dict_db::CedictDb;

use crate::gloss::extract_lemmas;
use crate::normalize::normalize_input;
use crate::rank::{MatchKind, RankContext, RankedCandidate, rank_candidates};
use crate::Result;

const MAX_RESULTS: usize = 50;

pub struct EnSuggestPipeline;

impl EnSuggestPipeline {
    pub fn search(
        db: &CedictDb,
        query: &str,
        boosts: Vec<(String, String)>,
    ) -> Result<Vec<RankedCandidate>> {
        let q = normalize_input(query);
        if q.is_empty() {
            return Ok(vec![]);
        }

        let mut raw = Vec::new();
        let mut seen_ids = HashSet::new();

        for e in db.lookup_english_prefix(&q)? {
            if seen_ids.insert(e.id) {
                raw.push((e, MatchKind::EnglishPrefix, q.clone()));
            }
        }

        if raw.len() < MAX_RESULTS {
            if let Ok(fts) = db.lookup_english_fts(&q) {
                for e in fts {
                    if seen_ids.insert(e.id) {
                        raw.push((e, MatchKind::EnglishFts, q.clone()));
                    }
                }
            }
        }

        let mut related = Vec::new();
        for (entry, _, _) in raw.iter().take(10) {
            if let Ok(lemmas) = db.related_lemmas(entry.id) {
                for lemma in lemmas {
                    if lemma.starts_with(&q) && lemma != q {
                        related.push(lemma);
                    }
                }
            }
            for lemma in extract_lemmas(&entry.definition) {
                if lemma.starts_with(&q) && lemma != q {
                    related.push(lemma);
                }
            }
        }
        related.sort();
        related.dedup();
        for lemma in related.into_iter().take(20) {
            for e in db.lookup_english_prefix(&lemma)? {
                if seen_ids.insert(e.id) {
                    raw.push((e, MatchKind::EnglishPrefix, lemma.clone()));
                }
            }
        }

        let ctx = RankContext::new(&q, boosts);
        Ok(rank_candidates(raw, &ctx, MAX_RESULTS))
    }
}

/// English word in → Chinese headwords out (direct dictionary hits, no related-English expansion).
pub struct EnToCnPipeline;

impl EnToCnPipeline {
    pub fn search(
        db: &CedictDb,
        query: &str,
        boosts: Vec<(String, String)>,
    ) -> Result<Vec<RankedCandidate>> {
        let q = normalize_input(query);
        if q.is_empty() {
            return Ok(vec![]);
        }

        let mut raw = Vec::new();
        let mut seen_ids = HashSet::new();

        for e in db.lookup_english_prefix(&q)? {
            if seen_ids.insert(e.id) {
                raw.push((e, MatchKind::EnglishPrefix, q.clone()));
            }
        }

        if raw.len() < MAX_RESULTS {
            if let Ok(fts) = db.lookup_english_fts(&q) {
                for e in fts {
                    if seen_ids.insert(e.id) {
                        raw.push((e, MatchKind::EnglishFts, q.clone()));
                    }
                }
            }
        }

        let ctx = RankContext::new(&q, boosts);
        Ok(rank_candidates(raw, &ctx, MAX_RESULTS))
    }
}
