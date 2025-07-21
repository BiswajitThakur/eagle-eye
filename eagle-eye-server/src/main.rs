use std::{
    io::{self, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
};

use aes::cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7};
use eagle_eye_broadcaster::ReceiverInfo;
use eagle_eye_jobs::file::RemoveFile;
use eagle_eye_proto::{server::EagleEyeServerSync, stream::EagleEyeStreamSync, task::GetId};

fn main() -> io::Result<()> {
    type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
    let key: [u8; 32] = [33; 32];
    let id: u128 = 123;

    let mut server: EagleEyeServerSync<EagleEyeStreamSync<1014, TcpStream, TcpStream>> =
        EagleEyeServerSync::default();
    server.register(RemoveFile::id(), RemoveFile::execute_on_server);

    let mut receiver = ReceiverInfo::builder()
        .prefix(":eagle-eye:")
        .buffer([0; 2048])
        .socket_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 6923))
        .build()?;
    while let Ok(Some((addr, data))) = receiver.next() {
        dbg!(99999);
        if data.len() < 52 {
            continue;
        }
        let data_len = u16::from_be_bytes([data[0], data[1]]) as usize;
        if data_len != data.len() - 2 {
            continue;
        }
        let mut iv = [0; 16];
        iv.copy_from_slice(&data[2..18]);
        let mut data = &mut data[18..];
        let pt = Aes256CbcDec::new(&key.into(), &iv.into());
        let data = pt.decrypt_padded_mut::<Pkcs7>(&mut data);
        if data.is_err() {
            continue;
        }
        let dec_data = data.unwrap();
        let mut got_id = [0u8; 16];
        got_id.copy_from_slice(&dec_data[..16]);
        let got_id = u128::from_be_bytes(got_id);
        if got_id != id {
            continue;
        }
        let secret = &dec_data[16..32];
        let port = u16::from_be_bytes([dec_data[32], dec_data[33]]);
        let addr = SocketAddr::new(addr.ip(), port);
        let stream = TcpStream::connect(addr);
        if stream.is_err() {
            continue;
        }
        let mut stream = stream.unwrap();
        if stream.write_all(&[0]).is_err() {
            continue;
        }
        if stream.write_all(secret).is_err() {
            continue;
        }
        let _ = server.handle_stream(stream.try_clone().unwrap(), stream);
    }

    Ok(())
}
