use std::{
    error::Error,
    io::{Read, Result, Write},
    sync::{Arc, Mutex},
};

use crate::{app_data::AppData, receiver::sync::handler::ConnectionHandler};

pub trait App {
    type Stream: Read + Write + Send + Sync;
    type BufStream: Read + Write + Send + Sync;
    type EStream: Read + Write + Send + Sync;
    type AppData: AppData + Send + Sync;
    type ConnectionHandler: ConnectionHandler<Self::AppData, Self::EStream> + Send + Sync;
    fn get_stream(this: Arc<Self>) -> impl FnMut() -> Option<Self::Stream>;
    fn to_buffer_stream(this: &Arc<Self>, stream: Self::Stream) -> Self::BufStream;
    fn log_error<E: Error>(_: &Arc<Self>, _error: E) {}
    fn encrypt_connection(
        this: &Arc<Self>,
        data: &Arc<Mutex<Self::AppData>>,
        stream: Self::BufStream,
    ) -> Result<Self::EStream>;
}
