use dict_db::{SearchMode, UserDb};

#[test]
fn history_and_saved_roundtrip() {
    let path = std::env::temp_dir().join("eng-dict-test-user.db");
    let _ = std::fs::remove_file(&path);
    let db = UserDb::open(path.to_str().unwrap()).unwrap();
    db.add_history("学习", SearchMode::ZhToEn, Some(1)).unwrap();
    let h = db.list_history(10).unwrap();
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].query, "学习");
    db.save_word("study", "学习", "xue2 xi2", "to study", "note").unwrap();
    let s = db.list_saved().unwrap();
    assert_eq!(s.len(), 1);
    assert_eq!(s[0].english, "study");
}
