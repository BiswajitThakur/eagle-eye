use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

use ee_http::HttpRequest;
use ee_stream::EStreamSync;

use crate::{ExeReceiverSync, ExeSenderSync, ExecuteResult, GetId};

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
        _req: &mut HttpRequest,
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

impl<const N: usize> ExeReceiverSync<N> for Ping {
    fn execute_on_receiver<'a>(
        mut stream: EStreamSync<N, &'a TcpStream, &'a TcpStream>,
    ) -> io::Result<EStreamSync<N, &'a TcpStream, &'a TcpStream>> {
        let mut buf = [0; 4];
        stream.read_exact(&mut buf)?;
        if *b"ping" == buf {
            stream.write_all(b"pong")?;
        }
        stream.flush()?;
        Ok(stream)
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
}
