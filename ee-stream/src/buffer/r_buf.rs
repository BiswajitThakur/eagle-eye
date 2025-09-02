use std::io;

pub(super) struct ReaderBuff {
    pub(super) buf: Box<[u8]>,
    pub(super) pos: usize,
    pub(super) filled: usize,
}

impl ReaderBuff {
    #[inline]
    pub(super) fn new() -> Self {
        Self::with_capacity(8 * 1024)
    }
    #[inline]
    pub(super) fn with_capacity(size: usize) -> Self {
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
    pub(super) fn fill_buf(&mut self, mut reader: impl io::Read) -> io::Result<&[u8]> {
        if self.pos >= self.filled {
            self.pos = 0;
            self.filled = reader.read(&mut self.buf)?;
        }
        unsafe { Ok(self.buf.get_unchecked(self.pos..self.filled)) }
    }
    #[inline]
    pub(super) fn consume(&mut self, amount: usize) {
        self.pos = std::cmp::min(self.pos + amount, self.filled);
    }
    #[inline]
    pub(super) fn discard_buffer(&mut self) {
        self.pos = 0;
        self.filled = 0;
    }
    #[inline]
    pub(super) fn buffer(&self) -> &[u8] {
        unsafe { self.buf.get_unchecked(self.pos..self.filled) }
    }
    #[inline]
    pub(super) fn buffer_mut(&mut self) -> &mut [u8] {
        unsafe { self.buf.get_unchecked_mut(self.pos..self.filled) }
    }
}
