use std::{ffi::OsString, io, str::FromStr};

use ee_http::HttpResponse;

use crate::{ExeSenderSync, ExecuteResult, GetId};

pub struct RemoveFile {
    path: OsString,
}

impl<T: Into<OsString>> From<T> for RemoveFile {
    fn from(value: T) -> Self {
        Self { path: value.into() }
    }
}

impl GetId for RemoveFile {
    fn id() -> &'static str {
        "remove-file"
    }
}

impl<T: io::Read + io::Write, W: io::Write> ExeSenderSync<T, W> for RemoveFile {
    fn execute_on_sender(&self, mut stream: T, http: W) -> std::io::Result<ExecuteResult> {
        let mut buf = [0; 1];
        let path = self.path.to_string_lossy();
        let bytes = path.as_bytes();
        let len = bytes.len() as u16;
        stream.write_all(&len.to_be_bytes())?;
        stream.write_all(bytes)?;
        stream.flush()?;
        stream.read_exact(&mut buf)?;
        let res = HttpResponse::new();
        let r = ExecuteResult::try_from(buf);
        if r.is_err() {
            res.send_str(http, "ERROR: faild to remove file")?;
            return Err(io::Error::other("Invalid Execution Result"));
        } else {
            res.send_str(http, "OK: success")?;
        }
        Ok(r.unwrap())
    }
}

impl RemoveFile {
    pub fn new(path: OsString) -> Self {
        Self { path }
    }
    pub fn execute_on_server<T: io::Read + io::Write>(mut stream: T) -> io::Result<T> {
        let mut buf = [0; 8];
        stream.read_exact(&mut buf[0..2])?;
        let mut len = u16::from_be_bytes([buf[0], buf[1]]) as usize;
        let mut path = Vec::new();
        loop {
            let read_len = std::cmp::min(len, buf.len());
            let n = stream.read(&mut buf[0..read_len])?;
            if n == 0 {
                break;
            }
            path.extend_from_slice(&buf[0..n]);
            len -= n;
            if len == 0 {
                break;
            }
        }
        let s = String::from_utf8_lossy(&path);
        let path = OsString::from_str(&s);
        if path.is_err() {
            stream.write_all(&ExecuteResult::InvalidPath.to_be_bytes())?;
            stream.flush()?;
            return Ok(stream);
        }
        let path = path.unwrap();
        if std::fs::remove_file(path).is_err() {
            stream.write_all(&ExecuteResult::Faild.to_be_bytes())?;
            stream.flush()?;
            return Ok(stream);
        }
        stream.write_all(&ExecuteResult::Ok.to_be_bytes())?;
        stream.flush()?;
        Ok(stream)
    }
}
