//! Writer-based compression/decompression streams

mod auto_finish;

use std::io;
use std::io::prelude::*;

#[cfg(feature = "parallel")]
use crate::stream::MtStreamBuilder;
use crate::stream::{Action, Check, Status, Stream};
pub use auto_finish::{AutoFinishXzDecoder, AutoFinishXzEncoder};

/// A compression stream which will have uncompressed data written to it and
/// will write compressed data to an output stream.
/// [XzEncoder] will no longer perform the finalization automatically in the next miner release, so you need to call [XzEncoder::finish] manually.
/// If you want to automate the finalization process, please use [XzEncoder::auto_finish].
pub struct XzEncoder<W: Write> {
    data: Stream,
    obj: Option<W>,
    buf: Vec<u8>,
}

/// A compression stream which will have compressed data written to it and
/// will write uncompressed data to an output stream.
/// [XzDecoder] will no longer perform the finalization automatically in the next miner release, so you need to call [XzDecoder::finish] manually.
/// If you want to automate the finalization process, please use [XzDecoder::auto_finish].
pub struct XzDecoder<W: Write> {
    data: Stream,
    obj: Option<W>,
    buf: Vec<u8>,
}

impl<W: Write> XzEncoder<W> {
    /// Create a new compression stream which will compress at the given level
    /// to write compress output to the give output stream.
    #[inline]
    pub fn new(obj: W, level: u32) -> XzEncoder<W> {
        let stream = Stream::new_easy_encoder(level, Check::Crc64).unwrap();
        XzEncoder::new_stream(obj, stream)
    }
    /// Create a new parallel compression stream which will compress at the given level
    /// to write compress output to the give output stream.
    #[cfg(feature = "parallel")]
    pub fn new_parallel(obj: W, level: u32) -> XzEncoder<W> {
        let stream = MtStreamBuilder::new()
            .preset(level)
            .check(Check::Crc64)
            .threads(num_cpus::get() as u32)
            .encoder()
            .unwrap();
        Self::new_stream(obj, stream)
    }

    /// Create a new encoder which will use the specified `Stream` to encode
    /// (compress) data into the provided `obj`.
    #[inline]
    pub fn new_stream(obj: W, stream: Stream) -> XzEncoder<W> {
        XzEncoder {
            data: stream,
            obj: Some(obj),
            buf: Vec::with_capacity(32 * 1024),
        }
    }

    /// Acquires a reference to the underlying writer.
    #[inline]
    pub fn get_ref(&self) -> &W {
        self.obj.as_ref().unwrap()
    }

    /// Acquires a mutable reference to the underlying writer.
    ///
    /// Note that mutating the output/input state of the stream may corrupt this
    /// object, so care must be taken when using this method.
    #[inline]
    pub fn get_mut(&mut self) -> &mut W {
        self.obj.as_mut().unwrap()
    }

    fn dump(&mut self) -> io::Result<()> {
        self.obj.as_mut().unwrap().write_all(&self.buf)?;
        self.buf.clear();
        Ok(())
    }

    /// Attempt to finish this output stream, writing out final chunks of data.
    ///
    /// Note that this function can only be used once data has finished being
    /// written to the output stream. After this function is called then further
    /// calls to `write` may result in a panic.
    ///
    /// # Panics
    ///
    /// Attempts to write data to this stream may result in a panic after this
    /// function is called.
    #[inline]
    pub fn try_finish(&mut self) -> io::Result<()> {
        loop {
            self.dump()?;
            let res = self.data.process_vec(&[], &mut self.buf, Action::Finish)?;
            if res == Status::StreamEnd {
                break;
            }
        }
        self.dump()
    }

    /// Consumes this encoder, finishing the compression stream.
    ///
    /// This will finish the underlying data stream and then return the contained
    /// writer if the finish succeeded.
    ///
    /// Note that this function may not be suitable to call in a situation where
    /// the underlying stream is an asynchronous I/O stream. To finish a stream
    /// the `try_finish` (or `shutdown`) method should be used instead. To
    /// re-acquire ownership of a stream it is safe to call this method after
    /// `try_finish` or `shutdown` has returned `Ok`.
    #[inline]
    pub fn finish(mut self) -> io::Result<W> {
        self.try_finish()?;
        Ok(self.obj.take().unwrap())
    }

    /// Returns the number of bytes produced by the compressor
    ///
    /// Note that, due to buffering, this only bears any relation to
    /// `total_in()` after a call to `flush()`.  At that point,
    /// `total_out() / total_in()` is the compression ratio.
    #[inline]
    pub fn total_out(&self) -> u64 {
        self.data.total_out()
    }

    /// Returns the number of bytes consumed by the compressor
    /// (e.g. the number of bytes written to this stream.)
    #[inline]
    pub fn total_in(&self) -> u64 {
        self.data.total_in()
    }

    /// Convert to [AutoFinishXzEncoder] that impl [Drop] trait.
    /// [AutoFinishXzEncoder] automatically calls [XzDecoder::try_finish] method when exiting the scope.
    #[inline]
    pub fn auto_finish(self) -> AutoFinishXzEncoder<W> {
        AutoFinishXzEncoder(self)
    }
}

impl<W: Write> Write for XzEncoder<W> {
    #[inline]
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        loop {
            self.dump()?;

            let total_in = self.total_in();
            self.data.process_vec(data, &mut self.buf, Action::Run)?;
            let written = (self.total_in() - total_in) as usize;

            if written > 0 || data.is_empty() {
                return Ok(written);
            }
        }
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        loop {
            self.dump()?;
            let status = self
                .data
                .process_vec(&[], &mut self.buf, Action::FullFlush)?;
            if status == Status::StreamEnd {
                break;
            }
        }
        self.obj.as_mut().unwrap().flush()
    }
}

impl<W: Read + Write> Read for XzEncoder<W> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.get_mut().read(buf)
    }
}

impl<W: Write> Drop for XzEncoder<W> {
    #[inline]
    fn drop(&mut self) {
        if self.obj.is_some() {
            let _ = self.try_finish();
        }
    }
}

impl<W: Write> XzDecoder<W> {
    /// Creates a new decoding stream which will decode into `obj` one xz stream
    /// from the input written to it.
    #[inline]
    pub fn new(obj: W) -> XzDecoder<W> {
        let stream = Stream::new_stream_decoder(u64::MAX, 0).unwrap();
        XzDecoder::new_stream(obj, stream)
    }

    /// Creates a new parallel decoding stream which will decode into `obj` one xz stream
    /// from the input written to it.
    #[cfg(feature = "parallel")]
    pub fn new_parallel(obj: W) -> Self {
        let stream = MtStreamBuilder::new()
            .memlimit_stop(u64::MAX)
            .threads(num_cpus::get() as u32)
            .decoder()
            .unwrap();
        Self::new_stream(obj, stream)
    }

    /// Creates a new decoding stream which will decode into `obj` all the xz streams
    /// from the input written to it.
    #[inline]
    pub fn new_multi_decoder(obj: W) -> XzDecoder<W> {
        let stream = Stream::new_stream_decoder(u64::MAX, liblzma_sys::LZMA_CONCATENATED).unwrap();
        XzDecoder::new_stream(obj, stream)
    }

    /// Creates a new decoding stream which will decode all input written to it
    /// into `obj`.
    ///
    /// A custom `stream` can be specified to configure what format this decoder
    /// will recognize or configure other various decoding options.
    #[inline]
    pub fn new_stream(obj: W, stream: Stream) -> XzDecoder<W> {
        XzDecoder {
            data: stream,
            obj: Some(obj),
            buf: Vec::with_capacity(32 * 1024),
        }
    }

    /// Acquires a reference to the underlying writer.
    #[inline]
    pub fn get_ref(&self) -> &W {
        self.obj.as_ref().unwrap()
    }

    /// Acquires a mutable reference to the underlying writer.
    ///
    /// Note that mutating the output/input state of the stream may corrupt this
    /// object, so care must be taken when using this method.
    #[inline]
    pub fn get_mut(&mut self) -> &mut W {
        self.obj.as_mut().unwrap()
    }

    fn dump(&mut self) -> io::Result<()> {
        self.obj.as_mut().unwrap().write_all(&self.buf)?;
        self.buf.clear();
        Ok(())
    }

    /// Attempt to finish this output stream, writing out final chunks of data.
    ///
    /// Note that this function can only be used once data has finished being
    /// written to the output stream. After this function is called then further
    /// calls to `write` may result in a panic.
    ///
    /// # Panics
    ///
    /// Attempts to write data to this stream may result in a panic after this
    /// function is called.
    #[inline]
    pub fn try_finish(&mut self) -> io::Result<()> {
        loop {
            self.dump()?;
            let res = self.data.process_vec(&[], &mut self.buf, Action::Finish)?;

            // When decoding a truncated file, XZ returns LZMA_BUF_ERROR and
            // decodes no new data, which corresponds to this crate's MemNeeded
            // status.  Since we're finishing, we cannot provide more data so
            // this is an error.
            //
            // See the 02_decompress.c example in xz-utils.
            if self.buf.is_empty() && res == Status::MemNeeded {
                let msg = "xz compressed stream is truncated or otherwise corrupt";
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, msg));
            }

            if res == Status::StreamEnd {
                break;
            }
        }
        self.dump()
    }

    /// Consumes this decoder, finishing the decompression stream.
    ///
    /// This will finish the underlying data stream and then return the contained
    /// writer if the finish succeeded.
    ///
    /// Note that this function may not be suitable to call in a situation where
    /// the underlying stream is an asynchronous I/O stream. To finish a stream
    /// the `try_finish` (or `shutdown`) method should be used instead. To
    /// re-acquire ownership of a stream it is safe to call this method after
    /// `try_finish` or `shutdown` has returned `Ok`.
    #[inline]
    pub fn finish(mut self) -> io::Result<W> {
        self.try_finish()?;
        Ok(self.obj.take().unwrap())
    }

    /// Returns the number of bytes produced by the decompressor
    ///
    /// Note that, due to buffering, this only bears any relation to
    /// `total_in()` after a call to `flush()`.  At that point,
    /// `total_in() / total_out()` is the compression ratio.
    #[inline]
    pub fn total_out(&self) -> u64 {
        self.data.total_out()
    }

    /// Returns the number of bytes consumed by the decompressor
    /// (e.g. the number of bytes written to this stream.)
    #[inline]
    pub fn total_in(&self) -> u64 {
        self.data.total_in()
    }

    /// Convert to [AutoFinishXzDecoder] that impl [Drop] trait.
    /// [AutoFinishXzDecoder] automatically calls [XzDecoder::try_finish] method when exiting the scope.
    #[inline]
    pub fn auto_finish(self) -> AutoFinishXzDecoder<W> {
        AutoFinishXzDecoder(self)
    }
}

impl<W: Write> Write for XzDecoder<W> {
    #[inline]
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        loop {
            self.dump()?;

            let before = self.total_in();
            let res = self.data.process_vec(data, &mut self.buf, Action::Run)?;
            let written = (self.total_in() - before) as usize;

            if written > 0 || data.is_empty() || res == Status::StreamEnd {
                return Ok(written);
            }
        }
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.dump()?;
        self.obj.as_mut().unwrap().flush()
    }
}

impl<W: Read + Write> Read for XzDecoder<W> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.get_mut().read(buf)
    }
}

impl<W: Write> Drop for XzDecoder<W> {
    #[inline]
    fn drop(&mut self) {
        if self.obj.is_some() {
            let _ = self.try_finish();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::LzmaOptions;
    use quickcheck::quickcheck;
    use std::iter::repeat;
    #[cfg(all(target_family = "wasm", target_os = "unknown"))]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    #[test]
    fn smoke() {
        let d = XzDecoder::new(Vec::new());
        let mut c = XzEncoder::new(d, 6);
        c.write_all(b"12834").unwrap();
        let s = repeat("12345").take(100000).collect::<String>();
        c.write_all(s.as_bytes()).unwrap();
        let data = c.finish().unwrap().finish().unwrap();
        assert_eq!(&data[0..5], b"12834");
        assert_eq!(data.len(), 500005);
        assert_eq!(format!("12834{}", s).as_bytes(), &*data);
    }

    #[test]
    fn write_empty() {
        let d = XzDecoder::new(Vec::new());
        let mut c = XzEncoder::new(d, 6);
        c.write(b"").unwrap();
        let data = c.finish().unwrap().finish().unwrap();
        assert_eq!(&data[..], b"");
    }

    #[test]
    fn qc_lzma1() {
        quickcheck(test as fn(_) -> _);

        fn test(v: Vec<u8>) -> bool {
            let stream = Stream::new_lzma_decoder(u64::MAX).unwrap();
            let w = XzDecoder::new_stream(Vec::new(), stream);
            let options = LzmaOptions::new_preset(6).unwrap();
            let stream = Stream::new_lzma_encoder(&options).unwrap();
            let mut w = XzEncoder::new_stream(w, stream);
            w.write_all(&v).unwrap();
            v == w.finish().unwrap().finish().unwrap()
        }
    }

    #[test]
    fn qc() {
        quickcheck(test as fn(_) -> _);

        fn test(v: Vec<u8>) -> bool {
            let w = XzDecoder::new(Vec::new());
            let mut w = XzEncoder::new(w, 6);
            w.write_all(&v).unwrap();
            v == w.finish().unwrap().finish().unwrap()
        }
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn qc_parallel_encode() {
        quickcheck(test as fn(_) -> _);

        fn test(v: Vec<u8>) -> bool {
            let w = XzDecoder::new(Vec::new());
            let mut w = XzEncoder::new_parallel(w, 6);
            w.write_all(&v).unwrap();
            v == w.finish().unwrap().finish().unwrap()
        }
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn qc_parallel_decode() {
        quickcheck(test as fn(_) -> _);

        fn test(v: Vec<u8>) -> bool {
            let w = XzDecoder::new_parallel(Vec::new());
            let mut w = XzEncoder::new(w, 6);
            w.write_all(&v).unwrap();
            v == w.finish().unwrap().finish().unwrap()
        }
    }
}
