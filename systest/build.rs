use std::env;

fn main() {
    let mut cfg = ctest2::TestGenerator::new();
    for (key, _) in env::vars() {
        if let Some(feature) = key.strip_prefix("CARGO_FEATURE_") {
            cfg.cfg("feature", Some(&feature.to_lowercase().replace('_', "-")));
        }
    }
    if let Ok(out) = env::var("DEP_LZMA_INCLUDE") {
        cfg.include(&out);
    }

    cfg.header("lzma.h");
    cfg.type_name(|n, _s, _| n.to_string());
    cfg.define("LZMA_API_STATIC", None);
    // lzma_index_s is an opaque type in C (only forward-declared), so ctest cannot
    // verify its size, alignment, or perform roundtrip tests.
    // The lzma_index typedef aliases it and must also be skipped.
    cfg.skip_type(|n| n == "__enum_ty" || n == "lzma_index");
    cfg.skip_struct(|n| n == "lzma_index_s");
    cfg.skip_roundtrip(|n| n == "lzma_index_s");
    cfg.generate("../liblzma-sys/src/lib.rs", "all.rs");
}
