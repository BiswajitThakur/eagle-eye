use std::io;

use aes::cipher::StreamCipher;

pub struct EStreamSync<'a> {
    read_cipher: ctr::Ctr64LE<aes::Aes256>,
    write_cipher: ctr::Ctr64LE<aes::Aes256>,
    write_buff: Box<[u8]>,
    reader: Box<dyn io::Read + 'a>,
    writer: Box<dyn io::Write + 'a>,
}

// TODO: remove me
#[cfg(debug_assertions)]
impl Drop for EStreamSync<'_> {
    fn drop(&mut self) {
        println!("....EStream Droped......");
    }
}

impl io::Read for EStreamSync<'_> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.reader.read(buf)?;
        self.read_cipher.apply_keystream(&mut buf[0..n]);
        Ok(n)
    }
}

impl io::Write for EStreamSync<'_> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for chunk in buf.chunks(self.write_buff.len()) {
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

/*
impl io::BufRead for EStreamSync<'_> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.reader.fill_buf()
    }
    #[inline]
    fn consume(&mut self, amt: usize) {
        self.reader.consume(amt);
    }
}
*/

impl EStreamSync<'_> {
    #[inline]
    pub fn builder<'a>() -> EStreamBuilderSync<'a> {
        EStreamBuilderSync {
            cipher: None,
            buffer_size: std::num::NonZero::new(8 * 1024),
            reader: None,
            writer: None,
        }
    }
}

pub struct EStreamBuilderSync<'a> {
    cipher: Option<ctr::Ctr64LE<aes::Aes256>>,
    buffer_size: Option<std::num::NonZero<usize>>,
    reader: Option<Box<dyn io::Read + 'a>>,
    writer: Option<Box<dyn io::Write + 'a>>,
}

impl<'a> EStreamBuilderSync<'a> {
    pub fn cipher(mut self, cipher: ctr::Ctr64LE<aes::Aes256>) -> Self {
        self.cipher = Some(cipher);
        self
    }
    pub fn reader(mut self, reader: impl io::Read + 'a) -> Self {
        self.reader = Some(Box::new(reader));
        self
    }
    pub fn writer(mut self, writer: impl io::Write + 'a) -> Self {
        self.writer = Some(Box::new(writer));
        self
    }
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = std::num::NonZero::new(size);
        self
    }
    pub fn build(self) -> Option<EStreamSync<'a>> {
        let Self {
            cipher,
            buffer_size,
            reader,
            writer,
        } = self;
        let buffer_size = buffer_size?;
        let v = Box::<[u8]>::new_uninit_slice(buffer_size.get());
        let buffer = unsafe { v.assume_init() };
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
