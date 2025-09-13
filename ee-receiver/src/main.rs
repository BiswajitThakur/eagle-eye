mod app;
mod config;
mod proto;
mod utils;

use std::{
    any::Any,
    collections::HashMap,
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream},
    sync::Arc,
    time::Duration,
};

use aes::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use ee_app::{
    app::receiver_app::{ReceiverApp, ReceiverAppServer},
    app_data::MyStorage,
};
use ee_broadcaster::ReceiverInfo;
use ee_stream::{buffer::BufReadWriter, e_stream::EStreamSync};
use rand::Rng;

//use ee_task::prelude::*;

//use crate::{app::AppSync, config::config, my_app::ThreadCounter};
//

struct MyApp {
    key: [u8; 32],
}

impl MyApp {
    fn new() -> Self {
        Self { key: [0; 32] }
    }
}

fn main() -> io::Result<()> {
    /*
        let mut config = config();
        config
            .register::<RemoveFileSync>() // sender can remove file of receiver
            .register::<Ping>(); // sender can check, receiver is online or ofline.

        let app = AppSync::new(config);
        app.run()?;
    */
    let mut server = ReceiverAppServer::new(move || MyApp::new());
    server
        .app_name("eagle-eye")
        .version((1, 0, 0))
        .app_data(AppData::new())
        .handler(Handler {})
        .max_connection(8);

    server.auth(|app, data, stream| {
        let iv = rand::rng().random::<[u8; 16]>();
        let data = rand::rng().random::<[u8; 32]>();
        let mut buf = [0u8; 32];
        let mut cipher = ctr::Ctr64LE::<aes::Aes256>::new(&app.key.into(), &iv.into());
        cipher.apply_keystream_b2b(&data, &mut buf).unwrap();
        stream.write_all(&iv)?;
        stream.write_all(&buf)?;
        stream.flush()?;
        stream.read_exact(&mut buf)?;
        if data != buf {
            stream.write_all(b":1:")?;
            stream.flush()?;
            return Ok(false);
        }
        stream.write_all(b":0:")?;
        stream.flush()?;
        Ok(true)
    });

    server.run();
    Ok(())
}

#[derive(Default)]
struct AppData {
    inner: HashMap<String, Box<dyn Any + Send + Sync + 'static>>,
}

impl AppData {
    fn new() -> Self {
        Self::default()
    }
}

#[derive(Default)]
struct Handler {}

impl ReceiverApp for MyApp {
    type Stream = TcpStream;
    type BufStream = BufReadWriter<TcpStream>;
    type EStream = EStreamSync<Self::BufStream>;
    type AppData = AppData;
    type ConnectionHandler = Handler;
    fn get_stream(_: Arc<Self>) -> impl FnMut() -> Option<Self::Stream> {
        let mut receiver = ReceiverInfo::builder()
            .prefix("eagle-eye")
            .buffer_size(4096)
            .socket_addr(SocketAddr::from(([255, 255, 255, 255], 7766)))
            .build()
            .unwrap();
        let v = move || {
            loop {
                if let Ok(Some((a, _))) = receiver.next() {
                    let v = TcpStream::connect(a);
                    if v.is_err() {
                        continue;
                    }
                    return v.ok();
                } else {
                    return None;
                }
            }
        };
        v
    }
    fn to_buffer_stream(_this: &Arc<Self>, stream: Self::Stream) -> Self::BufStream {
        BufReadWriter::with_capacity(8 * 1024, stream)
    }
    fn handle_connection(
        data: Arc<std::sync::Mutex<Self::AppData>>,
        handler: Arc<Self::ConnectionHandler>,
        stream: Self::EStream,
    ) -> io::Result<()> {
        todo!()
    }
    fn log_error<E: std::error::Error>(_this: &Arc<Self>, _error: E) {
        todo!()
    }
    fn encrypt_connection(
        this: &Arc<Self>,
        data: &Arc<std::sync::Mutex<Self::AppData>>,
        stream: Self::BufStream,
    ) -> io::Result<Self::EStream> {
        todo!()
    }
}
