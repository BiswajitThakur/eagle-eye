use std::io;

pub struct BufReadWriter<T> {
    read_buf: ReaderBuff,
    write_buf: WriterBuff,
    inner: T,
}

impl<T> BufReadWriter<T> {
    pub fn new(inner: T) -> Self {
        Self {
            read_buf: ReaderBuff::new(),
            write_buf: WriterBuff::new(),
            inner,
        }
    }
    pub fn with_read_write_capacity(r: usize, w: usize, inner: T) -> Self {
        Self {
            read_buf: ReaderBuff::with_capacity(r),
            write_buf: WriterBuff::with_capacity(w),
            inner,
        }
    }
    pub fn with_read_capacity(cap: usize, inner: T) -> Self {
        Self {
            read_buf: ReaderBuff::with_capacity(cap),
            write_buf: WriterBuff::new(),
            inner,
        }
    }
    pub fn with_write_capacity(cap: usize, inner: T) -> Self {
        Self {
            read_buf: ReaderBuff::new(),
            write_buf: WriterBuff::with_capacity(cap),
            inner,
        }
    }
    pub fn read_buffer(&self) -> &[u8] {
        self.read_buf.buffer()
    }
    pub fn read_buffer_mut(&mut self) -> &mut [u8] {
        self.read_buf.buffer_mut()
    }
    pub fn write_buffer(&self) -> &[u8] {
        self.write_buf.buffer()
    }
    pub fn write_buffer_mut(&mut self) -> &mut [u8] {
        self.write_buf.buffer_mut()
    }
}
struct ReaderBuff {
    buf: Box<[u8]>,
    pos: usize,
    filled: usize,
}

impl ReaderBuff {
    #[inline]
    fn new() -> Self {
        Self::with_capacity(8 * 1024)
    }
    #[inline]
    fn with_capacity(size: usize) -> Self {
        assert!(size != 0, "capacity can not be zero");
        let v = Box::<[u8]>::new_uninit_slice(size);
        let buf = unsafe { v.assume_init() };
        Self {
            buf,
            pos: 0,
            filled: 0,
        }
    }
    #[inline]
    fn fill_buf(&mut self, mut reader: impl io::Read) -> io::Result<&[u8]> {
        if self.pos >= self.filled {
            self.pos = 0;
            self.filled = reader.read(&mut self.buf)?;
        }
        unsafe { Ok(self.buf.get_unchecked(self.pos..self.filled)) }
    }
    #[inline]
    fn consume(&mut self, amount: usize) {
        self.pos = std::cmp::min(self.pos + amount, self.filled);
    }
    #[inline]
    fn discard_buffer(&mut self) {
        self.pos = 0;
        self.filled = 0;
    }
    #[inline]
    fn buffer(&self) -> &[u8] {
        unsafe { self.buf.get_unchecked(self.pos..self.filled) }
    }
    #[inline]
    fn buffer_mut(&mut self) -> &mut [u8] {
        unsafe { self.buf.get_unchecked_mut(self.pos..self.filled) }
    }
}

struct WriterBuff {
    buf: Vec<u8>,
}

impl WriterBuff {
    fn new() -> Self {
        Self::with_capacity(8 * 1024)
    }
    fn with_capacity(cap: usize) -> Self {
        assert!(cap != 0, "capacity can not be zero");
        Self {
            buf: Vec::with_capacity(cap),
        }
    }
    fn buffer(&self) -> &[u8] {
        &self.buf
    }
    fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buf
    }
    fn flush_buf(&mut self, mut writer: impl io::Write) -> io::Result<()> {
        struct BufGuard<'a> {
            buffer: &'a mut Vec<u8>,
            written: usize,
        }
        impl<'a> BufGuard<'a> {
            fn new(buffer: &'a mut Vec<u8>) -> Self {
                Self { buffer, written: 0 }
            }

            /// The unwritten part of the buffer
            fn remaining(&self) -> &[u8] {
                &self.buffer[self.written..]
            }

            /// Flag some bytes as removed from the front of the buffer
            fn consume(&mut self, amt: usize) {
                self.written += amt;
            }

            /// true if all of the bytes have been written
            fn done(&self) -> bool {
                self.written >= self.buffer.len()
            }
        }

        impl Drop for BufGuard<'_> {
            fn drop(&mut self) {
                if self.written > 0 {
                    self.buffer.drain(..self.written);
                }
            }
        }
        let mut guard = BufGuard::new(&mut self.buf);
        while !guard.done() {
            let r = writer.write(guard.remaining());
            match r {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "failed to write the buffered data",
                    ));
                }
                Ok(n) => guard.consume(n),
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
    fn spare_capacity(&self) -> usize {
        self.buf.capacity() - self.buf.len()
    }
    unsafe fn write_to_buffer_unchecked(&mut self, buf: &[u8]) {
        let old_len = self.buf.len();
        let buf_len = buf.len();
        let src = buf.as_ptr();
        unsafe {
            let dst = self.buf.as_mut_ptr().add(old_len);
            std::ptr::copy_nonoverlapping(src, dst, buf_len);
            self.buf.set_len(old_len + buf_len);
        }
    }
    fn write_to_buf(&mut self, buf: &[u8]) -> usize {
        let available = self.spare_capacity();
        let amt_to_buffer = available.min(buf.len());

        unsafe {
            self.write_to_buffer_unchecked(&buf[..amt_to_buffer]);
        }
        amt_to_buffer
    }
    #[cold]
    #[inline(never)]
    fn write_cold(&mut self, buf: &[u8], mut writer: impl io::Write) -> io::Result<usize> {
        if buf.len() > self.spare_capacity() {
            self.flush_buf(&mut writer)?;
        }

        if buf.len() >= self.buf.capacity() {
            writer.write(buf)
        } else {
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }
            Ok(buf.len())
        }
    }
    #[inline(never)]
    fn write_all_cold(&mut self, buf: &[u8], mut writer: impl io::Write) -> io::Result<()> {
        if buf.len() > self.spare_capacity() {
            self.flush_buf(&mut writer)?;
        }
        if buf.len() >= self.buf.capacity() {
            writer.write_all(buf)
        } else {
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }

            Ok(())
        }
    }
}

impl<T: io::Read> io::Read for BufReadWriter<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.read_buf.pos == self.read_buf.filled && buf.len() >= self.read_buf.buf.len() {
            self.read_buf.discard_buffer();
            return self.inner.read(buf);
        }
        let mut rem = std::io::BufRead::fill_buf(self)?;
        let nread = rem.read(buf)?;
        std::io::BufRead::consume(self, nread);
        Ok(nread)
    }
}

impl<T: io::Read> std::io::BufRead for BufReadWriter<T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.read_buf.fill_buf(&mut self.inner)
    }
    fn consume(&mut self, amount: usize) {
        self.read_buf.consume(amount)
    }
}

impl<T: io::Write> io::Write for BufReadWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() < self.write_buf.spare_capacity() {
            unsafe {
                self.write_buf.write_to_buffer_unchecked(buf);
            }

            Ok(buf.len())
        } else {
            self.write_buf.write_cold(buf, &mut self.inner)
        }
    }
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        if buf.len() < self.write_buf.spare_capacity() {
            unsafe {
                self.write_buf.write_to_buffer_unchecked(buf);
            }

            Ok(())
        } else {
            self.write_buf.write_all_cold(buf, &mut self.inner)
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        self.write_buf
            .flush_buf(&mut self.inner)
            .and_then(|()| self.inner.flush())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    use crate::buffer::BufReadWriter;

    #[test]
    fn test_buf_read_writer_read_1() {
        let mut buf = [0; 2];
        let mut reader = BufReadWriter::with_read_capacity(8, Cursor::new("hello world.a.b.z"));
        assert_eq!(reader.read_buffer(), &[]);

        assert_eq!(reader.read(&mut buf[0..1]).unwrap(), 1);
        assert_eq!(buf[0], b'h');
        assert_eq!(reader.read_buffer(), b"ello wo");

        assert_eq!(reader.read(&mut buf).unwrap(), 2);
        assert_eq!(&buf, b"el");
        assert_eq!(reader.read_buffer(), b"lo wo");

        assert_eq!(reader.read(&mut buf).unwrap(), 2);
        assert_eq!(&buf, b"lo");
        assert_eq!(reader.read_buffer(), b" wo");

        assert_eq!(reader.read(&mut buf).unwrap(), 2);
        assert_eq!(&buf, b" w");
        assert_eq!(reader.read_buffer(), b"o");

        assert_eq!(reader.read(&mut buf).unwrap(), 1);
        assert_eq!(&buf[0..1], &*b"o");
        assert_eq!(reader.read_buffer(), b"");

        assert_eq!(reader.read(&mut buf).unwrap(), 2);
        assert_eq!(&buf, b"rl");
        assert_eq!(reader.read_buffer(), b"d.a.b.");

        assert_eq!(reader.read(&mut buf).unwrap(), 2);
        assert_eq!(&buf, b"d.");
        assert_eq!(reader.read_buffer(), b"a.b.");

        assert_eq!(reader.read(&mut buf).unwrap(), 2);
        assert_eq!(&buf, b"a.");
        assert_eq!(reader.read_buffer(), b"b.");

        assert_eq!(reader.read(&mut buf).unwrap(), 2);
        assert_eq!(&buf, b"b.");
        assert_eq!(reader.read_buffer(), b"");

        assert_eq!(reader.read(&mut buf).unwrap(), 1);
        assert_eq!(&buf[0..1], &*b"z");
        assert_eq!(reader.read_buffer(), b"");

        assert_eq!(reader.read(&mut buf).unwrap(), 0);
        assert_eq!(reader.read_buffer(), b"");

        assert_eq!(reader.read(&mut buf).unwrap(), 0);
        assert_eq!(reader.read_buffer(), b"");
    }

    #[test]
    fn test_buf_read_writer_read_2() {
        let mut buf = [0; 9];
        let mut reader = BufReadWriter::with_read_capacity(8, Cursor::new("hello world"));
        assert_eq!(reader.read_buffer(), &[]);

        assert_eq!(reader.read(&mut buf).unwrap(), 9);
        assert_eq!(&buf, b"hello wor");
        assert_eq!(reader.read_buffer(), &[]);

        assert_eq!(reader.read(&mut buf).unwrap(), 2);
        assert_eq!(&buf[0..2], b"ld");
        assert_eq!(reader.read_buffer(), &[]);

        assert_eq!(reader.read(&mut buf).unwrap(), 0);
        assert_eq!(reader.read_buffer(), &[]);
    }

    #[test]
    fn test_buf_read_writer_read_3() {
        let mut buf = [0; 9];
        let mut reader = BufReadWriter::with_read_capacity(8, Cursor::new("hello world"));

        assert_eq!(reader.read(&mut buf[0..1]).unwrap(), 1);
        assert_eq!(buf[0], b'h');
        assert_eq!(reader.read_buffer(), b"ello wo");

        assert_eq!(reader.read(&mut buf).unwrap(), 7);
        assert_eq!(&buf[0..7], b"ello wo");
        assert_eq!(reader.read_buffer(), &[]);

        assert_eq!(reader.read(&mut buf).unwrap(), 3);
        assert_eq!(&buf[0..3], b"rld");
        assert_eq!(reader.read_buffer(), &[]);

        assert_eq!(reader.read(&mut buf).unwrap(), 0);
        assert_eq!(reader.read_buffer(), &[]);
    }
}
