use std::fs;
use std::io;
use std::path::PathBuf;

const BUNDLED_DB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/cedict.db"));

pub fn default_cedict_path() -> String {
    extract_bundled_db().expect("extract bundled cedict.db")
}

fn extract_bundled_db() -> io::Result<String> {
    let path = bundled_db_path();
    if path.exists() {
        return Ok(path.to_string_lossy().into_owned());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, BUNDLED_DB)?;
    Ok(path.to_string_lossy().into_owned())
}

fn bundled_db_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("eng-dict").join("cedict-bundled.db")
}
