use std::{
    fs,
    io::{self, BufReader, BufWriter, Read, Write},
    net::{SocketAddr, TcpStream},
    path::Path,
    sync::{Arc, atomic::AtomicUsize},
};

use aes::cipher::{
    BlockDecryptMut, KeyIvInit, StreamCipher, StreamCipherSeek, block_padding::Pkcs7,
};
use ee_stream::EStreamSync;
use rand::Rng;

use crate::receiver::EagleEyeServerSync;

pub fn process_broadcast_data(
    key: [u8; 32],
    id: u128,
    addr: SocketAddr,
    data: &mut [u8],
) -> Option<(SocketAddr, [u8; 16])> {
    type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
    if data.len() < 52 {
        return None;
    }
    let data_len = u16::from_be_bytes([data[0], data[1]]) as usize;
    if data_len != data.len() - 2 {
        return None;
    }
    let mut iv = [0; 16];
    iv.copy_from_slice(&data[2..18]);
    let data = &mut data[18..];
    println!("Before Decrypt: {data:?}");
    let pt = Aes256CbcDec::new(&key.into(), &iv.into());
    let data = pt.decrypt_padded_mut::<Pkcs7>(data).ok()?;
    println!("After Decrypt: {data:?}");
    let mut got_id = [0u8; 16];
    got_id.copy_from_slice(&data[..16]);
    let got_id = u128::from_be_bytes(got_id);
    if got_id != id {
        return None;
    }
    let secret = &data[16..32];
    let port = u16::from_be_bytes([data[32], data[33]]);
    let addr = SocketAddr::new(addr.ip(), port);
    let mut sec = [0u8; 16];
    sec.copy_from_slice(secret);
    Some((addr, sec))
}

pub fn process<const N: usize>(
    handler: Arc<EagleEyeServerSync<EStreamSync<N, TcpStream, TcpStream>>>,
    thread_count: Arc<AtomicUsize>,
    addr: SocketAddr,
    secret: [u8; 16],
) {
    let stream = TcpStream::connect(addr);
    if stream.is_err() {
        thread_count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        return;
    }
    let mut stream = stream.unwrap();
    if stream.write_all(&[0]).is_err() {
        thread_count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        return;
    }
    if stream.write_all(&secret).is_err() {
        thread_count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        return;
    }
    let _ = handler.handle_stream(stream.try_clone().unwrap(), stream);
    thread_count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
}

pub(crate) fn handle_auth_on_receiver_sync<const N: usize, R: io::Read, W: io::Write>(
    key: [u8; 32],
    reader: R,
    writer: W,
) -> io::Result<Option<EStreamSync<N, R, W>>> {
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
    let e_stream = EStreamSync::<N, R, W>::builder()
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
    let _ = writeln!(w, "{err}");
}
