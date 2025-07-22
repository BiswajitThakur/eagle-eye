use std::{
    io::{self, Write},
    path::PathBuf,
};

use ee_stream::{EStreamSync, FlowControl};
use ee_task::TaskRegisterySync;

use crate::utils::{handle_auth_on_receiver_sync, write_log_sync};

pub struct EagleEyeServerSync<T> {
    id: u128,
    key: [u8; 32],
    handler: TaskRegisterySync<T>,
    log: Option<PathBuf>,
}

impl<T> Default for EagleEyeServerSync<T> {
    fn default() -> Self {
        Self {
            id: 0,
            key: [0; 32],
            handler: TaskRegisterySync::new(),
            log: None,
        }
    }
}

impl<T> EagleEyeServerSync<T> {
    pub fn new(key: [u8; 32], handler: TaskRegisterySync<T>) -> Self {
        Self {
            id: 0,
            key,
            handler,
            log: None,
        }
    }
    pub fn key(mut self, key: [u8; 32]) -> Self {
        self.key = key;
        self
    }
    pub fn id(mut self, id: u128) -> Self {
        self.id = id;
        self
    }

    pub fn set_log_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.log = Some(path.into());
        self
    }
    pub fn register(&mut self, id: &'static str, f: fn(T) -> io::Result<T>) {
        self.handler.register(id, f);
    }
}

impl<const N: usize, R: io::Read, W: io::Write> EagleEyeServerSync<EStreamSync<N, R, W>> {
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
    pub fn handle_stream(&self, r: R, w: W) -> io::Result<()> {
        let mut e_stream = match handle_auth_on_receiver_sync::<N, R, W>(self.key, r, w) {
            Ok(None) => return Ok(()),
            Ok(Some(v)) => v,
            Err(err) => {
                write_log_sync(self.log.as_ref(), err);
                return Ok(());
            }
        };
        loop {
            let id = Self::read_id(&mut e_stream).unwrap();
            match id.as_str() {
                ":end:" => break,
                ":stop-server:" => break,
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
        Ok(())
    }
}
