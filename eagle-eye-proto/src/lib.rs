use std::{
    io::{self, BufReader, BufWriter, Read, Write},
    net::TcpStream,
    time::Duration,
};

use aes::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use rand::Rng;

pub struct EagleEyeStreamSync<const N: usize, R: io::Read, W: io::Write> {
    cipher: ctr::Ctr64LE<aes::Aes256>,
    write_buff: [u8; N],
    reader: BufReader<R>,
    writer: BufWriter<W>,
}

impl<const N: usize, R: io::Read, W: io::Write> io::Read for EagleEyeStreamSync<N, R, W> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.reader.read(buf)?;
        self.cipher.apply_keystream(&mut buf[0..n]);
        Ok(n)
    }
}

impl<const N: usize, R: io::Read, W: io::Write> io::Write for EagleEyeStreamSync<N, R, W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for chunk in buf.chunks(N) {
            self.cipher
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

impl<const N: usize, R: io::Read, W: io::Write> EagleEyeStreamSync<N, R, W> {
    pub fn builder() -> EagleEyeStreamBuilderSync<N, R, W> {
        EagleEyeStreamBuilderSync {
            key: None,
            iv: None,
            buffer: None,
            reader: None,
            writer: None,
        }
    }
}

pub struct EagleEyeStreamBuilderSync<const N: usize, R: io::Read, W: io::Write> {
    key: Option<[u8; 32]>,
    iv: Option<[u8; 16]>,
    buffer: Option<[u8; N]>,
    reader: Option<BufReader<R>>,
    writer: Option<BufWriter<W>>,
}

impl<const N: usize, R: io::Read, W: io::Write> EagleEyeStreamBuilderSync<N, R, W> {
    pub fn key(mut self, key: [u8; 32]) -> Self {
        self.key = Some(key);
        self
    }
    pub fn iv(mut self, iv: [u8; 16]) -> Self {
        self.iv = Some(iv);
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
    pub fn build(self) -> Option<EagleEyeStreamSync<N, R, W>> {
        let Self {
            key,
            iv,
            buffer,
            reader,
            writer,
        } = self;

        let cipher = ctr::Ctr64LE::<aes::Aes256>::new(&key?.into(), &iv?.into());
        let buffer = buffer.unwrap_or([0; N]);
        Some(EagleEyeStreamSync {
            cipher,
            write_buff: buffer,
            reader: reader?,
            writer: writer?,
        })
    }
}

pub fn handle_stream_server_sync<const N: usize>(
    key: [u8; 32],
    stream: &TcpStream,
) -> io::Result<Option<EagleEyeStreamSync<N, &TcpStream, &TcpStream>>> {
    stream.set_read_timeout(Some(Duration::from_secs(1)))?;
    let mut reader = BufReader::new(stream);
    let mut writer = BufWriter::new(stream);
    let iv = rand::rng().random::<[u8; 16]>();
    let data = rand::rng().random::<[u8; 32]>();
    let mut buf = [0u8; 32];
    let mut cipher = ctr::Ctr64LE::<aes::Aes256>::new(&key.into(), &iv.into());
    cipher.apply_keystream_b2b(&data, &mut buf).unwrap();
    writer.write_all(&iv)?;
    writer.write_all(&buf)?;
    writer.flush()?;
    reader.read_exact(&mut buf)?;
    if data != buf {
        writer.write_all(b":1:")?;
        writer.flush()?;
        return Ok(None);
    }
    writer.write_all(b":0:")?;
    writer.flush()?;
    cipher.seek(0);
    stream.set_read_timeout(None)?;
    Ok(Some(EagleEyeStreamSync {
        cipher,
        write_buff: [0; N],
        reader,
        writer,
    }))
}

pub fn handle_stream_client_sync<const N: usize>(
    key: [u8; 32],
    stream: &TcpStream,
) -> io::Result<Option<EagleEyeStreamSync<N, &TcpStream, &TcpStream>>> {
    stream.set_read_timeout(Some(Duration::from_secs(1)))?;
    let mut reader = BufReader::new(stream);
    let mut writer = BufWriter::new(stream);
    let mut iv = [0; 16];
    let mut buf = [0u8; 32];
    reader.read_exact(&mut iv)?;
    reader.read_exact(&mut buf)?;
    let mut cipher = ctr::Ctr64LE::<aes::Aes256>::new(&key.into(), &iv.into());
    cipher.apply_keystream(&mut buf);
    writer.write_all(&buf)?;
    writer.flush()?;
    reader.read_exact(&mut buf[0..3])?;
    if &buf[0..3] != &*b":0:" {
        return Ok(None);
    }
    cipher.seek(0);
    stream.set_read_timeout(None)?;
    Ok(Some(EagleEyeStreamSync {
        cipher,
        write_buff: [0; N],
        reader,
        writer,
    }))
}
