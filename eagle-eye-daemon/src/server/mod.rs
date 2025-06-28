use std::{
    io,
    net::{SocketAddr, TcpListener},
};

use eagle_eye_proto::{
    server::EagleEyeServerSync, stream::EagleEyeStreamSync, task::TaskRegisterySync,
};

pub struct EagleEyeDaemon<T> {
    id: [u8; 32],
    key: [u8; 32],
    name: String,
    addr: SocketAddr,
    jobs: TaskRegisterySync<T>,
    server: EagleEyeServerSync<T>,
}

impl<T> EagleEyeDaemon<T> {
    pub fn register(&mut self, id: &'static str, f: fn(T) -> io::Result<T>) {
        self.jobs.register(id, f);
    }

    /*
    pub fn run<I: Iterator<Item = io::Result<(R, W)>>>(&self, incoming: I) -> io::Result<()> {

        todo!()
    }*/
}
