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

#[allow(dead_code)]
pub fn cedict_pinyin_norm(pinyin: &str) -> String {
    pinyin
        .to_lowercase()
        .replace([':', '·'], " ")
        .split_whitespace()
        .map(|syllable| {
            let mut base = String::new();
            let mut tone = String::new();
            for ch in syllable.chars() {
                if ch.is_ascii_digit() {
                    tone.push(ch);
                } else if ch.is_ascii_alphabetic() {
                    base.push(ch);
                }
            }
            if tone.is_empty() {
                base
            } else {
                format!("{base}{tone}")
            }
        })
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
    fn normalizes_cedict_pinyin() {
        assert_eq!(cedict_pinyin_norm("xue2 xi2"), "xue2 xi2");
    }
}
