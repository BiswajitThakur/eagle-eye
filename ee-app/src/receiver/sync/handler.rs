use std::{
    io::{self, Read, Write},
    sync::{Arc, Mutex},
};

pub trait ConnectionHandler<T: Read + Write>: Default + Send + Sync {
    type AppData: Default + Send + Sync;
    fn get(
        &self,
        id: impl AsRef<str>,
    ) -> Option<&Box<dyn Fn(&Arc<Mutex<Self::AppData>>, &mut T) -> io::Result<()>>>;
}

pub struct DefaultConnectionHandler<T: Read + Write> {
    inner: Vec<T>,
}

impl<T: Read + Write> Default for DefaultConnectionHandler<T> {
    fn default() -> Self {
        Self { inner: Vec::new() }
    }
}

impl<T: Read + Write + Send + Sync> ConnectionHandler<T> for DefaultConnectionHandler<T> {
    type AppData = ();
    fn get(
        &self,
        id: impl AsRef<str>,
    ) -> Option<&Box<dyn Fn(&Arc<Mutex<Self::AppData>>, &mut T) -> io::Result<()>>> {
        todo!()
    }
}
