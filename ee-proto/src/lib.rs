use std::{
    any::TypeId,
    collections::HashMap,
    fmt::Write,
    io::{self, Read},
};

trait AppReceiver {
    fn get_handle<T, U>(id: impl AsRef<str>) -> Option<impl Fn(U, T) -> io::Result<()>>
    where
        T: Read + Write;
}
