use std::{
    io::{self, Read},
    net::{SocketAddr, TcpListener, TcpStream},
    ops::ControlFlow,
    path::PathBuf,
};

use crate::{
    FlowControl,
    stream::EagleEyeStreamSync,
    task::TaskRegisterySync,
    utils::{handle_auth_on_server_sync, write_log_sync},
};

pub struct EagleEyeListenerSync<T> {
    key: [u8; 32],
    handler: TaskRegisterySync<T>,
    log: Option<PathBuf>,
}

impl<T> EagleEyeListenerSync<T> {
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

impl<const N: usize, R: io::Read, W: io::Write> EagleEyeListenerSync<EagleEyeStreamSync<N, R, W>> {
    fn get_id<U: io::Read + io::Write>(mut stream: U) -> io::Result<String> {
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
    pub fn run_server<I: Iterator<Item = io::Result<(R, W)>>>(
        &self,
        incoming: I,
    ) -> io::Result<()> {
        let mut is_exit = false;
        for stream_result in incoming {
            match stream_result {
                Ok((stream1, stream2)) => {
                    let mut e_stream =
                        handle_auth_on_server_sync::<N, R, W>(self.key, stream1, stream2)
                            .unwrap()
                            .unwrap();
                    let mut flow;
                    loop {
                        let id = Self::get_id(&mut e_stream).unwrap();
                        if id.is_empty() {
                            break;
                        }
                        match id.as_str() {
                            ":end:" => break,
                            ":stop-server:" => {
                                is_exit = true;
                                break;
                            }
                            _ => {}
                        }
                        let fun = self.handler.get(id);
                        (flow, e_stream) = fun(e_stream).unwrap();
                        match flow {
                            FlowControl::Close => break,
                            FlowControl::Continue => continue,
                            FlowControl::StopServer => {
                                is_exit = true;
                                break;
                            }
                        }
                    }
                }
                Err(err) => write_log_sync(self.log.as_ref(), err),
            }
            if is_exit {
                break;
            }
        }
        Ok(())
    }
}
