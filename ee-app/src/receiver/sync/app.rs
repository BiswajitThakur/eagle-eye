use std::{
    error::Error,
    io::{Read, Result, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
};

use ee_stream::{buffer::BufReadWriter, e_stream::EStreamSync};

use crate::receiver::sync::handler::{ConnectionHandler, DefaultConnectionHandler};

pub trait App {
    type Stream: Read + Write + Send + Sync;
    type BufStream: Read + Write + Send + Sync;
    type EStream: Read + Write + Send + Sync;
    type AppData: Default + Send + Sync;
    type ConnectionHandler: ConnectionHandler<Self::EStream, AppData = Self::AppData>;
    fn get_stream(this: Arc<Self>) -> impl FnMut() -> Option<Self::Stream>;
    fn to_buffer_stream(this: &Arc<Self>, stream: Self::Stream) -> Self::BufStream;
    fn log_error<E: Error>(_: &Arc<Self>, _error: E) {}
    fn encrypt_connection(
        this: &Arc<Self>,
        data: &Arc<Mutex<Self::AppData>>,
        stream: Self::BufStream,
    ) -> Result<Self::EStream>;
}

pub struct DefaultApp {}

impl App for DefaultApp {
    type Stream = TcpStream;
    type BufStream = BufReadWriter<Self::Stream>;
    type EStream = EStreamSync<Self::BufStream>;
    type AppData = ();
    type ConnectionHandler = DefaultConnectionHandler<Self::EStream>;
    fn get_stream(this: Arc<Self>) -> impl FnMut() -> Option<Self::Stream> {
        || todo!()
    }
    fn to_buffer_stream(this: &Arc<Self>, stream: Self::Stream) -> Self::BufStream {
        todo!()
    }
    fn log_error<E: Error>(_: &Arc<Self>, _error: E) {
        todo!()
    }
    fn encrypt_connection(
        this: &Arc<Self>,
        data: &Arc<Mutex<Self::AppData>>,
        stream: Self::BufStream,
    ) -> Result<Self::EStream> {
        todo!()
    }
}
