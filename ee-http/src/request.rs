use std::collections::HashMap;

use crate::Method;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct HttpRequest {
    method: Method,
    path: String,
    protocol_version: String,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

impl HttpRequest {
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }
    pub fn set_method(&mut self, method: Method) -> &mut Self {
        self.method = method;
        self
    }
    pub fn get_method(&self) -> Method {
        self.method
    }
    pub fn path<T: Into<String>>(mut self, path: T) -> Self {
        self.path = path.into();
        self
    }
    pub fn set_path<T: Into<String>>(&mut self, path: T) -> &mut Self {
        self.path = path.into();
        self
    }
    pub fn get_path(&self) -> &str {
        self.path.as_str()
    }
    pub fn version<T: Into<String>>(mut self, version: T) -> Self {
        self.protocol_version = version.into();
        self
    }
    pub fn set_version<T: Into<String>>(&mut self, version: T) -> &mut Self {
        self.protocol_version = version.into();
        self
    }
    pub fn get_version(&self) -> &str {
        self.protocol_version.as_str()
    }
    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }
    pub fn header<U: Into<String>, V: Into<String>>(mut self, key: U, value: V) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
    pub fn set_header<U: Into<String>, V: Into<String>>(&mut self, key: U, value: V) -> &mut Self {
        self.headers.insert(key.into(), value.into());
        self
    }
    pub fn get_header<T: AsRef<str>>(&self, key: T) -> Option<&str> {
        self.headers.get(key.as_ref()).map(|v| v.as_str())
    }
    pub fn body(mut self, body: Option<Vec<u8>>) -> Self {
        self.body = body;
        self
    }
    pub fn set_body(&mut self, body: Vec<u8>) -> &mut Self {
        self.body = Some(body);
        self
    }
    pub fn get_body(&self) -> Option<&[u8]> {
        self.body.as_ref().map(|v| v.as_slice())
    }
}

/*
impl HttpRequest {
    pub fn from_reader<R: io::BufRead>(mut reader: R) -> io::Result<Self> {
        let unexpected_eof_err = || {
            io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Unexpected EOF while parsing request",
            )
        };

        // Parse method
        let mut method = Vec::new();
        if Self::collect_while(&mut reader, |b| b != b' ', &mut method, 12)?.is_none() {
            return Err(unexpected_eof_err());
        }

        // Skip space
        Self::consume_byte(&mut reader, b' ')?;

        // Parse path
        let mut path = Vec::new();
        if Self::collect_while(&mut reader, |b| b != b' ', &mut path, 4096)?.is_none() {
            return Err(unexpected_eof_err());
        }

        // Skip space
        Self::consume_byte(&mut reader, b' ')?;

        // Parse version
        let mut version = Vec::new();
        if Self::collect_while(&mut reader, |b| b != b'\r', &mut version, 16)?.is_none() {
            return Err(unexpected_eof_err());
        }

        // Consume '\r\n'
        Self::consume_byte(&mut reader, b'\r')?;
        Self::consume_byte(&mut reader, b'\n')?;

        // Parse headers
        let mut headers = HashMap::new();
        loop {
            let mut line = String::new();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 {
                return Err(unexpected_eof_err());
            }
            if line == "\r\n" {
                break; // empty line = end of headers
            }
            if let Some((name, value)) = line.split_once(':') {
                headers.insert(
                    name.trim().to_string(),
                    value.trim_start().trim_end_matches("\r\n").to_string(),
                );
            }
        }

        // Parse body (if Content-Length exists)
        let body = if let Some(len_str) = headers.get("Content-Length") {
            let len = len_str.parse::<usize>().unwrap_or(0);
            let mut buffer = vec![0u8; len];
            reader.read_exact(&mut buffer)?;
            Some(buffer)
        } else {
            None
        };

        Ok(Self {
            method: String::from_utf8(method).unwrap_or_default(),
            path: String::from_utf8(path).unwrap_or_default(),
            protocol_version: String::from_utf8(version).unwrap_or_default(),
            headers,
            body,
        })
    }

    fn consume_byte<R: io::BufRead>(reader: &mut R, expected: u8) -> io::Result<()> {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        if buf[0] != expected {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Expected byte '{}', got '{}'", expected, buf[0]),
            ))
        } else {
            Ok(())
        }
    }

    fn collect_while<R: io::Read, F: Fn(u8) -> bool>(
        mut reader: R,
        f: F,
        v: &mut Vec<u8>,
        mut max_consume: usize,
    ) -> io::Result<Option<u8>> {
        let mut buf = [0u8; 1];
        while max_consume != 0 {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                return Ok(None);
            }
            if !f(buf[0]) {
                return Ok(Some(buf[0]));
            }
            v.push(buf[0]);
            max_consume -= 1;
        }
        Ok(Some(buf[0]))
    }
}
*/
