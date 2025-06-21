use std::{
    io,
    net::{SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
};

use crate::{
    FlowControl,
    stream::EagleEyeStreamSync,
    utils::{handle_stream_server_sync, write_log_sync},
};

pub struct EagleEyeListenerSync {
    key: [u8; 32],
    log: Option<PathBuf>,
}

impl EagleEyeListenerSync {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key, log: None }
    }
    pub fn set_log_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.log = Some(path.into());
        self
    }
}

impl EagleEyeListenerSync {
    pub fn run<const N: usize>(
        self,
        addr: SocketAddr,
        f: fn(&mut EagleEyeStreamSync<N, &TcpStream, &TcpStream>) -> io::Result<FlowControl>,
    ) -> io::Result<()> {
        let listener = TcpListener::bind(addr)?;
        let mut is_exit = false;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let mut e_stream = handle_stream_server_sync::<N>(self.key, &stream)?.unwrap();
                    loop {
                        match f(&mut e_stream) {
                            Ok(FlowControl::Continue) => continue,
                            Ok(FlowControl::Close) => break,
                            Ok(FlowControl::StopServer) => {
                                is_exit = true;
                                break;
                            }
                            Err(err) => write_log_sync(self.log.as_ref(), err),
                        };
                    }
                }
                Err(err) => {
                    write_log_sync(self.log.as_ref(), err);
                }
            }
            if is_exit {
                break;
            }
        }
        Ok(())
    }
}
