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
//! liblzma = { version = "0.2.0", features = ["static"] }
//! ```
//!
//! # Multithreading
//!
//! This crate optionally can support multithreading using the `parallel`
//! feature of this crate:
//!
//! ```toml
//! liblzma = { version = "0.2.0", features = ["parallel"] }
//! ```
//!
//! # Async I/O
//!
//! This crate optionally can support async I/O streams with the Tokio stack via
//! the `tokio` feature of this crate:
//!
//! ```toml
//! liblzma = { version = "0.2.0", features = ["tokio"] }
//! ```
//!
//! All methods are internally capable of working with streams that may return
//! `ErrorKind::WouldBlock` when they're not ready to perform the particular
//! operation.
//!
//! Note that care needs to be taken when using these objects, however. The
//! Tokio runtime, in particular, requires that data is fully flushed before
//! dropping streams. For compatibility with blocking streams all streams are
//! flushed/written when they are dropped, and this is not always a suitable
//! time to perform I/O. If I/O streams are flushed before drop, however, then
//! these operations will be a noop.

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

/// Compress all data from the given source as if using an [read::XzEncoder].
///
/// Compressed data will be appended to `destination`.
pub fn copy_encode<R: Read, W: Write>(source: R, mut destination: W, level: u32) -> io::Result<()> {
    io::copy(&mut read::XzEncoder::new(source, level), &mut destination)?;
    Ok(())
}

/// Decompress all data from the given source as if using an [read::XzDecoder].
///
/// Decompressed data will be appended to `destination`.
pub fn copy_decode<R: Read, W: Write>(source: R, mut destination: W) -> io::Result<()> {
    io::copy(&mut read::XzDecoder::new(source), &mut destination)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;

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
}
