use dict_core::{EnSuggestPipeline, ZhToEnPipeline};
use dict_db::CedictDb;

fn sample_db() -> CedictDb {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/sample.db");
    if !std::path::Path::new(path).exists() {
        std::process::Command::new("cargo")
            .args([
                "run",
                "-p",
                "import_cedict",
                "--",
                "--input",
                concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/sample.u8"),
                "--output",
                path,
            ])
            .status()
            .expect("run import");
    }
    CedictDb::open_readonly(path).expect("open sample db")
}

#[test]
fn zh_lookup_study() {
    let db = sample_db();
    let pipeline = ZhToEnPipeline::default();
    let results = pipeline
        .search(&db, "学习", false, false, vec![])
        .expect("search");
    assert!(!results.is_empty());
    assert!(results.iter().any(|r| r.english.contains("study") || r.english.contains("learn")));
}

#[test]
fn en_prefix_learn() {
    let db = sample_db();
    let results = EnSuggestPipeline::search(&db, "learn", vec![]).expect("search");
    assert!(!results.is_empty());
}
