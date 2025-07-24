use std::io;

use ee_http::HttpRequest;

use crate::{ExeSenderSync, ExecuteResult, GetId};

pub struct Ping;

impl GetId for Ping {
    fn id() -> &'static str {
        "ping"
    }
}

impl<T: io::Read + io::Write, W: io::Write> ExeSenderSync<T, W> for Ping {
    fn execute_on_sender(
        &self,
        mut stream: T,
        req: &mut HttpRequest,
        _: W,
    ) -> std::io::Result<ExecuteResult> {
        let mut buf = [0; 4];
        stream.write_all(b"ping")?;
        stream.flush()?;
        stream.read_exact(&mut buf)?;
        if *b"pong" == buf {
            Ok(ExecuteResult::Ok)
        } else {
            Err(std::io::Error::other("ERROR: invalid response"))
        }
    }
}

impl Default for Ping {
    fn default() -> Self {
        Self::new()
    }
}

impl Ping {
    pub fn new() -> Self {
        Self
    }
    pub fn execute_on_server<T: io::Read + io::Write>(mut stream: T) -> io::Result<T> {
        let mut buf = [0; 4];
        stream.read_exact(&mut buf)?;
        if *b"ping" == buf {
            stream.write_all(b"pong")?;
        }
        stream.flush()?;
        Ok(stream)
    }
}
