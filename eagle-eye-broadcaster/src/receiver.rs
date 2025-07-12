use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

pub struct ReceiverInfo<const S: usize> {
    prefix: Vec<u8>,
    is_running: Arc<AtomicBool>,
    block_ip: Arc<Mutex<Vec<IpAddr>>>,
    buf: [u8; S],
    socket: UdpSocket,
}

impl<const S: usize> ReceiverInfo<S> {
    pub fn builder() -> ReceiverInfoBuilder<S> {
        ReceiverInfoBuilder::default()
    }
}

pub struct ReceiverInfoBuilder<const S: usize> {
    prefix: Vec<u8>,
    is_running: Arc<AtomicBool>,
    block_ip: Arc<Mutex<Vec<IpAddr>>>,
    time_out: Option<Duration>,
    buf: [u8; S],
    socket_addr: SocketAddr,
}

impl<const S: usize> Default for ReceiverInfoBuilder<S> {
    fn default() -> Self {
        Self {
            prefix: Vec::new(),
            is_running: Arc::new(AtomicBool::new(true)),
            block_ip: Arc::new(Mutex::new(Vec::new())),
            time_out: None,
            buf: [0; S],
            socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
        }
    }
}

impl<const S: usize> ReceiverInfoBuilder<S> {
    pub fn prefix<T: Into<Vec<u8>>>(mut self, value: T) -> Self {
        self.prefix = value.into();
        self
    }
    pub fn is_running(mut self, r: Arc<AtomicBool>) -> Self {
        self.is_running = r;
        self
    }
    pub fn block_ip(mut self, ips: Arc<Mutex<Vec<IpAddr>>>) -> Self {
        self.block_ip = ips;
        self
    }
    pub fn buffer(mut self, buf: [u8; S]) -> Self {
        self.buf = buf;
        self
    }
    pub fn socket_addr(mut self, addr: SocketAddr) -> Self {
        self.socket_addr = addr;
        self
    }
    pub fn build(self) -> io::Result<ReceiverInfo<S>> {
        let socket = UdpSocket::bind(self.socket_addr)?;
        socket.set_read_timeout(self.time_out)?;
        Ok(ReceiverInfo {
            prefix: self.prefix,
            is_running: self.is_running,
            block_ip: self.block_ip,
            buf: self.buf,
            socket,
        })
    }
}

impl<const S: usize> Iterator for ReceiverInfo<S> {
    type Item = io::Result<(SocketAddr, Vec<u8>)>;
    fn next(&mut self) -> Option<Self::Item> {
        let socket = &self.socket;
        loop {
            if !self.is_running.load(Ordering::Relaxed) {
                return None;
            }
            let b = self.block_ip.lock().unwrap();
            match socket.recv_from(&mut self.buf) {
                Ok((total, addr)) if self.buf.starts_with(&self.prefix) => {
                    if b.contains(&addr.ip()) {
                        continue;
                    }
                    return Some(Ok((addr, self.buf[self.prefix.len()..total].to_vec())));
                }
                Ok(_) => {
                    if !self.is_running.load(Ordering::Relaxed) {
                        return None;
                    }
                    continue;
                }
                Err(err) => return Some(Err(err)),
            }
        }
    }
}
