use std::{
    io::{self, Read, Write},
    marker::PhantomData,
    sync::{Arc, Mutex},
};

pub trait ConnectionHandler<Data, Stream: Read + Write>: Default {
    fn get(
        &self,
        id: impl AsRef<str>,
    ) -> Option<&Box<dyn Fn(&Arc<Mutex<Data>>, &mut Stream) -> io::Result<()>>>;
}
