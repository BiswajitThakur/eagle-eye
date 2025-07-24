use std::{
    borrow::Cow,
    ffi::OsStr,
    fmt,
    io::{self, BufReader},
    path::Path,
};

use crate::Status;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    protocol_version: String,
    status: Status,
    content_type: Option<String>,
    content_length: Option<usize>,
    headers: Vec<(String, String)>,
}

impl Default for HttpResponse {
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

impl HttpResponse {
    pub fn new() -> Self {
        Self::default()
    }
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
        self.content_type.as_deref()
    }
    pub fn content_length(mut self, value: usize) -> Self {
        self.content_length = Some(value);
        self
    }
    pub fn get_content_length(&self) -> Option<usize> {
        self.content_length
    }
    pub fn push_header<U: Into<String>, V: Into<String>>(mut self, key: U, value: V) -> Self {
        let key = key.into();
        let value = value.into();
        let index = self
            .headers
            .iter()
            .enumerate()
            .find(|&(_, (k, _))| k == &key)
            .map(|(i, _)| i);
        match index {
            Some(v) => {
                self.headers[v] = (key, value);
            }
            None => {
                self.headers.push((key, value));
            }
        }
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
            write!(stream, "{key}: {value}\r\n")?;
        }
        write!(stream, "\r\n")?;
        stream.flush()
    }
    pub fn send_str<W: io::Write, T: AsRef<str>>(self, mut stream: W, value: T) -> io::Result<()> {
        let bytes = value.as_ref().as_bytes();
        write!(stream, "{} {}\r\n", self.protocol_version, self.status)?;
        write!(
            stream,
            "Content-Type: {}\r\n",
            self.content_type.as_deref().unwrap_or("text/plain")
        )?;
        write!(
            stream,
            "Content-Length: {}\r\n",
            self.content_length.unwrap_or(bytes.len())
        )?;
        for (key, value) in self.headers {
            write!(stream, "{key}: {value}\r\n")?;
        }
        write!(stream, "\r\n")?;
        stream.write_all(bytes)?;
        stream.flush()
    }
    pub fn send_file<W: io::Write, P: AsRef<Path>>(self, mut stream: W, path: P) -> io::Result<()> {
        let path = path.as_ref();
        let ext = path.extension();
        let file = std::fs::File::open(path)?;
        let n = file.metadata()?.len();
        let mut file_reader = BufReader::new(file);
        write!(stream, "{} {}\r\n", self.protocol_version, self.status)?;
        write!(
            stream,
            "Content-Type: {}\r\n",
            match ext {
                // text
                Some(v) if v == OsStr::new("html") => "text/html",
                Some(v) if v == OsStr::new("htm") => "text/html",
                Some(v) if v == OsStr::new("css") => "text/css",
                Some(v) if v == OsStr::new("js") => "application/javascript",
                Some(v) if v == OsStr::new("json") => "application/json",
                Some(v) if v == OsStr::new("txt") => "text/plain",
                Some(v) if v == OsStr::new("csv") => "text/csv",
                Some(v) if v == OsStr::new("xml") => "application/xml",

                // image
                Some(v) if v == OsStr::new("jpg") => "image/jpeg",
                Some(v) if v == OsStr::new("jpeg") => "image/jpeg",
                Some(v) if v == OsStr::new("png") => "image/png",
                Some(v) if v == OsStr::new("gif") => "image/gif",
                Some(v) if v == OsStr::new("webp") => "image/webp",
                Some(v) if v == OsStr::new("svg") => "image/svg+xml",
                Some(v) if v == OsStr::new("ico") => "image/x-icon",

                // audio
                Some(v) if v == OsStr::new("mp3") => "audio/mpeg",
                Some(v) if v == OsStr::new("wav") => "audio/wav",
                Some(v) if v == OsStr::new("ogg") => "audio/ogg",
                Some(v) if v == OsStr::new("m4a") => "audio/mp4",

                // video
                Some(v) if v == OsStr::new("mp4") => "video/mp4",
                Some(v) if v == OsStr::new("webm") => "video/webm",
                Some(v) if v == OsStr::new("ogg") => "video/ogg",
                Some(v) if v == OsStr::new("mov") => "video/quicktime",

                // executable
                Some(v) if v == OsStr::new("wasm") => "application/wasm",
                Some(v) if v == OsStr::new("sh") => "application/x-sh",

                // unknown
                _ => "application/octet-stream",
            }
        )?;
        write!(stream, "Content-Length: {n}\r\n")?;
        for (key, value) in self.headers {
            write!(stream, "{key}: {value}\r\n")?;
        }
        write!(stream, "\r\n")?;
        std::io::copy(&mut file_reader, &mut stream)?;
        Ok(())
    }
}
