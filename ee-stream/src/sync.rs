use std::io::{self, BufReader, BufWriter};

use aes::cipher::StreamCipher;

pub struct EStreamSync<const N: usize, R: io::Read, W: io::Write> {
    read_cipher: ctr::Ctr64LE<aes::Aes256>,
    write_cipher: ctr::Ctr64LE<aes::Aes256>,
    write_buff: [u8; N],
    reader: BufReader<R>,
    writer: BufWriter<W>,
}

// TODO: remove me
#[cfg(debug_assertions)]
impl<const N: usize, R: io::Read, W: io::Write> Drop for EStreamSync<N, R, W> {
    fn drop(&mut self) {
        println!("....EStream Droped......");
    }
}

impl<const N: usize, R: io::Read, W: io::Write> io::Read for EStreamSync<N, R, W> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.reader.read(buf)?;
        self.read_cipher.apply_keystream(&mut buf[0..n]);
        Ok(n)
    }
}

impl<const N: usize, R: io::Read, W: io::Write> io::Write for EStreamSync<N, R, W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for chunk in buf.chunks(N) {
            self.write_cipher
                .apply_keystream_b2b(chunk, &mut self.write_buff[0..chunk.len()])
                .unwrap();
            self.writer.write(&self.write_buff[0..chunk.len()])?;
        }
        Ok(buf.len())
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<const N: usize, R: io::Read, W: io::Write> io::BufRead for EStreamSync<N, R, W> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.reader.fill_buf()
    }
    #[inline]
    fn consume(&mut self, amt: usize) {
        self.reader.consume(amt);
    }
}

impl<const N: usize, R: io::Read, W: io::Write> EStreamSync<N, R, W> {
    #[inline]
    pub fn get_ref_reader(&self) -> &R {
        self.reader.get_ref()
    }
    #[inline]
    pub fn get_mut_reader(&mut self) -> &mut R {
        self.reader.get_mut()
    }
    #[inline]
    pub fn get_ref_writer(&self) -> &W {
        self.writer.get_ref()
    }
    #[inline]
    pub fn get_mut_writer(&mut self) -> &mut W {
        self.writer.get_mut()
    }
    #[inline]
    pub fn builder() -> EStreamBuilderSync<N, R, W> {
        EStreamBuilderSync {
            cipher: None,
            buffer: None,
            reader: None,
            writer: None,
        }
    }
}

pub struct EStreamBuilderSync<const N: usize, R: io::Read, W: io::Write> {
    cipher: Option<ctr::Ctr64LE<aes::Aes256>>,
    buffer: Option<[u8; N]>,
    reader: Option<BufReader<R>>,
    writer: Option<BufWriter<W>>,
}

impl<const N: usize, R: io::Read, W: io::Write> EStreamBuilderSync<N, R, W> {
    pub fn cipher(mut self, cipher: ctr::Ctr64LE<aes::Aes256>) -> Self {
        self.cipher = Some(cipher);
        self
    }
    pub fn reader(mut self, reader: BufReader<R>) -> Self {
        self.reader = Some(reader);
        self
    }
    pub fn writer(mut self, writer: BufWriter<W>) -> Self {
        self.writer = Some(writer);
        self
    }
    pub fn buffer(mut self, buffer: [u8; N]) -> Self {
        self.buffer = Some(buffer);
        self
    }
    pub fn build(self) -> Option<EStreamSync<N, R, W>> {
        let Self {
            cipher,
            buffer,
            reader,
            writer,
        } = self;
        let buffer = buffer.unwrap_or([0; N]);
        let cipher = cipher?;
        Some(EStreamSync {
            read_cipher: cipher.clone(),
            write_cipher: cipher,
            write_buff: buffer,
            reader: reader?,
            writer: writer?,
        })
    }
}
