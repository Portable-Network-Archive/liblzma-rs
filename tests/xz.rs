use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use liblzma::read;
use liblzma::stream;
use liblzma::write;

#[test]
fn standard_files() {
    for file in Path::new("liblzma-sys/xz/tests/files").read_dir().unwrap() {
        let file = file.unwrap();
        if file.path().extension().and_then(|s| s.to_str()) != Some("xz") {
            continue;
        }

        let filename = file.file_name().into_string().unwrap();

        // This appears to be implementation-defined how it's handled
        if filename.contains("unsupported-check") {
            continue;
        }

        println!("testing {:?}", file.path());
        let mut contents = Vec::new();
        File::open(&file.path())
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        if filename.starts_with("bad") || filename.starts_with("unsupported") {
            test_bad(&contents);
        } else {
            test_good(&contents);
        }
    }
}

fn test_good(data: &[u8]) {
    let mut ret = Vec::new();
    read::XzDecoder::new_multi_decoder(data)
        .read_to_end(&mut ret)
        .unwrap();
    let mut w = write::XzDecoder::new_multi_decoder(ret);
    w.write_all(data).unwrap();
    w.finish().unwrap();
}

fn test_bad(data: &[u8]) {
    let mut ret = Vec::new();
    let stream = stream::Stream::new_stream_decoder(u64::MAX, stream::CONCATENATED).unwrap();
    let result = read::XzDecoder::new_stream(data, stream).read_to_end(&mut ret);
    assert!(result.is_err(), "{result:?}");
    let mut w = write::XzDecoder::new(ret);
    assert!(w.write_all(data).is_err() || w.finish().is_err());
}

fn raw_lzma2_filters(opts: &stream::LzmaOptions) -> stream::Filters {
    let mut filters = stream::Filters::new();
    filters.lzma2(opts);
    filters
}

#[test]
fn preset_dict_roundtrip() {
    let dict = b"the quick brown fox jumps over the lazy dog. ".repeat(16);
    let data = b"the quick brown fox jumps over the lazy dog and then runs away quickly";

    // Encode with a preset dictionary. Drop `opts` before encoding to prove the
    // dictionary buffer's lifetime is owned by `Filters`, not the borrowed opts.
    let compressed = {
        let mut opts = stream::LzmaOptions::new_preset(6).unwrap();
        opts.preset_dict(dict.clone());
        let filters = raw_lzma2_filters(&opts);
        drop(opts);
        let enc = stream::Stream::new_raw_encoder(&filters).unwrap();
        let mut out = Vec::new();
        read::XzEncoder::new_stream(&data[..], enc)
            .read_to_end(&mut out)
            .unwrap();
        out
    };

    // Decode with the same preset dictionary -> original data.
    let mut opts = stream::LzmaOptions::new_preset(6).unwrap();
    opts.preset_dict(dict.clone());
    let filters = raw_lzma2_filters(&opts);
    let dec = stream::Stream::new_raw_decoder(&filters).unwrap();
    let mut decoded = Vec::new();
    read::XzDecoder::new_stream(&compressed[..], dec)
        .read_to_end(&mut decoded)
        .unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn preset_dict_affects_output() {
    let dict = b"the quick brown fox jumps over the lazy dog. ".repeat(16);
    let data = b"the quick brown fox jumps over the lazy dog and then runs away quickly";

    let mut opts = stream::LzmaOptions::new_preset(6).unwrap();
    opts.preset_dict(dict.clone());
    let filters = raw_lzma2_filters(&opts);
    let enc = stream::Stream::new_raw_encoder(&filters).unwrap();
    let mut compressed = Vec::new();
    read::XzEncoder::new_stream(&data[..], enc)
        .read_to_end(&mut compressed)
        .unwrap();

    // Decoding the same stream WITHOUT the preset dictionary must not yield the
    // original data, proving the dictionary actually influenced encoding.
    let opts_no_dict = stream::LzmaOptions::new_preset(6).unwrap();
    let filters_no_dict = raw_lzma2_filters(&opts_no_dict);
    let dec = stream::Stream::new_raw_decoder(&filters_no_dict).unwrap();
    let mut decoded = Vec::new();
    let result = read::XzDecoder::new_stream(&compressed[..], dec).read_to_end(&mut decoded);
    assert!(
        result.is_err() || decoded != data,
        "decoding without the preset dictionary unexpectedly reproduced the data"
    );
}

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn impls_send_and_sync() {
    assert_send_sync::<stream::Stream>();
    assert_send_sync::<read::XzDecoder<&[u8]>>();
    assert_send_sync::<read::XzEncoder<&[u8]>>();
    assert_send_sync::<write::XzEncoder<&mut [u8]>>();
    assert_send_sync::<write::XzDecoder<&mut [u8]>>();
}
