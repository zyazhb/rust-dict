use dict_db::{CedictDb, SearchMode, UserDb};

use crate::en_pipeline::EnToCnPipeline;
use crate::normalize::{is_latin_input, normalize_query};
use crate::online::{OnlineProvider, dto_to_ranked};
use crate::rank::RankedCandidate;
use crate::zh_pipeline::ZhToEnPipeline;
use crate::Result;

#[derive(Default)]
pub struct QueryRouter {
    pub zh: ZhToEnPipeline,
}

impl QueryRouter {
    pub fn search_local(
        &self,
        cedict: &CedictDb,
        user: &UserDb,
        query: &str,
        mode: SearchMode,
        pinyin_mode: bool,
        use_trad: bool,
    ) -> Result<Vec<RankedCandidate>> {
        let boosts = user.saved_english_boosts()?;
        let q = normalize_query(query);
        if q.is_empty() {
            return Ok(vec![]);
        }
        match mode {
            SearchMode::ZhToEn => self.zh.search(cedict, &q, pinyin_mode, use_trad, boosts),
            SearchMode::EnToCn => {
                if !is_latin_input(&q) {
                    return self.zh.search(cedict, &q, pinyin_mode, use_trad, boosts);
                }
                EnToCnPipeline::search(cedict, &q, boosts)
            }
        }
    }

    pub fn needs_online_fallback(
        &self,
        local: &[RankedCandidate],
        threshold: f32,
        force: bool,
    ) -> bool {
        force || local.is_empty() || local.first().is_none_or(|c| c.score < threshold)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn search_with_online(
        &self,
        cedict: &CedictDb,
        user: &UserDb,
        query: &str,
        mode: SearchMode,
        pinyin_mode: bool,
        use_trad: bool,
        online: Option<&dyn OnlineProvider>,
        force_online: bool,
        threshold: f32,
    ) -> Result<Vec<RankedCandidate>> {
        let q = normalize_query(query);
        let mut local = self.search_local(cedict, user, &q, mode, pinyin_mode, use_trad)?;

        if mode != SearchMode::ZhToEn {
            return Ok(local);
        }

        if !self.needs_online_fallback(&local, threshold, force_online) {
            return Ok(local);
        }

        let Some(provider) = online else {
            return Ok(local);
        };

        let hash = format!("{:x}", md5_hash(&q));
        if let Some(cached) = user.get_online_cache(&hash)? {
            if let Ok(dtos) = serde_json::from_str::<Vec<crate::online::OnlineCandidateDto>>(&cached)
            {
                let online_results = dto_to_ranked(dtos, &q);
                local.extend(online_results);
                return Ok(local);
            }
        }

        match provider.search_zh_to_en(&q) {
            Ok(dtos) => {
                if let Ok(json) = serde_json::to_string(&dtos) {
                    let _ = user.set_online_cache(&hash, &json);
                }
                let online_results = dto_to_ranked(dtos, &q);
                local.extend(online_results);
            }
            Err(e) if local.is_empty() => return Err(e),
            Err(_) => {}
        }
        Ok(local)
    }
}

fn md5_hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}
