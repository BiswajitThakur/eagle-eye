use aes::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use rand::Rng;
use std::{
    fs,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    path::{Path, PathBuf},
    time::Duration,
};

use crate::stream::EagleEyeStreamSync;

pub fn handle_stream_server_sync<'a, const N: usize>(
    key: [u8; 32],
    stream: &'a TcpStream,
) -> io::Result<Option<EagleEyeStreamSync<N, &'a TcpStream, &'a TcpStream>>> {
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
    let e_stream = EagleEyeStreamSync::<N, &TcpStream, &TcpStream>::builder()
        .cipher(cipher)
        .reader(reader)
        .writer(writer)
        .build();
    Ok(e_stream)
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
    let e_stream = EagleEyeStreamSync::<N, &TcpStream, &TcpStream>::builder()
        .cipher(cipher)
        .reader(reader)
        .writer(writer)
        .build();
    Ok(e_stream)
}

// TODO: write date and time in log
pub(crate) fn write_log_sync<P: AsRef<Path>>(path: Option<P>, err: io::Error) {
    if path.is_none() {
        return;
    }
    let file = fs::OpenOptions::new().append(true).open(path.unwrap());
    if file.is_err() {
        return;
    }
    let mut w = BufWriter::new(file.unwrap());
    if writeln!(w, "------------------------").is_err() {
        return;
    }
    if writeln!(w, "{}", err).is_err() {
        return;
    }
}
