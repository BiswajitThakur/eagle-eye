use std::{
    io,
    net::{IpAddr, SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
    time::Duration,
};

pub struct ReceiverInfo {
    prefix: Vec<u8>,
    block_ip: Vec<IpAddr>,
    interval: Arc<Mutex<Duration>>,
    buf: Vec<u8>,
    recv_buf_len: usize,
    socket: UdpSocket,
}

impl ReceiverInfo {}

impl Iterator for ReceiverInfo {
    type Item = io::Result<SocketAddr>;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
        /*
        loop {
            let t = *self.interval.lock().unwrap();
            match self.socket.recv(&mut self.buf) {
                Ok(total) if total == self.buf_len => {
                    std::thread::sleep(t);
                }
                Ok(_) => std::thread::sleep(t),
                Err(err) => return Some(Err(err)),
            }
        }*/
    }
}
