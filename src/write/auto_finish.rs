use crate::write::{XzDecoder, XzEncoder};
use std::io;
use std::io::Write;

/// A compression stream which will have uncompressed data written to it and
/// will write compressed data to an output stream.
/// [AutoFinishXzEncoder] impl [Drop] trait, so automatically calls [XzEncoder::try_finish] method when exiting the scope.
/// However, it is not guaranteed that `try_finish` will complete successfully, and it is recommended to call `try_finish` manually if you want to ensure that the process is successful.
pub struct AutoFinishXzEncoder<W: Write>(pub(super) XzEncoder<W>);

impl<W: Write> AutoFinishXzEncoder<W> {
    /// Acquires a reference to the underlying writer.
    #[inline]
    pub fn get_ref(&self) -> &W {
        self.0.get_ref()
    }

    /// Acquires a mutable reference to the underlying writer.
    ///
    /// Note that mutating the output/input state of the stream may corrupt this
    /// object, so care must be taken when using this method.
    #[inline]
    pub fn get_mut(&mut self) -> &mut W {
        self.0.get_mut()
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
        self.0.try_finish()
    }

    /// Consumes this encoder, flushing the output stream.
    ///
    /// This will flush the underlying data stream and then return the contained
    /// writer if the flush succeeded.
    ///
    /// Note that this function may not be suitable to call in a situation where
    /// the underlying stream is an asynchronous I/O stream. To finish a stream
    /// the `try_finish` method should be used instead. To
    /// re-acquire ownership of a stream it is safe to call this method after
    /// `try_finish` has returned `Ok`.
    #[inline]
    pub fn finish(mut self) -> io::Result<W> {
        self.try_finish()?;
        Ok(self.0.obj.take().unwrap())
    }

    /// Returns the number of bytes produced by the compressor
    ///
    /// Note that, due to buffering, this only bears any relation to
    /// `total_in()` after a call to `flush()`.  At that point,
    /// `total_out() / total_in()` is the compression ratio.
    #[inline]
    pub fn total_out(&self) -> u64 {
        self.0.total_out()
    }

    /// Returns the number of bytes consumed by the compressor
    /// (e.g. the number of bytes written to this stream.)
    #[inline]
    pub fn total_in(&self) -> u64 {
        self.0.total_out()
    }
}

impl<W: Write> Write for AutoFinishXzEncoder<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl<W: Write> Drop for AutoFinishXzEncoder<W> {
    #[inline]
    fn drop(&mut self) {
        if self.0.obj.is_some() {
            let _ = self.0.try_finish();
        }
    }
}

/// A compression stream which will have compressed data written to it and
/// will write uncompressed data to an output stream.
/// [AutoFinishXzDecoder] impl [Drop] trait, so automatically calls [XzDecoder::try_finish] method when exiting the scope.
/// However, it is not guaranteed that `try_finish` will complete successfully, and it is recommended to call `try_finish` manually if you want to ensure that the process is successful.
pub struct AutoFinishXzDecoder<W: Write>(pub(super) XzDecoder<W>);

impl<W: Write> AutoFinishXzDecoder<W> {
    /// Acquires a reference to the underlying writer.
    #[inline]
    pub fn get_ref(&self) -> &W {
        self.0.get_ref()
    }

    /// Acquires a mutable reference to the underlying writer.
    ///
    /// Note that mutating the output/input state of the stream may corrupt this
    /// object, so care must be taken when using this method.
    #[inline]
    pub fn get_mut(&mut self) -> &mut W {
        self.0.get_mut()
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
        self.0.try_finish()
    }

    /// Consumes this decoder, flushing the output stream.
    ///
    /// This will flush the underlying data stream and then return the contained
    /// writer if the flush succeeded.
    ///
    /// Note that this function may not be suitable to call in a situation where
    /// the underlying stream is an asynchronous I/O stream. To finish a stream
    /// the `try_finish` method should be used instead. To
    /// re-acquire ownership of a stream it is safe to call this method after
    /// `try_finish` has returned `Ok`.
    #[inline]
    pub fn finish(mut self) -> io::Result<W> {
        self.try_finish()?;
        Ok(self.0.obj.take().unwrap())
    }

    /// Returns the number of bytes produced by the decompressor
    ///
    /// Note that, due to buffering, this only bears any relation to
    /// `total_in()` after a call to `flush()`.  At that point,
    /// `total_in() / total_out()` is the compression ratio.
    #[inline]
    pub fn total_out(&self) -> u64 {
        self.0.total_out()
    }

    /// Returns the number of bytes consumed by the decompressor
    /// (e.g. the number of bytes written to this stream.)
    #[inline]
    pub fn total_in(&self) -> u64 {
        self.0.total_in()
    }
}

impl<W: Write> Write for AutoFinishXzDecoder<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl<W: Write> Drop for AutoFinishXzDecoder<W> {
    #[inline]
    fn drop(&mut self) {
        if self.0.obj.is_some() {
            let _ = self.0.try_finish();
        }
    }
}
