use std::io;

pub(super) struct WriterBuff {
    pub(super) buf: Vec<u8>,
}

impl WriterBuff {
    pub(super) fn new() -> Self {
        Self::with_capacity(8 * 1024)
    }
    pub(super) fn with_capacity(cap: usize) -> Self {
        assert!(cap != 0, "capacity can not be zero");
        Self {
            buf: Vec::with_capacity(cap),
        }
    }
    pub(super) fn buffer(&self) -> &[u8] {
        &self.buf
    }
    pub(super) fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buf
    }
    pub(super) fn flush_buf(&mut self, mut writer: impl io::Write) -> io::Result<()> {
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
    pub(super) fn spare_capacity(&self) -> usize {
        self.buf.capacity() - self.buf.len()
    }
    pub(super) unsafe fn write_to_buffer_unchecked(&mut self, buf: &[u8]) {
        let old_len = self.buf.len();
        let buf_len = buf.len();
        let src = buf.as_ptr();
        unsafe {
            let dst = self.buf.as_mut_ptr().add(old_len);
            std::ptr::copy_nonoverlapping(src, dst, buf_len);
            self.buf.set_len(old_len + buf_len);
        }
    }
    pub(super) fn write_to_buf(&mut self, buf: &[u8]) -> usize {
        let available = self.spare_capacity();
        let amt_to_buffer = available.min(buf.len());

        unsafe {
            self.write_to_buffer_unchecked(&buf[..amt_to_buffer]);
        }
        amt_to_buffer
    }
    #[cold]
    #[inline(never)]
    pub(super) fn write_cold(
        &mut self,
        buf: &[u8],
        mut writer: impl io::Write,
    ) -> io::Result<usize> {
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
    pub(super) fn write_all_cold(
        &mut self,
        buf: &[u8],
        mut writer: impl io::Write,
    ) -> io::Result<()> {
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
