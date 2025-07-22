use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
    sync::{Arc, atomic::AtomicUsize},
};

use aes::cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7};
use eagle_eye_proto::server::EagleEyeServerSync;
use ee_stream::EStreamSync;

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
    let mut data = &mut data[18..];
    println!("Before Decrypt: {:?}", data);
    let pt = Aes256CbcDec::new(&key.into(), &iv.into());
    let data = pt.decrypt_padded_mut::<Pkcs7>(&mut data).ok()?;
    println!("After Decrypt: {:?}", data);
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
