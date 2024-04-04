use std::io::{self, BufRead, Read};

/// Size of buffer in bytes
const BUFFER_SIZE: usize = 8 * 1024;

/// [std::io::BufReader](io::BufReader) alternative
pub(crate) struct BufReader<R> {
    inner: R,
    buf: [u8; BUFFER_SIZE],
    /// current read position
    pos: usize,
    /// end of data in buffer
    cap: usize,
}

impl<R> BufReader<R> {
    #[inline]
    pub fn new(inner: R) -> BufReader<R> {
        BufReader {
            inner,
            buf: [0u8; BUFFER_SIZE],
            pos: 0,
            cap: 0,
        }
    }

    #[inline]
    pub fn into_inner(self) -> R {
        self.inner
    }

    #[inline]
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }
}
impl<R: Read> BufReader<R> {
    /// internal fill buffer method
    fn fill_buf(&mut self) -> io::Result<()> {
        if self.pos >= self.cap {
            self.cap = self.inner.read(&mut self.buf)?;
            self.pos = 0; // restart to head of buffer
        }
        Ok(())
    }
}

impl<R: Read> Read for BufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.fill_buf()?;
        if self.pos >= self.cap {
            // reached to eof
            return Ok(0);
        }

        let amt = std::cmp::min(buf.len(), self.cap - self.pos);
        buf[..amt].copy_from_slice(&self.buf[self.pos..self.pos + amt]);
        self.pos += amt;
        Ok(amt)
    }
}

impl<R: Read> BufRead for BufReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.fill_buf()?;
        Ok(&self.buf[self.pos..self.cap])
    }

    fn consume(&mut self, amt: usize) {
        self.pos = std::cmp::min(self.pos + amt, self.cap);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;
    use std::io::Cursor;

    #[test]
    fn read_small_buffer() {
        let data = b"Hello, world!";
        let cursor = Cursor::new(data.as_ref());

        let mut buf_reader = BufReader::new(cursor);

        let mut buffer = [0; 5];

        let n = buf_reader.read(&mut buffer).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&buffer, b"Hello");

        let n = buf_reader.read(&mut buffer).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&buffer[..5], b", wor");

        let n = buf_reader.read(&mut buffer).unwrap();
        assert_eq!(n, 3);
        assert_eq!(&buffer[..3], b"ld!");
    }

    #[test]
    fn bufread() {
        quickcheck(test as fn(_) -> _);

        fn test(v: Vec<u8>) -> bool {
            let mut e = Vec::new();
            let mut reader = BufReader::new(&v[..]);
            reader.read_to_end(&mut e).unwrap();
            v == e
        }
    }
}
