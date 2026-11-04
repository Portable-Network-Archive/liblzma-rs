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
    // lzma_index_s, lzma_index_hash_s and lzma_internal_s are opaque types in C
    // (only forward-declared), so ctest cannot verify their size, alignment, or
    // perform roundtrip tests. Their typedef aliases must also be skipped.
    // The anonymous unions/structs inside lzma_index_iter have bindgen-invented
    // type names (__bindgen_ty_*) that do not exist in C, so those types are
    // skipped as structs as well.
    cfg.skip_type(|n| {
        n == "__enum_ty" || n == "lzma_index" || n == "lzma_index_hash" || n == "lzma_internal"
    });
    cfg.skip_struct(|n| {
        n == "lzma_index_s"
            || n == "lzma_index_hash_s"
            || n == "lzma_internal_s"
            || n.contains("__bindgen_ty")
    });
    // lzma_index_iter itself is a complete struct; only the types of its
    // fields are anonymous in C, so only the field-type checks are skipped.
    // Its size, alignment and field offsets are still verified.
    cfg.skip_field_type(|s, _f| s == "lzma_index_iter");
    cfg.skip_roundtrip(|n| {
        n == "lzma_index_s"
            || n == "lzma_index_hash_s"
            || n == "lzma_internal_s"
            || n == "lzma_index_iter"
    });
    // LZMA_H_INTERNAL is the lzma.h include guard, not part of the API.
    cfg.skip_const(|n| n == "LZMA_H_INTERNAL");
    // The *_mt functions exist only when liblzma is built with thread support;
    // systest links the build without it.
    cfg.skip_fn(|n| {
        n == "lzma_stream_encoder_mt"
            || n == "lzma_stream_decoder_mt"
            || n == "lzma_stream_encoder_mt_memusage"
    });
    // bindgen renames the C field `type` to `type_` because it is a Rust keyword.
    cfg.field_name(|_s, field| match field {
        "type_" => "type".to_string(),
        _ => field.to_string(),
    });
    // bindgen prefixes enum variants with the enum type name
    // (e.g. lzma_ret_LZMA_OK); the C name is the bare variant name.
    cfg.const_cname(|n| {
        const ENUMS: &[&str] = &[
            "lzma_reserved_enum",
            "lzma_ret",
            "lzma_action",
            "lzma_check",
            "lzma_delta_type",
            "lzma_match_finder",
            "lzma_mode",
            "lzma_index_iter_mode",
        ];
        for e in ENUMS {
            if let Some(rest) = n.strip_prefix(&format!("{e}_")) {
                return rest.to_string();
            }
        }
        n.to_string()
    });
    cfg.generate("../liblzma-sys/src/lib.rs", "all.rs");
}
