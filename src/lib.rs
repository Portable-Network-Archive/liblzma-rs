//! LZMA/XZ encoding and decoding streams
//!
//! This library is a binding to liblzma currently to provide LZMA and xz
//! encoding/decoding streams. I/O streams are provided in the `read`, `write`,
//! and `bufread` modules (same types, different bounds). Raw in-memory
//! compression/decompression is provided via the `stream` module and contains
//! many of the raw APIs in liblzma.
//!
//! # Examples
//!
//! ```
//! use liblzma::read::{XzDecoder, XzEncoder};
//! use std::io::prelude::*;
//!
//! // Round trip some bytes from a byte source, into a compressor, into a
//! // decompressor, and finally into a vector.
//! let data = "Hello, World!".as_bytes();
//! let compressor = XzEncoder::new(data, 9);
//! let mut decompressor = XzDecoder::new(compressor);
//!
//! let mut contents = String::new();
//! decompressor.read_to_string(&mut contents).unwrap();
//! assert_eq!(contents, "Hello, World!");
//! ```
//! # Static linking
//!
//! You can enable static-linking using the `static` feature, so that the XZ
//! library is not required at runtime:
//!
//! ```toml
//! liblzma = { version = "0.4", features = ["static"] }
//! ```
//!
//! # Multithreading
//!
//! This crate optionally can support multithreading using the `parallel`
//! feature of this crate:
//!
//! ```toml
//! liblzma = { version = "0.4", features = ["parallel"] }
//! ```
//!
//! # Async I/O
//!
//! Dropped `tokio` support since 0.4.0.
//! If you need to use async I/O, use `async-compression` crate with `lzma` feature flag.
//!
//! ```toml
//! async-compression = { version = "0.4", features = ["lzma"] }
//! ```

#![doc(html_root_url = "https://docs.rs/liblzma/0.4.2")]
#![deny(missing_docs)]

use std::io::{self, prelude::*};

pub mod stream;

pub mod bufread;
pub mod read;
pub mod write;

/// Decompress from the given source as if using a [read::XzDecoder].
///
/// Result will be in the xz format.
pub fn decode_all<R: Read>(source: R) -> io::Result<Vec<u8>> {
    let mut vec = Vec::new();
    let mut r = read::XzDecoder::new(source);
    r.read_to_end(&mut vec)?;
    Ok(vec)
}

/// Compress from the given source as if using a [read::XzEncoder].
///
/// The input data must be in the xz format.
pub fn encode_all<R: Read>(source: R, level: u32) -> io::Result<Vec<u8>> {
    let mut vec = Vec::new();
    let mut r = read::XzEncoder::new(source, level);
    r.read_to_end(&mut vec)?;
    Ok(vec)
}

/// Compress all data from the given source as if using a [read::XzEncoder].
///
/// Compressed data will be appended to `destination`.
pub fn copy_encode<R: Read, W: Write>(source: R, mut destination: W, level: u32) -> io::Result<()> {
    io::copy(&mut read::XzEncoder::new(source, level), &mut destination)?;
    Ok(())
}

/// Decompress all data from the given source as if using a [read::XzDecoder].
///
/// Decompressed data will be appended to `destination`.
pub fn copy_decode<R: Read, W: Write>(source: R, mut destination: W) -> io::Result<()> {
    io::copy(&mut read::XzDecoder::new(source), &mut destination)?;
    Ok(())
}

/// Find the size in bytes of uncompressed data from xz file.
#[cfg(feature = "bindgen")]
pub fn uncompressed_size<R: Read + Seek>(mut source: R) -> io::Result<u64> {
    use std::mem::MaybeUninit;
    let mut footer = [0u8; liblzma_sys::LZMA_STREAM_HEADER_SIZE as usize];

    source.seek(io::SeekFrom::End(
        0 - (liblzma_sys::LZMA_STREAM_HEADER_SIZE as i64),
    ))?;
    source.read_exact(&mut footer)?;

    let lzma_stream_flags = unsafe {
        let mut lzma_stream_flags = MaybeUninit::uninit();
        let ret =
            liblzma_sys::lzma_stream_footer_decode(lzma_stream_flags.as_mut_ptr(), footer.as_ptr());

        if ret != liblzma_sys::LZMA_OK {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to parse lzma footer",
            ));
        }

        lzma_stream_flags.assume_init()
    };

    let index_plus_footer =
        liblzma_sys::LZMA_STREAM_HEADER_SIZE as usize + lzma_stream_flags.backward_size as usize;

    source.seek(io::SeekFrom::End(0 - index_plus_footer as i64))?;

    let buf = source
        .bytes()
        .take(index_plus_footer)
        .collect::<io::Result<Vec<u8>>>()?;

    let uncompressed_size = unsafe {
        let mut i: MaybeUninit<*mut liblzma_sys::lzma_index> = MaybeUninit::uninit();
        let mut memlimit = u64::MAX;
        let mut in_pos = 0usize;

        let ret = liblzma_sys::lzma_index_buffer_decode(
            i.as_mut_ptr(),
            &mut memlimit,
            std::ptr::null(),
            buf.as_ptr(),
            &mut in_pos,
            buf.len(),
        );

        if ret != liblzma_sys::LZMA_OK {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to parse lzma footer",
            ));
        }

        let i = i.assume_init();

        let uncompressed_size = liblzma_sys::lzma_index_uncompressed_size(i);

        liblzma_sys::lzma_index_end(i, std::ptr::null());

        uncompressed_size
    };

    Ok(uncompressed_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;
    #[cfg(all(target_family = "wasm", target_os = "unknown"))]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    #[test]
    fn all() {
        quickcheck(test as fn(_) -> _);

        fn test(v: Vec<u8>) -> bool {
            let e = encode_all(&v[..], 6).unwrap();
            let d = decode_all(&e[..]).unwrap();
            v == d
        }
    }

    #[test]
    fn copy() {
        quickcheck(test as fn(_) -> _);

        fn test(v: Vec<u8>) -> bool {
            let mut e = Vec::new();
            copy_encode(&v[..], &mut e, 6).unwrap();
            let mut d = Vec::new();
            copy_decode(&e[..], &mut d).unwrap();
            v == d
        }
    }

    #[test]
    #[cfg(feature = "bindgen")]
    fn size() {
        quickcheck(test as fn(_) -> _);

        fn test(v: Vec<u8>) -> bool {
            let mut e = Vec::new();
            copy_encode(&v[..], &mut e, 6).unwrap();

            let s = super::uncompressed_size(std::io::Cursor::new(e)).unwrap();

            (s as usize) == v.len()
        }
    }
}
