use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(feature, values(\"bundled-dict\"))");

    if env::var("CARGO_FEATURE_BUNDLED_DICT").is_err() {
        return;
    }

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let source = manifest.join("../../data/cedict.db");
    if !source.exists() {
        panic!(
            "feature `bundled-dict` requires data/cedict.db.\n\
             Run: ./scripts/prepare-dict.sh"
        );
    }

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("cedict.db");
    std::fs::copy(&source, &out).expect("copy cedict.db into OUT_DIR");

    println!("cargo:rerun-if-changed={}", source.display());
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_BUNDLED_DICT");
}
