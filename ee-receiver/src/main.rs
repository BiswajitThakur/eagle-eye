mod app;
mod config;
mod proto;
mod utils;

use std::{
    io,
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use ee_app::{
    app::receiver_app::{ReceiverApp, ReceiverAppServer},
    app_data::MyStorage,
};
use ee_broadcaster::ReceiverInfo;
use ee_stream::{buffer::BufReadWriter, e_stream::EStreamSync};

//use ee_task::prelude::*;

//use crate::{app::AppSync, config::config, my_app::ThreadCounter};
//

struct MyApp {}

fn main() -> io::Result<()> {
    /*
        let mut config = config();
        config
            .register::<RemoveFileSync>() // sender can remove file of receiver
            .register::<Ping>(); // sender can check, receiver is online or ofline.

        let app = AppSync::new(config);
        app.run()?;
    */
    let server = ReceiverAppServer::new(move || MyApp {})
        .app_name("eagle-eye")
        .version((1, 0, 0))
        .app_data(AppData {})
        .handler(Handler {})
        .max_connection(8);
    server.run();
    Ok(())
}

#[derive(Default)]
struct AppData {}
#[derive(Default)]
struct Handler {}

impl ReceiverApp for MyApp {
    type Stream = TcpStream;
    type BufStream = BufReadWriter<TcpStream>;
    type EStream = EStreamSync<Self::BufStream>;
    type AppData = AppData;
    type ConnectionHandler = Handler;
    fn get_stream(&self) -> impl FnMut() -> Option<Self::Stream> {
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
    fn accept_version(&self) -> impl Fn((u32, u32, u32)) -> bool {
        |(_, _, _)| true
    }
    fn to_buffer_stream(&self, stream: Self::Stream) -> Self::BufStream {
        BufReadWriter::with_capacity(8 * 1024, stream)
    }
    fn handle_auth(&self, stream: &mut Self::BufStream) -> io::Result<bool> {
        todo!()
    }
    fn log_error<E: std::error::Error>(&self, _error: E) {
        todo!()
    }
    fn encrypt_connection(
        &self,
        data: &std::sync::Arc<std::sync::Mutex<Self::AppData>>,
        stream: Self::BufStream,
    ) -> io::Result<Self::EStream> {
        todo!()
    }
    fn handle_connection(
        data: std::sync::Arc<std::sync::Mutex<Self::AppData>>,
        handler: std::sync::Arc<Self::ConnectionHandler>,
        stream: Self::EStream,
    ) -> io::Result<()> {
        todo!()
    }
}
