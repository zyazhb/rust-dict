pub fn split_english_senses(definition: &str) -> Vec<String> {
    let cleaned = definition.trim_start_matches('/').trim_end_matches('/');
    if cleaned.is_empty() {
        return vec![];
    }
    cleaned
        .split(['/', ';'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn extract_headword(sense: &str) -> String {
    let sense = sense.trim();
    if let Some(idx) = sense.find('(') {
        sense[..idx].trim().to_string()
    } else if let Some(idx) = sense.find(',') {
        sense[..idx].trim().to_string()
    } else {
        sense.to_string()
    }
}

pub fn extract_lemmas(definition: &str) -> Vec<String> {
    let mut lemmas = Vec::new();
    for sense in split_english_senses(definition) {
        for word in extract_headword(&sense)
            .split_whitespace()
            .flat_map(|w| w.split(','))
        {
            let w = word
                .trim_matches(|c: char| !c.is_ascii_alphanumeric())
                .to_lowercase();
            if w.len() >= 2 && w.chars().all(|c| c.is_ascii_alphabetic()) {
                lemmas.push(w);
            }
        }
    }
    lemmas.sort();
    lemmas.dedup();
    lemmas
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_senses() {
        let d = "/to study/learning/";
        let s = split_english_senses(d);
        assert_eq!(s.len(), 2);
        assert!(s[0].contains("study"));
    }

    #[test]
    fn extracts_headword() {
        assert_eq!(extract_headword("to study (a subject)"), "to study");
    }
}
