use aes::cipher::StreamCipher;
use std::{
    io::{self, BufReader, BufWriter, Write},
    net::TcpStream,
};

use crate::{
    Connection,
    task::{ExecuteResult, TaskSync},
};

pub struct EagleEyeStreamSync<const N: usize, R: io::Read, W: io::Write> {
    read_cipher: ctr::Ctr64LE<aes::Aes256>,
    write_cipher: ctr::Ctr64LE<aes::Aes256>,
    write_buff: [u8; N],
    reader: BufReader<R>,
    writer: BufWriter<W>,
}

impl<const N: usize, R: io::Read, W: io::Write> io::Read for EagleEyeStreamSync<N, R, W> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.reader.read(buf)?;
        self.read_cipher.apply_keystream(&mut buf[0..n]);
        Ok(n)
    }
}

impl<const N: usize, R: io::Read, W: io::Write> io::Write for EagleEyeStreamSync<N, R, W> {
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

impl<const N: usize, R: io::Read, W: io::Write> io::BufRead for EagleEyeStreamSync<N, R, W> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.reader.fill_buf()
    }
    #[inline]
    fn consume(&mut self, amt: usize) {
        self.reader.consume(amt);
    }
}

/*
impl<const N: usize, R: io::Read, W: io::Write> ServerTaskSync<&mut EagleEyeStreamSync<N, R, W>>
    for &mut EagleEyeStreamSync<N, R, W>
{
    fn id(&self) -> &'static str {
        todo!()
    }
    fn execute(&self, stream: &mut EagleEyeStreamSync<N, R, W>) -> io::Result<Connection> {
        todo!()
    }
}
*/

impl<const N: usize, R: io::Read, W: io::Write> EagleEyeStreamSync<N, R, W> {
    pub fn builder() -> EagleEyeStreamBuilderSync<N, R, W> {
        EagleEyeStreamBuilderSync {
            cipher: None,
            buffer: None,
            reader: None,
            writer: None,
        }
    }
    pub fn send_task<U: io::Write, E: io::Write, T: for<'a> TaskSync<&'a mut Self, U, E>>(
        &mut self,
        task: T,
        ok: U,
        err: E,
    ) -> io::Result<ExecuteResult> {
        let result = task.execute(self, ok, err)?;
        let flag: u8 = 0b00000000;
        self.write_all(&flag.to_be_bytes())?;
        Ok(result)
    }
    pub fn end(&mut self) -> io::Result<()> {
        let flag: u8 = 0b01000000;
        self.write_all(&flag.to_be_bytes())?;
        Ok(())
    }
    pub fn stop_server(&mut self) -> io::Result<()> {
        let flag: u8 = 0b11000000;
        self.write_all(&flag.to_be_bytes())?;
        Ok(())
    }
    pub fn handle_from_listener(
        &mut self,
        f: fn(&mut Self) -> io::Result<Connection>,
    ) -> io::Result<Connection> {
        f(self)
    }
}

macro_rules! shutdown_eagle_eye_stream {
    ($stream:ty) => {
        impl<const N: usize> EagleEyeStreamSync<N, $stream, $stream> {
            pub fn shutdown_read(&self) -> io::Result<()> {
                let inner = self.reader.get_ref();
                inner.shutdown(std::net::Shutdown::Read)
            }
            pub fn shutdown_write(&mut self) -> io::Result<()> {
                self.writer.flush()?;
                let inner = self.writer.get_ref();
                inner.shutdown(std::net::Shutdown::Write)
            }
            pub fn shutdown_both(&mut self) -> io::Result<()> {
                self.writer.flush()?;
                let inner = self.writer.get_ref();
                inner.shutdown(std::net::Shutdown::Write)
            }
        }
    };
}

shutdown_eagle_eye_stream!(&TcpStream);
shutdown_eagle_eye_stream!(TcpStream);

pub struct EagleEyeStreamBuilderSync<const N: usize, R: io::Read, W: io::Write> {
    cipher: Option<ctr::Ctr64LE<aes::Aes256>>,
    buffer: Option<[u8; N]>,
    reader: Option<BufReader<R>>,
    writer: Option<BufWriter<W>>,
}

impl<const N: usize, R: io::Read, W: io::Write> EagleEyeStreamBuilderSync<N, R, W> {
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
    pub fn build(self) -> Option<EagleEyeStreamSync<N, R, W>> {
        let Self {
            cipher,
            buffer,
            reader,
            writer,
        } = self;
        let buffer = buffer.unwrap_or([0; N]);
        let cipher = cipher?;
        Some(EagleEyeStreamSync {
            read_cipher: cipher.clone(),
            write_cipher: cipher,
            write_buff: buffer,
            reader: reader?,
            writer: writer?,
        })
    }
}
