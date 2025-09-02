use std::{io, num::NonZero};

use aes::cipher::StreamCipher;

use crate::buffer::BufReadWriter;

pub struct EStreamSync<T: io::Read + io::Write> {
    inner: BufReadWriter<T>,
    read_cipher: ctr::Ctr64LE<aes::Aes256>,
    write_cipher: ctr::Ctr64LE<aes::Aes256>,
    write_buff: Box<[u8]>,
}

// TODO: remove me
#[cfg(debug_assertions)]
impl<T: io::Read + io::Write> Drop for EStreamSync<T> {
    fn drop(&mut self) {
        println!("....EStream Droped......");
    }
}

impl<T: io::Read + io::Write> io::Read for EStreamSync<T> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.read_cipher.apply_keystream(&mut buf[0..n]);
        Ok(n)
    }
}

impl<T: io::Read + io::Write> io::Write for EStreamSync<T> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let r = std::cmp::min(buf.len(), self.write_buff.len());
        let src = unsafe { buf.get_unchecked(0..r) };
        let dest = unsafe { self.write_buff.get_unchecked_mut(0..r) };
        self.write_cipher.apply_keystream_b2b(src, dest).unwrap();
        self.inner.write_all(dest)?;
        Ok(r)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: io::Read + io::Write> io::BufRead for EStreamSync<T> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }
    #[inline]
    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt);
    }
}

impl<T: io::Read + io::Write> EStreamSync<T> {
    #[inline]
    pub fn builder() -> EStreamBuilderSync<T> {
        EStreamBuilderSync {
            cipher: None,
            read_buf_size: NonZero::new(8 * 1024),
            write_buf_size: NonZero::new(8 * 1024),
            inner: None,
        }
    }
}

pub struct EStreamBuilderSync<T: io::Read + io::Write> {
    cipher: Option<ctr::Ctr64LE<aes::Aes256>>,
    read_buf_size: Option<NonZero<usize>>,
    write_buf_size: Option<NonZero<usize>>,
    inner: Option<T>,
}

impl<T: io::Read + io::Write> EStreamBuilderSync<T> {
    pub fn cipher(mut self, cipher: ctr::Ctr64LE<aes::Aes256>) -> Self {
        self.cipher = Some(cipher);
        self
    }
    pub fn inner(mut self, v: T) -> Self {
        self.inner = Some(v);
        self
    }
    pub fn read_buffer_size(mut self, size: usize) -> Self {
        self.read_buf_size = NonZero::new(size);
        self
    }
    pub fn write_buffer_size(mut self, size: usize) -> Self {
        self.write_buf_size = NonZero::new(size);
        self
    }
    pub fn build(self) -> Option<EStreamSync<T>> {
        let Self {
            cipher,
            read_buf_size,
            write_buf_size,
            inner,
        } = self;
        let read_buffer_size = read_buf_size?;
        let write_buffer_size = write_buf_size?;
        let v = Box::<[u8]>::new_uninit_slice(write_buffer_size.get());
        let buffer = unsafe { v.assume_init() };
        let cipher = cipher?;
        Some(EStreamSync {
            inner: BufReadWriter::with_read_write_capacity(
                read_buffer_size.get(),
                write_buffer_size.get(),
                inner?,
            ),
            read_cipher: cipher.clone(),
            write_cipher: cipher,
            write_buff: buffer,
        })
    }
}
