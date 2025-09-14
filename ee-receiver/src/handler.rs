use std::{
    io::{self, Read, Write},
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use ee_app::receiver::sync::handler::ConnectionHandler as Handler;

pub struct ConnectionHandler<Data, T: Read + Write> {
    inner: Vec<T>,
    marker: PhantomData<Data>,
}

impl<Data, T: Read + Write> Default for ConnectionHandler<Data, T> {
    fn default() -> Self {
        Self {
            inner: Vec::new(),
            marker: PhantomData,
        }
    }
}

impl<Data, Stream: Read + Write + Send + Sync> Handler<Data, Stream>
    for ConnectionHandler<Data, Stream>
{
    fn get(
        &self,
        id: impl AsRef<str>,
    ) -> Option<&Box<dyn Fn(&Arc<Mutex<Data>>, &mut Stream) -> io::Result<()>>> {
        todo!()
    }
}
