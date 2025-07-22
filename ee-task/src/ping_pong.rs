use std::io;

use eagle_eye_proto::task::{ExecuteResult, GetId, TaskSync};

pub struct Ping;

impl GetId for Ping {
    fn id() -> &'static str {
        "ping"
    }
}

impl<T: io::Read + io::Write, W: io::Write> TaskSync<T, W> for Ping {
    fn execute_on_client(&self, mut stream: T, _: W) -> std::io::Result<ExecuteResult> {
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
