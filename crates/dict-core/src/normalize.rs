use unicode_normalization::UnicodeNormalization;

pub fn normalize_input(s: &str) -> String {
    s.trim()
        .nfkc()
        .collect::<String>()
        .to_lowercase()
}

pub fn normalize_query(s: &str) -> String {
    s.trim().nfkc().collect::<String>()
}

pub fn is_latin_input(s: &str) -> bool {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .all(|c| c.is_ascii_alphabetic() || c == '-' || c == '\'' || c == ' ')
}

pub fn is_pinyin_input(s: &str) -> bool {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .all(|c| c.is_ascii_alphanumeric())
        && s.chars().any(|c| c.is_ascii_alphabetic())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_latin() {
        assert!(is_latin_input("learn"));
        assert!(!is_latin_input("学习"));
    }

    #[test]
    fn detects_pinyin() {
        assert!(is_pinyin_input("xue2xi1"));
        assert!(!is_pinyin_input("学习"));
    }
}
