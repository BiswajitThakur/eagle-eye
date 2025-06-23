#[cfg(feature = "sync")]
use aes::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
#[cfg(feature = "sync")]
use rand::Rng;
#[cfg(feature = "sync")]
use std::{
    fs,
    io::{self, BufReader, BufWriter, Read, Write},
    path::Path,
};

#[cfg(feature = "sync")]
use crate::stream::EagleEyeStreamSync;
#[cfg(feature = "sync")]
pub(crate) fn handle_auth_on_server_sync<const N: usize, R: io::Read, W: io::Write>(
    key: [u8; 32],
    reader: R,
    writer: W,
) -> io::Result<Option<EagleEyeStreamSync<N, R, W>>> {
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);
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
    let e_stream = EagleEyeStreamSync::<N, R, W>::builder()
        .cipher(cipher)
        .reader(reader)
        .writer(writer)
        .build();
    Ok(e_stream)
}

#[cfg(feature = "sync")]
pub(crate) fn handle_auth_on_client_sync<const N: usize, R: io::Read, W: io::Write>(
    key: [u8; 32],
    reader: R,
    writer: W,
) -> io::Result<Option<EagleEyeStreamSync<N, R, W>>> {
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);
    let mut iv = [0; 16];
    let mut buf = [0u8; 32];
    reader.read_exact(&mut iv)?;
    reader.read_exact(&mut buf)?;
    let mut cipher = ctr::Ctr64LE::<aes::Aes256>::new(&key.into(), &iv.into());
    cipher.apply_keystream(&mut buf);
    writer.write_all(&buf)?;
    writer.flush()?;
    reader.read_exact(&mut buf[0..3])?;
    if buf[0..3] != *b":0:" {
        return Ok(None);
    }
    cipher.seek(0);
    let e_stream = EagleEyeStreamSync::<N, R, W>::builder()
        .cipher(cipher)
        .reader(reader)
        .writer(writer)
        .build();
    Ok(e_stream)
}

// TODO: write date and time in log
#[cfg(feature = "sync")]
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
    let _ = writeln!(w, "{}", err);
}
