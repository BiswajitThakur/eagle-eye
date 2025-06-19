use std::{
    io,
    net::{SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
};

use crate::{
    Connection,
    stream::EagleEyeStreamSync,
    task::sync::ServerTaskSync,
    utils::{handle_stream_server_sync, write_log_sync},
};

pub struct EagleEyeListenerSync<T: io::Read + io::Write> {
    key: [u8; 32],
    log: Option<PathBuf>,
    tasks: Vec<Box<dyn ServerTaskSync<T>>>,
}

impl<T: io::Read + io::Write> EagleEyeListenerSync<T> {
    pub fn new(key: [u8; 32]) -> Self {
        Self {
            key,
            log: None,
            tasks: Vec::new(),
        }
    }
    pub fn set_log_path(mut self, path: PathBuf) -> Self {
        self.log = Some(path);
        self
    }
}

impl<const N: usize> EagleEyeListenerSync<&mut EagleEyeStreamSync<N, &TcpStream, &TcpStream>> {
    pub fn run(self, addr: SocketAddr) -> io::Result<()> {
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    let mut e_stream = handle_stream_server_sync::<N>(self.key, &s)?.unwrap();
                    loop {
                        let task = self.tasks.get(0).unwrap();
                        match e_stream.execute_task(task) {
                            Ok(Connection::Close) => break,
                            Ok(Connection::Continue) => continue,
                            Err(err) => write_log_sync(self.log.as_ref(), err),
                        }
                    }
                }
                Err(err) => {
                    write_log_sync(self.log.as_ref(), err);
                }
            }
        }
        Ok(())
    }
}
