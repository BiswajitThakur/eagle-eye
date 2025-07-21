mod utils;

use std::{
    io::{self, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    sync::{Arc, atomic::AtomicUsize},
    time::Duration,
};

use aes::cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7};
use eagle_eye_broadcaster::ReceiverInfo;
use eagle_eye_jobs::file::RemoveFile;
use eagle_eye_proto::{server::EagleEyeServerSync, stream::EagleEyeStreamSync, task::GetId};

const MAX_CONNECTIONS: usize = 16;

fn main() -> io::Result<()> {
    type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
    let key: [u8; 32] = [33; 32];
    let id: u128 = 123;
    let thread_counter = Arc::new(AtomicUsize::new(0));

    let mut handler: EagleEyeServerSync<EagleEyeStreamSync<512, TcpStream, TcpStream>> =
        EagleEyeServerSync::default().key([33; 32]);
    handler.register(RemoveFile::id(), RemoveFile::execute_on_server);

    let handler = Arc::new(handler);

    let mut receiver = ReceiverInfo::builder()
        .prefix(":eagle-eye:")
        .buffer([0; 2048])
        .socket_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 6923))
        .build()?;

    while let Ok(Some((addr, data))) = receiver.next() {
        let count = thread_counter.load(std::sync::atomic::Ordering::SeqCst);
        if count >= MAX_CONNECTIONS {
            std::thread::sleep(Duration::from_millis(500));
            continue;
        }
        thread_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let h = handler.clone();
        std::thread::spawn(move || process::<512>(h, key, id, addr, data, thread_counter));
    }

    Ok(())
}

fn process<const N: usize>(
    handler: Arc<EagleEyeServerSync<EagleEyeStreamSync<N, TcpStream, TcpStream>>>,
    key: [u8; 32],
    id: u128,
    addr: SocketAddr,
    data: &mut [u8],
    current: Arc<AtomicUsize>,
) {
    type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
    if data.len() < 52 {
        current.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        return;
    }
    let data_len = u16::from_be_bytes([data[0], data[1]]) as usize;
    if data_len != data.len() - 2 {
        current.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        return;
    }
    let mut iv = [0; 16];
    iv.copy_from_slice(&data[2..18]);
    let mut data = &mut data[18..];
    println!("Before Decrypt: {:?}", data);
    let pt = Aes256CbcDec::new(&key.into(), &iv.into());
    let data = pt.decrypt_padded_mut::<Pkcs7>(&mut data);
    if data.is_err() {
        current.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        return;
    }
    let data = data.unwrap();
    println!("After Decrypt: {:?}", data);
    let mut got_id = [0u8; 16];
    got_id.copy_from_slice(&data[..16]);
    let got_id = u128::from_be_bytes(got_id);
    if got_id != id {
        current.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        return;
    }
    let secret = &data[16..32];
    let port = u16::from_be_bytes([data[32], data[33]]);
    let addr = SocketAddr::new(addr.ip(), port);
    let stream = TcpStream::connect(addr);
    if stream.is_err() {
        current.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        return;
    }
    let mut stream = stream.unwrap();
    if stream.write_all(&[0]).is_err() {
        current.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        return;
    }
    if stream.write_all(secret).is_err() {
        current.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        return;
    }
    let _ = handler.handle_stream(stream.try_clone().unwrap(), stream);
}
