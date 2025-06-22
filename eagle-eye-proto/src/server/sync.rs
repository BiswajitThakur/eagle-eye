use std::{io, path::PathBuf};

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
    pub fn handle_stream<S: AsRef<str>>(&self, id: S, stream: T) -> io::Result<(FlowControl, T)> {
        let fun = self.handler.get(id);
        fun(stream)
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
    fn is_know_task<U: io::Read>(mut stream: U) -> io::Result<bool> {
        let mut buf = [0; 1];
        stream.read_exact(&mut buf)?;
        let flow = FlowControl::try_from(buf);
        if flow.is_err() {
            return Err(io::Error::other("Invalid Flow"));
        }
        Ok(flow.unwrap() == FlowControl::Continue)
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
                    let mut flow;
                    loop {
                        let id = Self::read_id(&mut e_stream).unwrap();
                        if id.is_empty() {
                            break;
                        }
                        match id.as_str() {
                            ":end:" => break,
                            ":stop-server:" => {
                                should_stop_server = true;
                                break;
                            }
                            _ => {}
                        }
                        /*
                        let is_knon_task = match Self::is_know_task(&mut e_stream) {
                            Ok(v) => v,
                            Err(err) => {
                                write_log_sync(self.log.as_ref(), err);
                                break;
                            }
                        };
                        dbg!(is_knon_task);
                        if !is_knon_task {
                            break;
                        }*/
                        let fun = self.handler.get(id);
                        (flow, e_stream) = match fun(e_stream) {
                            Ok((u, v)) => (u, v),
                            Err(err) => {
                                write_log_sync(self.log.as_ref(), err);
                                break;
                            }
                        };
                        match flow {
                            FlowControl::Close => break,
                            FlowControl::Continue => continue,
                            FlowControl::StopServer => {
                                should_stop_server = true;
                                break;
                            }
                        }
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
