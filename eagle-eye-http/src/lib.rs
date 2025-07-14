mod status;

pub use status::Status;

use std::{
    borrow::Cow,
    fmt,
    io::{self, BufReader},
    path::Path,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Method {
    Connect,
    Delete,
    #[default]
    Get,
    Head,
    Options,
    Post,
    Put,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect => write!(f, "CONNECT"),
            Self::Delete => write!(f, "DELETE"),
            Self::Get => write!(f, "GET"),
            Self::Head => write!(f, "HEAD"),
            Self::Options => write!(f, "OPTIONS"),
            Self::Post => write!(f, "POST"),
            Self::Put => write!(f, "PUT"),
        }
    }
}

pub struct EagleEyeHttpResponse {
    protocol_version: String,
    status: Status,
    content_type: Option<String>,
    content_length: Option<usize>,
    headers: Vec<(String, String)>,
}

impl Default for EagleEyeHttpResponse {
    fn default() -> Self {
        Self {
            protocol_version: "HTTP/1.1".to_owned(),
            status: Status::default(),
            content_type: None,
            content_length: None,
            headers: Vec::new(),
        }
    }
}

impl EagleEyeHttpResponse {
    pub fn protocol_version<T: Into<String>>(mut self, version: T) -> Self {
        self.protocol_version = version.into();
        self
    }
    pub fn get_protocol_version(&self) -> &str {
        self.protocol_version.as_str()
    }
    pub fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }
    pub fn get_status(&self) -> &Status {
        &self.status
    }
    pub fn content_type<T: Into<String>>(mut self, value: T) -> Self {
        self.content_type = Some(value.into());
        self
    }
    pub fn get_content_type(&self) -> Option<&str> {
        self.content_type.as_ref().map(|v| v.as_str())
    }
    pub fn content_length(mut self, value: usize) -> Self {
        self.content_length = Some(value);
        self
    }
    pub fn get_content_length(&self) -> Option<usize> {
        self.content_length
    }
    pub fn push_header<U: Into<String>, V: Into<String>>(mut self, key: U, value: V) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }
    pub fn get_header<T: AsRef<str>>(&self, key: T) -> Option<Cow<'_, str>> {
        let key = key.as_ref().trim();
        match key {
            "Content-Type" => self
                .content_type
                .as_ref()
                .map(|v| Cow::Borrowed(v.as_str())),
            "Content-Length" => self.content_length.map(|v| Cow::Owned(v.to_string())),
            v => self
                .headers
                .iter()
                .find(|&u| u.0 == v)
                .map(|u| Cow::Borrowed(u.0.as_str())),
        }
    }
    pub fn send<W: io::Write>(self, mut stream: W) -> io::Result<()> {
        write!(stream, "{} {}\r\n", self.protocol_version, self.status)?;
        if self.content_type.is_some() {
            write!(stream, "Content-Type: {}", self.content_type.unwrap())?;
        }
        for (key, value) in self.headers {
            write!(stream, "{}: {}\r\n", key, value)?;
        }
        write!(stream, "\r\n")?;
        stream.flush()
    }
    pub fn send_byte<W: io::Write, T: AsRef<[u8]>>(
        self,
        mut stream: W,
        value: T,
    ) -> io::Result<()> {
        let value = value.as_ref();
        write!(stream, "{} {}\r\n", self.protocol_version, self.status)?;
        write!(
            stream,
            "Content-Type: {}\r\n",
            self.content_type
                .as_ref()
                .map(|v| v.as_str())
                .unwrap_or("text/plain")
        )?;
        write!(
            stream,
            "Content-Length: {}\r\n",
            self.content_length.unwrap_or(value.len())
        )?;
        for (key, value) in self.headers {
            write!(stream, "{}: {}\r\n", key, value)?;
        }
        write!(stream, "\r\n")?;
        stream.flush()
    }
    pub fn send_file<W: io::Write, P: AsRef<Path>>(self, mut stream: W, path: P) -> io::Result<()> {
        let file = std::fs::File::open(path)?;
        let mut n = file.metadata()?.len() as usize;
        let file_reader = BufReader::new(file);
        Ok(())
    }
}

/*
pub struct EagleEyeHttpRequest<T> {
    method: Method,
    path: String,
    protocol_version: String,
    headers: Vec<(String, String)>,
    body_len: Option<usize>,
    stream: T,
}

impl<T: io::Read> io::Read for EagleEyeHttpRequest<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.body_len.is_none() {
            return self.stream.read(buf);
        }
        let len = self.body_len.as_mut().unwrap();
        if *len == 0 {
            Ok(0)
        } else {
            let buf_len = buf.len();
            let n = self
                .stream
                .read(&mut buf[0..std::cmp::min(*len, buf_len)])?;
            *len -= n;
            Ok(n)
        }
    }
}
*/

