use std::io;

use crate::Connection;

#[derive(Debug, Clone)]
pub enum ExecuteResult {
    Ok,
    Error(String),
}

pub trait TaskSync<T: io::Read + io::Write, W: io::Write, E: io::Write> {
    fn id() -> &'static str;
    fn execute(&self, stream: T, ok: W, err: E) -> io::Result<ExecuteResult>;
    fn execute_on_listener(mut stream: T) -> io::Result<Connection> {
        // response
        // <0 - success | 1 - faild>
        todo!()
    }
}
