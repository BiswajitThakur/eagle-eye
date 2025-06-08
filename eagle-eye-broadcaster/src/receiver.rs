use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

pub enum AddrType {
    Verified(SocketAddr),
    Unverified(SocketAddr),
}

pub struct ReceiverInfo<const S: usize> {
    prefix: Vec<u8>,
    is_running: Arc<AtomicBool>,
    block_ip: Vec<IpAddr>,
    block_ip_stack: Arc<Mutex<Vec<IpAddr>>>,
    //interval: Arc<Mutex<Duration>>,
    buf: [u8; S],
    recv_buf_len: Vec<usize>,
    socket: Option<UdpSocket>,
}

impl<const S: usize> ReceiverInfo<S> {
    pub fn builder() -> ReceiverInfoBuilder<S> {
        ReceiverInfoBuilder::default()
    }
}

pub struct ReceiverInfoBuilder<const S: usize> {
    prefix: Vec<u8>,
    is_running: Arc<AtomicBool>,
    block_ip: Vec<IpAddr>,
    block_ip_stack: Arc<Mutex<Vec<IpAddr>>>,
    //interval: Arc<Mutex<Duration>>,
    buf: [u8; S],
    recv_buf_len: Vec<usize>,
    socket: Option<UdpSocket>,
}

impl<const S: usize> Default for ReceiverInfoBuilder<S> {
    fn default() -> Self {
        Self {
            prefix: Vec::new(),
            is_running: Arc::new(AtomicBool::new(true)),
            block_ip: Vec::new(),
            block_ip_stack: Arc::new(Mutex::new(Vec::new())),
            buf: [0; S],
            recv_buf_len: Vec::new(),
            socket: None,
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
        self.block_ip_stack = ips;
        self
    }
    pub fn buffer(mut self, buf: [u8; S]) -> Self {
        self.buf = buf;
        self
    }
    pub fn recv_buf_len(mut self, l: Vec<usize>) -> Self {
        self.recv_buf_len = l;
        self
    }
    pub fn socket(mut self, s: UdpSocket) -> Self {
        self.socket = Some(s);
        self
    }
    pub fn build(self) -> ReceiverInfo<S> {
        ReceiverInfo {
            prefix: self.prefix,
            is_running: self.is_running,
            block_ip: self.block_ip,
            block_ip_stack: self.block_ip_stack,
            buf: self.buf,
            recv_buf_len: self.recv_buf_len,
            socket: self.socket,
        }
    }
}

impl<const S: usize> Iterator for ReceiverInfo<S> {
    type Item = io::Result<AddrType>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.socket.is_none() {
            return None;
        }
        let socket = self.socket.as_ref().unwrap();
        loop {
            if !self.is_running.load(Ordering::Relaxed) {
                return None;
            }
            let mut b = self.block_ip_stack.lock().unwrap();
            while let Some(addr) = b.pop() {
                self.block_ip.push(addr);
            }
            match socket.recv_from(&mut self.buf) {
                Ok((total, addr)) if self.recv_buf_len.contains(&total) => {
                    if self.block_ip.contains(&addr.ip()) {
                        continue;
                    }
                    if !self.buf.starts_with(&self.prefix) {
                        return Some(Ok(AddrType::Unverified(addr)));
                    }
                    let buf = self.buf;
                    let p = self.prefix.len();
                    let version = u16::from_be_bytes([buf[p], buf[p + 1]]);
                    let _flags = u16::from_be_bytes([buf[p + 2], buf[p + 3]]);
                    let port = u16::from_be_bytes([buf[p + 4], buf[p + 5]]);
                    match version {
                        4 => {
                            let bits = u32::from_be_bytes([
                                buf[p + 6],
                                buf[p + 7],
                                buf[p + 8],
                                buf[p + 9],
                            ]);
                            let ipv4 = Ipv4Addr::from_bits(bits);
                            let ip = if ipv4 == Ipv4Addr::new(0, 0, 0, 0) {
                                addr.ip()
                            } else {
                                IpAddr::V4(ipv4)
                            };
                            return Some(Ok(AddrType::Verified(SocketAddr::new(ip, port))));
                        }
                        6 => {
                            let mut addr_byte = [0; 16];
                            for (index, &v) in (buf[p + 6..p + 6 + 16]).iter().enumerate() {
                                addr_byte[index] = v;
                            }
                            let bits = u128::from_be_bytes(addr_byte);
                            let ipv6 = Ipv6Addr::from_bits(bits);
                            return Some(Ok(AddrType::Verified(SocketAddr::new(
                                IpAddr::V6(ipv6),
                                port,
                            ))));
                        }
                        _ => continue, // invalid version, ignore it
                    }
                }
                Ok((_, addr)) => return Some(Ok(AddrType::Unverified(addr))),
                Err(err) => return Some(Err(err)),
            }
        }
    }
}
