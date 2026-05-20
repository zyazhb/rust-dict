use std::collections::HashMap;

static TOP_WORDS: &[&str] = &[
    "the", "be", "to", "of", "and", "a", "in", "that", "have", "i", "it", "for", "not", "on",
    "with", "he", "as", "you", "do", "at", "this", "but", "his", "by", "from", "they", "we",
    "say", "her", "she", "or", "an", "will", "my", "one", "all", "would", "there", "their",
    "what", "so", "up", "out", "if", "about", "who", "get", "which", "go", "me", "when",
    "make", "can", "like", "time", "no", "just", "him", "know", "take", "people", "into",
    "year", "your", "good", "some", "could", "them", "see", "other", "than", "then", "now",
    "look", "only", "come", "its", "over", "think", "also", "back", "after", "use", "two",
    "how", "our", "work", "first", "well", "way", "even", "new", "want", "because", "any",
    "these", "give", "day", "most", "us", "study", "learn", "school", "student", "teacher",
    "read", "write", "speak", "listen", "word", "language", "english", "chinese", "book",
    "master", "practice", "understand", "know", "teach", "education", "knowledge",
];

pub fn frequency_rank(word: &str) -> f32 {
    let w = word.to_lowercase();
    TOP_WORDS
        .iter()
        .position(|&x| x == w)
        .map(|pos| 1.0 - (pos as f32 / TOP_WORDS.len() as f32))
        .unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn build_frequency_map() -> HashMap<String, f32> {
    TOP_WORDS
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let score = 1.0 - (i as f32 / TOP_WORDS.len() as f32);
            (w.to_string(), score)
        })
        .collect()
}
