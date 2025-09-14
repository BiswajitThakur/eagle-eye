use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream},
    sync::Arc,
};

use aes::cipher::{
    BlockDecryptMut, KeyIvInit, StreamCipher, StreamCipherSeek, block_padding::Pkcs7,
};
use ee_app::receiver::sync::app::App as SenderApp;
use ee_broadcaster::ReceiverInfo;
use ee_stream::{buffer::BufReadWriter, e_stream::EStreamSync};
use rand::Rng;

use crate::{data::AppData, handler::ConnectionHandler};

pub struct App {
    id: u128,
    key: [u8; 32],
    socket_addr: SocketAddr,
    broadcast_buf_size: usize,
    broadcast_data_prefix: &'static str,
}

impl App {
    pub fn new() -> Self {
        Self {
            id: 0,
            key: [0; 32],
            socket_addr: SocketAddr::from(([0, 0, 0, 0], 7766)),
            broadcast_buf_size: 8 * 1024,
            broadcast_data_prefix: "",
        }
    }
    pub fn key(&self) -> &[u8; 32] {
        &self.key
    }
    pub fn id(&self) -> u128 {
        self.id
    }
}

impl SenderApp for App {
    type Stream = TcpStream;
    type BufStream = BufReadWriter<Self::Stream>;
    type EStream = EStreamSync<Self::Stream>;
    type AppData = AppData;
    type ConnectionHandler = ConnectionHandler<Self::AppData, Self::EStream>;
    fn to_buffer_stream(_this: &Arc<Self>, stream: Self::Stream) -> Self::BufStream {
        BufReadWriter::with_capacity(8 * 1024, stream)
    }
    fn log_error<E: std::error::Error>(_: &Arc<Self>, error: E) {
        eprintln!("{}\n", error);
    }
    fn encrypt_connection(
        this: &Arc<Self>,
        _data: &Arc<std::sync::Mutex<Self::AppData>>,
        mut stream: Self::BufStream,
    ) -> std::io::Result<Self::EStream> {
        let iv = rand::rng().random::<[u8; 16]>();
        let data = rand::rng().random::<[u8; 32]>();
        let mut buf = [0u8; 32];
        let mut cipher = ctr::Ctr64LE::<aes::Aes256>::new(this.key().into(), &iv.into());
        cipher.apply_keystream_b2b(&data, &mut buf).unwrap();
        stream.write_all(&iv)?;
        stream.write_all(&buf)?;
        stream.flush()?;
        stream.read_exact(&mut buf)?;
        if data != buf {
            stream.write_all(b":1:")?;
            stream.flush()?;
            return Err(io::Error::other("error"));
        }
        stream.write_all(b":0:")?;
        stream.flush()?;
        cipher.seek(0);
        let e_stream: EStreamSync<TcpStream> = EStreamSync::builder()
            .cipher(cipher)
            .read_buffer_size(8 * 1024)
            .write_buffer_size(8 * 1024)
            .inner(stream.inner())
            .build()
            .unwrap();
        Ok(e_stream)
    }
    fn get_stream(this: Arc<Self>) -> impl FnMut() -> Option<Self::Stream> {
        let mut receiver = ReceiverInfo::builder()
            .prefix(this.broadcast_data_prefix)
            .buffer_size(this.broadcast_buf_size)
            .socket_addr(this.socket_addr)
            .build()
            .unwrap();
        let v = move || {
            loop {
                if let Ok(Some((addr, buf, len))) = receiver.next() {
                    type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
                    if len < 52 {
                        continue;
                    }
                    let data_len = u16::from_be_bytes([buf[0], buf[1]]) as usize;
                    if data_len != len - 2 {
                        continue;
                    }
                    let mut iv = [0; 16];
                    iv.copy_from_slice(&buf[2..18]);
                    let data = &mut buf[18..len];
                    // TODO: remove me
                    println!("Before Decrypt: {data:?}");
                    let pt = Aes256CbcDec::new(this.key().into(), &iv.into());
                    let data = pt.decrypt_padded_mut::<Pkcs7>(data).ok()?;
                    // TODO: remove me
                    println!("After Decrypt: {data:?}");
                    let mut got_id = [0u8; 16];
                    got_id.copy_from_slice(&data[..16]);
                    let got_id = u128::from_be_bytes(got_id);
                    if got_id != this.id() {
                        continue;
                    }
                    let secret = &data[16..32];
                    let port = u16::from_be_bytes([data[32], data[33]]);
                    let addr = SocketAddr::new(addr.ip(), port);

                    let v = TcpStream::connect(addr);
                    if v.is_err() {
                        continue;
                    }
                    let mut v = v.unwrap();
                    if v.write_all(secret).is_err() {
                        continue;
                    }
                    if v.flush().is_err() {
                        continue;
                    }
                    return Some(v);
                } else {
                    return None;
                }
            }
        };
        v
    }
}
