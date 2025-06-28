use std::{
    io::{self, Write},
    path::PathBuf,
};

use crate::{
    FlowControl,
    stream::EagleEyeStreamSync,
    task::TaskRegisterySync,
    utils::{handle_auth_on_server_sync, write_log_sync},
};

pub struct EagleEyeServerSync<T> {
    key: [u8; 32],
    handler: TaskRegisterySync<T>,
    log: Option<PathBuf>,
}

impl<T> EagleEyeServerSync<T> {
    pub fn new(key: [u8; 32], handler: TaskRegisterySync<T>) -> Self {
        Self {
            key,
            handler,
            log: None,
        }
    }
    pub fn set_log_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.log = Some(path.into());
        self
    }
    pub fn register(&mut self, id: &'static str, f: fn(T) -> io::Result<T>) {
        self.handler.register(id, f);
    }
}

impl<const N: usize, R: io::Read, W: io::Write> EagleEyeServerSync<EagleEyeStreamSync<N, R, W>> {
    fn read_id<U: io::Read>(mut stream: U) -> io::Result<String> {
        let mut buf = [0; 1];
        let mut result = String::new();
        loop {
            let n = stream.read(&mut buf)?;
            if n == 0 || buf[0] == b'\n' {
                break;
            }
            result.push(buf[0] as char);
        }
        Ok(result)
    }
    pub fn run<I: Iterator<Item = io::Result<(R, W)>>>(&self, incoming: I) -> io::Result<()> {
        let mut should_stop_server = false;
        for stream_result in incoming {
            match stream_result {
                Ok((stream1, stream2)) => {
                    let mut e_stream =
                        match handle_auth_on_server_sync::<N, R, W>(self.key, stream1, stream2) {
                            Ok(None) => continue,
                            Ok(Some(v)) => v,
                            Err(err) => {
                                write_log_sync(self.log.as_ref(), err);
                                continue;
                            }
                        };
                    loop {
                        let id = Self::read_id(&mut e_stream).unwrap();
                        match id.as_str() {
                            ":end:" => break,
                            ":stop-server:" => {
                                should_stop_server = true;
                                break;
                            }
                            _ => {}
                        }
                        let fun = self.handler.get(id);
                        if fun.is_none() {
                            e_stream.write_all(&FlowControl::Close.to_be_bytes())?;
                            e_stream.flush()?;
                            continue;
                        } else {
                            e_stream.write_all(&FlowControl::Continue.to_be_bytes())?;
                            e_stream.flush()?;
                        };
                        e_stream = match fun.unwrap()(e_stream) {
                            Ok(v) => v,
                            Err(err) => {
                                write_log_sync(self.log.as_ref(), err);
                                break;
                            }
                        };
                    }
                }
                Err(err) => write_log_sync(self.log.as_ref(), err),
            }
            if should_stop_server {
                break;
            }
        }
        Ok(())
    }
}
