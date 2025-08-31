use std::io::{self, Read, Write};

use ee_http::{HttpRequest, HttpResponse};
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

impl ExeReceiverSync for Ping {
    fn execute_on_receiver(mut stream: EStreamSync) -> io::Result<EStreamSync> {
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

pub struct DeviceInfo {}

impl DeviceInfo {
    pub fn new() -> Self {
        Self {}
    }
}

impl GetId for DeviceInfo {
    fn id() -> &'static str {
        "dv-id"
    }
}

impl<T: io::Read + io::Write, W: io::Write> ExeSenderSync<T, W> for DeviceInfo {
    fn execute_on_sender(
        &self,
        mut stream: T,
        _req: &mut HttpRequest,
        http: W,
    ) -> io::Result<ExecuteResult> {
        let mut buf = [0u8; 8];
        stream.read_exact(&mut buf[0..1])?;
        let mut user_len = u8::from_be_bytes([buf[0]]) as usize;
        stream.read_exact(&mut buf[0..1])?;
        let mut os_len = u8::from_be_bytes([buf[0]]) as usize;
        let mut user = Vec::new();
        loop {
            if user_len == 0 {
                break;
            }
            let n = stream.read(&mut buf[0..std::cmp::min(user_len, 8)])?;
            user_len -= n;
            user.extend_from_slice(&buf[0..n]);
        }
        let mut os = Vec::new();
        loop {
            if os_len == 0 {
                break;
            }
            let n = stream.read(&mut buf[0..std::cmp::min(os_len, 8)])?;
            os_len -= n;
            os.extend_from_slice(&buf[0..n]);
        }
        let user = unsafe { String::from_utf8_unchecked(user) };
        let os = unsafe { String::from_utf8_unchecked(os) };
        let data = format!("{{\"user\":\"{}\",\"os\":{}}}", user, os);
        HttpResponse::new().send_json_str(http, data)?;
        Ok(ExecuteResult::Ok)
    }
}

impl ExeReceiverSync for DeviceInfo {
    fn execute_on_receiver(mut stream: EStreamSync) -> io::Result<EStreamSync> {
        let user = std::env::var(if cfg!(windows) { "USERNAME" } else { "USER" })
            .unwrap_or("unknown".to_owned());
        let os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            "unknown"
        };
        stream.write_all(&(user.len() as u8).to_be_bytes())?;
        stream.write_all(&(os.len() as u8).to_be_bytes())?;
        stream.write_all(user.as_bytes())?;
        stream.write_all(os.as_bytes())?;
        stream.write_all(&ExecuteResult::Ok.to_be_bytes())?;
        stream.flush()?;
        Ok(stream)
    }
}
