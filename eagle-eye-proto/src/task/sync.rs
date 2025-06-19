use std::io;

use crate::Connection;

pub trait ServerTaskSync<T: io::Read + io::Write> {
    fn id(&self) -> &'static str;
    fn execute(&self, stream: T) -> io::Result<Connection>;
}

pub trait ClientTaskSync<T: io::Read + io::Write> {
    fn id(&self) -> &'static str;
    fn execute(&self, stream: T) -> io::Result<()>;
}
