//! Pinyin normalization: toned (`ni3 men5`), compact fuzzy (`nimen`), spaced (`ni men`).

/// Lowercase toned form with spaces between syllables.
pub fn normalize_pinyin(s: &str) -> String {
    let mut out = String::new();
    for ch in s.trim().to_lowercase().chars() {
        match ch {
            'a'..='z' | '0'..='9' | ' ' => out.push(ch),
            'ā' | 'á' | 'ǎ' | 'à' => out.push('a'),
            'ē' | 'é' | 'ě' | 'è' => out.push('e'),
            'ī' | 'í' | 'ǐ' | 'ì' => out.push('i'),
            'ō' | 'ó' | 'ǒ' | 'ò' => out.push('o'),
            'ū' | 'ú' | 'ǔ' | 'ù' => out.push('u'),
            'ǖ' | 'ǘ' | 'ǚ' | 'ǜ' | 'ü' => out.push('u'),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Strip tone digits and non-letters from one syllable: `ni3` -> `ni`, `lü4` -> `lu`.
pub fn strip_syllable_tone(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| if c == 'v' { 'u' } else { c })
        .collect()
}

/// Build compact fuzzy key from toned/normalized phrase: `ni3 men5` -> `nimen`.
pub fn fuzzy_compact_from_norm(pinyin_norm: &str) -> String {
    pinyin_norm
        .split_whitespace()
        .map(strip_syllable_tone)
        .collect()
}

/// Build compact fuzzy key from user input (any spacing/tones): `ni men`, `nimen`, `ni3men5` -> `nimen`.
pub fn fuzzy_compact_from_query(query: &str) -> String {
    query
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| if c == 'v' { 'u' } else { c })
        .collect()
}

/// Search keys to try: toned exact, compact fuzzy, fuzzy prefix.
pub fn pinyin_search_keys(query: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let toned = normalize_pinyin(query);
    if !toned.is_empty() {
        keys.push(toned);
    }
    let fuzzy = fuzzy_compact_from_query(query);
    if !fuzzy.is_empty() && !keys.iter().any(|k| k == &fuzzy) {
        keys.push(fuzzy);
    }
    // spaced syllables without tones: "ni men" -> also try as norm pieces
    if query.contains(char::is_whitespace) {
        let spaced: String = query
            .split_whitespace()
            .map(strip_syllable_tone)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if !spaced.is_empty() && !keys.iter().any(|k| k == &spaced) {
            keys.push(spaced);
        }
    }
    keys
}

pub fn cedict_pinyin_norm(pinyin: &str) -> String {
    pinyin
        .to_lowercase()
        .replace([':', '·'], " ")
        .split_whitespace()
        .map(|syllable| syllable.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_user_pinyin() {
        assert_eq!(normalize_pinyin("Xue2 Xi1"), "xue2 xi1");
    }

    #[test]
    fn fuzzy_compact_toned() {
        assert_eq!(fuzzy_compact_from_norm("ni3 men5"), "nimen");
    }

    #[test]
    fn fuzzy_compact_spaced() {
        assert_eq!(fuzzy_compact_from_query("ni men"), "nimen");
    }

    #[test]
    fn fuzzy_compact_merged() {
        assert_eq!(fuzzy_compact_from_query("nimen"), "nimen");
        assert_eq!(fuzzy_compact_from_query("ni3men5"), "nimen");
    }

    #[test]
    fn search_keys_include_fuzzy() {
        let keys = pinyin_search_keys("ni men");
        assert!(keys.contains(&"nimen".to_string()));
    }
}
