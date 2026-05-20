use std::env;
use std::fs::File;
use std::path::Path;

use cedict::parse_reader;
use dict_db::schema::CEDICT_SCHEMA;
use rusqlite::Connection;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = arg(&args, "--input").expect("usage: import_cedict --input cc-cedict.u8 --output data/cedict.db");
    let output = arg(&args, "--output").expect("usage: import_cedict --input cc-cedict.u8 --output data/cedict.db");

    if let Some(parent) = Path::new(&output).parent() {
        std::fs::create_dir_all(parent).expect("create output dir");
    }
    if Path::new(&output).exists() {
        std::fs::remove_file(&output).expect("remove old db");
    }

    let mut conn = Connection::open(&output).expect("open db");
    conn.execute_batch("PRAGMA journal_mode = OFF; PRAGMA synchronous = OFF;")
        .expect("pragma");
    conn.execute_batch(CEDICT_SCHEMA).expect("schema");

    let file = File::open(&input).expect("open cedict file");
    let mut count = 0u64;

    {
        let tx = conn.transaction().expect("tx");
        {
            let mut insert = tx
                .prepare(
                    "INSERT INTO entries (id, trad, simp, pinyin, pinyin_norm, definition)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                )
                .expect("insert stmt");
            let mut lemma_insert = tx
                .prepare("INSERT OR IGNORE INTO english_lemmas (lemma, entry_id) VALUES (?1, ?2)")
                .expect("lemma stmt");

            for entry in parse_reader(file) {
                count += 1;
                let id = count as i64;
                let pinyin_norm = cedict_pinyin_norm(entry.pinyin());
                let defs: Vec<&str> = entry.definitions().collect();
                let def_slash = format!("/{} /", defs.join("/"));
                insert
                    .execute(rusqlite::params![
                        id,
                        entry.traditional(),
                        entry.simplified(),
                        entry.pinyin(),
                        pinyin_norm,
                        def_slash,
                    ])
                    .expect("insert entry");

                for lemma in extract_import_lemmas(&def_slash) {
                    lemma_insert
                        .execute(rusqlite::params![lemma, id])
                        .expect("insert lemma");
                }
            }
        }
        tx.commit().expect("commit");
    }

    conn.execute_batch(
        "INSERT INTO entries_fts(rowid, definition) SELECT id, definition FROM entries;
         INSERT INTO entries_fts(entries_fts) VALUES('rebuild');",
    )
    .expect("fts rebuild");

    println!("imported {count} entries -> {output}");
}

fn arg(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn cedict_pinyin_norm(pinyin: &str) -> String {
    pinyin
        .to_lowercase()
        .split_whitespace()
        .map(|syllable| syllable.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_import_lemmas(definition: &str) -> Vec<String> {
    let cleaned = definition.trim_matches('/');
    cleaned
        .split(['/', ';'])
        .flat_map(|sense| {
            let head = sense.split('(').next().unwrap_or(sense).trim();
            head.split_whitespace()
                .flat_map(|w| w.split(','))
                .map(|w| {
                    w.trim_matches(|c: char| !c.is_ascii_alphanumeric())
                        .to_lowercase()
                })
                .filter(|w| w.len() >= 2 && w.chars().all(|c| c.is_ascii_alphabetic()))
        })
        .collect()
}
