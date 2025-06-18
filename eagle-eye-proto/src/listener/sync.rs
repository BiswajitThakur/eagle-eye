use std::{
    io,
    net::{SocketAddr, TcpListener},
    path::PathBuf,
};

use crate::{job::JobsSync, utils::write_log_sync};

pub struct EagleEyeListenerSync {
    key: [u8; 32],
    log: Option<PathBuf>,
}

impl EagleEyeListenerSync {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key, log: None }
    }
    pub fn set_log_path(mut self, path: PathBuf) -> Self {
        self.log = Some(path);
        self
    }
    pub fn run<const N: usize>(self, addr: SocketAddr, jobs: JobsSync<N>) -> io::Result<()> {
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    jobs.run_server(self.key, s)?;
                }
                Err(err) => {
                    write_log_sync(self.log.as_ref(), err);
                }
            }
        }
        Ok(())
    }
}
