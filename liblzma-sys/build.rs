use std::env;
use std::fs;
use std::path::PathBuf;

const SKIP_FILENAMES: &[&str] = &["crc32_small", "crc64_small", "crc_clmul_consts_gen"];
const MIN_LIBLZMA: &str = "5.8.0";

fn main() {
    let target = env::var("TARGET").unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=LZMA_API_STATIC");
    let want_static = cfg!(feature = "static") || env::var("LZMA_API_STATIC").is_ok();
    let msvc = target.contains("msvc");

    // If a static link is desired, we compile from source.
    // If we're compiling for MSVC, pkg-config runs a risk of picking up MinGW
    // libraries by accident, so disable it.
    //
    // Otherwise, check the system to see if it has a lzma library already
    // installed that we can use.
    let mut config = pkg_config::Config::new();
    // Skip liblzma version check if `uncheck_liblzma_version` feature flag enabled.
    if !cfg!(feature = "uncheck_liblzma_version") {
        config.atleast_version(MIN_LIBLZMA);
    }
    if !want_static && !msvc {
        let pkg = config.probe("liblzma");

        if pkg.is_ok() {
            return;
        }
    }

    let want_parallel = cfg!(feature = "parallel");

    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:root={}", out_dir);
    let include_dir = env::current_dir().unwrap().join("xz/src/liblzma/api");
    println!("cargo:include={}", include_dir.display());

    let mut src_files = [
        "xz/src/liblzma/common",
        "xz/src/liblzma/lzma",
        "xz/src/liblzma/lz",
        "xz/src/liblzma/check",
        "xz/src/liblzma/delta",
        "xz/src/liblzma/rangecoder",
        "xz/src/liblzma/simple",
    ]
    .iter()
    .flat_map(|dir| read_dir_files(dir))
    .chain(vec![
        "xz/src/common/tuklib_cpucores.c".into(),
        "xz/src/common/tuklib_physmem.c".into(),
    ])
    .collect::<Vec<_>>();

    if !want_parallel {
        src_files = src_files
            .into_iter()
            .filter(|path| !path.file_stem().unwrap().to_str().unwrap().ends_with("mt"))
            .collect::<Vec<_>>();
    }

    // sort to make build reproducible.
    src_files.sort();

    let mut build = cc::Build::new();

    if !cfg!(debug_assertions) {
        build.define("NDEBUG", None);
    }

    build
        .files(src_files)
        // all C preproc defines are in `./config.h`
        .define("HAVE_CONFIG_H", "1")
        .include("xz/src/liblzma/api")
        .include("xz/src/liblzma/lzma")
        .include("xz/src/liblzma/lz")
        .include("xz/src/liblzma/check")
        .include("xz/src/liblzma/simple")
        .include("xz/src/liblzma/delta")
        .include("xz/src/liblzma/common")
        .include("xz/src/liblzma/rangecoder")
        .include("xz/src/common")
        .include(env::current_dir().unwrap());

    if !target.ends_with("msvc") {
        build.flag("-std=c99");
        if want_parallel {
            build.flag("-pthread");
        }
    }
    if cfg!(feature = "fat-lto") {
        build.flag_if_supported("-flto");
    } else if cfg!(feature = "thin-lto") {
        // find the first flag in `flags` that is supported and add that to.
        let flags = ["-flto=thin", "-flto"];
        let option = flags
            .into_iter()
            .find(|flag| build.is_flag_supported(flag).unwrap_or_default());
        if let Some(flag) = option {
            build.flag(flag);
        }
    }
    if want_parallel {
        build.define("LZMA_SYS_ENABLE_THREADS", "1");
    }

    if let Ok(s) = env::var("CARGO_CFG_TARGET_ENDIAN") {
        if s == "big" {
            build.define("WORDS_BIGENDIAN", None);
        }
    }

    // List out the WASM targets that need wasm-shim.
    // Note that Emscripten already provides its own C standard library so
    // wasm32-unknown-emscripten should not be included here.
    let need_wasm_shim = target == "wasm32-unknown-unknown" || target.starts_with("wasm32-wasi");
    if need_wasm_shim {
        println!("cargo:rerun-if-changed=wasm-shim/assert.h");
        println!("cargo:rerun-if-changed=wasm-shim/stdlib.h");
        println!("cargo:rerun-if-changed=wasm-shim/string.h");
        build.include("wasm-shim/");
    }

    build.compile("liblzma.a");
}

fn read_dir_files(dir: &str) -> impl Iterator<Item = PathBuf> {
    fs::read_dir(dir)
        .unwrap_or_else(|_| panic!("failed to read dir {}", dir))
        .filter_map(|ent| {
            let ent = ent.expect("failed to read entry");

            if ent.file_type().unwrap().is_dir() {
                return None;
            }

            let path = ent.path();

            if path.extension().unwrap() != "c" {
                return None;
            }

            {
                let file_stem = path.file_stem().unwrap().to_str().unwrap();
                if SKIP_FILENAMES.contains(&file_stem) {
                    return None;
                }
                if file_stem.ends_with("tablegen") {
                    return None;
                }
            }

            Some(path)
        })
}
