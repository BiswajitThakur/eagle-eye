use std::{
    ffi::OsString,
    io,
    path::{Path, PathBuf},
    str::FromStr,
};

use ee_http::{HttpRequest, HttpResponse};

use crate::{ExeReceiverSync, ExeSenderSync, ExecuteResult, GetId};

pub struct RemoveFileSync {
    path: OsString,
}

impl<T: Into<OsString>> From<T> for RemoveFileSync {
    fn from(value: T) -> Self {
        Self { path: value.into() }
    }
}

impl GetId for RemoveFileSync {
    fn id() -> &'static str {
        "remove-file"
    }
}

impl<T: io::Read + io::Write, W: io::Write> ExeSenderSync<T, W> for RemoveFileSync {
    fn execute_on_sender(
        &self,
        mut stream: T,
        req: &mut HttpRequest,
        http: W,
    ) -> std::io::Result<ExecuteResult> {
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
            match r.unwrap() {
                ExecuteResult::Ok => {
                    HttpResponse::new().send_json_str(http, r#"{"status":"ok"}"#)?
                }
                _ => HttpResponse::new().send_json_str(http, r#"{"status":"faild"}"#)?,
            }
        }
        Ok(r.unwrap())
    }
}

impl<T: io::Read + io::Write> ExeReceiverSync<T> for RemoveFileSync {
    fn execute_on_receiver(mut stream: T) -> io::Result<T> {
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

impl RemoveFileSync {
    pub fn new(path: OsString) -> Self {
        Self { path }
    }
}

#[derive(Debug, PartialEq)]
pub struct LsSync {
    path: PathBuf,
    show_hidden: bool,
}

impl LsSync {
    pub fn home() -> Self {
        todo!()
    }
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        todo!()
    }
}

impl GetId for LsSync {
    fn id() -> &'static str {
        "ls"
    }
}

impl<T: io::Read + io::Write, W: io::Write> ExeSenderSync<T, W> for LsSync {
    fn execute_on_sender(
        &self,
        mut stream: T,
        req: &mut HttpRequest,
        http: W,
    ) -> io::Result<ExecuteResult> {
        let path = self
            .path
            .as_os_str()
            .to_string_lossy()
            .encode_utf16()
            .collect::<Vec<u16>>();
        let len = if let Some(v) = (path.len() as u16).checked_mul(2) {
            v
        } else {
            return Err(io::Error::other("Path length is too large."));
        };
        stream.write_all(&len.to_be_bytes())?;
        for v in path {
            stream.write_all(&v.to_be_bytes())?;
        }
        todo!()
    }
}

impl LsSync {
    pub fn execute_on_server<T: io::Read + io::Write>(mut stream: T) -> io::Result<T> {
        let mut buf = [0u8; 2];
        stream.read_exact(&mut buf)?;
        let mut len = u16::from_be_bytes(buf) as usize;
        if len % 2 != 0 {
            return Err(io::Error::other("invalid path length"));
        }
        let mut v: Vec<u16> = Vec::new();
        loop {
            if len == 0 {
                break;
            }
            stream.read_exact(&mut buf)?;
            v.push(u16::from_be_bytes(buf));
            len -= buf.len();
        }
        let s = String::from_utf16(&v).map_err(|v| io::Error::other(v.to_string()))?;
        for t in std::fs::read_dir(s)? {
            let entry = t?;
            let m = entry.metadata()?;
        }
        todo!()
    }
}

fn to_cross_vec_u8(path: &Path) -> Vec<u8> {
    let mut v = Vec::new();
    for i in path.to_string_lossy().encode_utf16() {
        v.extend_from_slice(&i.to_be_bytes());
    }
    v
}

fn from_cros_vec_u8(value: Vec<u8>) -> io::Result<PathBuf> {
    let mut v: Vec<u16> = Vec::with_capacity(value.len() / 2);
    let mut i = 0;
    while i + 1 < value.len() {
        let n = u16::from_be_bytes([value[i], value[i + 1]]);
        v.push(n);
        i += 2;
    }
    let s = String::from_utf16(&v).map_err(|v| io::Error::other(v.to_string()))?;
    let path = PathBuf::from_str(s.as_str()).map_err(|v| io::Error::other(v.to_string()))?;
    Ok(path)
}
