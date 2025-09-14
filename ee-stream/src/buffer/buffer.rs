use std::io::{self, Write};

use crate::buffer::{r_buf::ReaderBuff, w_buf::WriterBuff};

pub struct BufReadWriter<T: io::Write> {
    read_buf: ReaderBuff,
    write_buf: WriterBuff,
    inner: T,
}

/*
impl<T: io::Write> Drop for BufReadWriter<T> {
    fn drop(&mut self) {
        if !self.write_buf.buffer().is_empty() {
            let _ = self.flush();
        }
    }
}
*/
impl<T: io::Write> BufReadWriter<T> {
    pub fn new(inner: T) -> Self {
        Self {
            read_buf: ReaderBuff::new(),
            write_buf: WriterBuff::new(),
            inner,
        }
    }
    pub fn with_capacity(cap: usize, inner: T) -> Self {
        Self {
            read_buf: ReaderBuff::with_capacity(cap),
            write_buf: WriterBuff::with_capacity(cap),
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
    pub fn inner_ref(&self) -> &T {
        &self.inner
    }
    pub fn inner(self) -> T {
        self.inner
    }
    pub fn inner_ref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T: io::Read + io::Write> io::Read for BufReadWriter<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.read_buf.pos == self.read_buf.filled && buf.len() >= self.read_buf.buf.len() {
            self.read_buf.discard_buffer();
            return self.inner.read(buf);
        }
        let rem = std::io::BufRead::fill_buf(self)?;
        let amt = std::cmp::min(rem.len(), buf.len());
        let src = rem.as_ptr();
        let dst = buf.as_mut_ptr();
        unsafe {
            std::ptr::copy_nonoverlapping(src, dst, amt);
        }
        std::io::BufRead::consume(self, amt);
        Ok(amt)
    }
}

impl<T: io::Read + io::Write> std::io::BufRead for BufReadWriter<T> {
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
