use std::io;

struct ReaderBuff {
    buf: Box<[u8]>,
    pos: usize,
    filled: usize,
}

struct WriterBuff {}

impl ReaderBuff {
    fn fill_buf(&mut self, mut reader: impl io::Read) -> io::Result<&[u8]> {
        if self.pos >= self.filled {
            self.pos = 0;
            self.filled = reader.read(&mut self.buf)?;
        }
        unsafe { Ok(self.buf.get_unchecked(self.pos..self.filled)) }
    }
    fn consume(&mut self, amount: usize) {
        self.pos = std::cmp::min(self.pos + amount, self.filled);
    }
    #[inline]
    pub fn discard_buffer(&mut self) {
        self.pos = 0;
        self.filled = 0;
    }
}

pub struct BufReadWriter<T> {
    read_buf: ReaderBuff,
    write_buf: WriterBuff,
    inner: T,
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
        // let w = std::io::BufWriter::new(std::fs::File::open("hello").unwrap());
        // w.write_all(b"");
        todo!()
    }
    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}
