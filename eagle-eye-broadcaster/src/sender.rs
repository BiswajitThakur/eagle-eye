use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

pub struct SenderInfo {
    prefix: Vec<u8>,
    socket_addr: SocketAddr,
    broadcast_addr: SocketAddr,
    interval: Option<Arc<AtomicU64>>,
    flags: u16, // for future uses
    send_addr: SocketAddr,
}

impl SenderInfo {
    pub fn builder() -> SenderInfoBuilder {
        SenderInfoBuilder::default()
    }
    pub fn send(self, is_running: Arc<AtomicBool>) -> io::Result<()> {
        let Self {
            prefix,
            interval,
            socket_addr,
            broadcast_addr,
            flags,
            send_addr,
        } = self;
        let version: u16 = match send_addr.ip() {
            IpAddr::V4(_) => 4,
            IpAddr::V6(_) => 6,
        };
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&prefix);
        buffer.extend_from_slice(&version.to_be_bytes());
        buffer.extend_from_slice(&flags.to_be_bytes());
        buffer.extend_from_slice(&send_addr.port().to_be_bytes());
        match send_addr.ip() {
            IpAddr::V4(ipv4) => {
                buffer.extend_from_slice(&ipv4.to_bits().to_be_bytes());
            }
            IpAddr::V6(ipv6) => {
                buffer.extend_from_slice(&ipv6.to_bits().to_be_bytes());
            }
        }
        let socket = UdpSocket::bind(socket_addr)?;
        socket.set_broadcast(true)?;
        loop {
            if !is_running.load(Ordering::Relaxed) {
                break;
            }
            socket.send_to(&buffer, broadcast_addr)?;
            match interval {
                Some(ref interval) => {
                    let millis = interval.load(Ordering::Relaxed);
                    std::thread::sleep(Duration::from_millis(millis));
                }
                None => std::thread::sleep(Duration::from_secs(3)),
            }
        }
        Ok(())
    }
}

pub struct SenderInfoBuilder {
    prefix: Vec<u8>,
    socket_addr: SocketAddr,
    broadcast_addr: SocketAddr,
    interval: Option<Arc<AtomicU64>>,
    send_addr: SocketAddr,
}

impl Default for SenderInfoBuilder {
    fn default() -> Self {
        Self {
            prefix: Vec::new(),
            socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            broadcast_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)), 7511),
            interval: None,
            send_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
        }
    }
}

impl SenderInfoBuilder {
    pub fn prefix(mut self, prefix: Vec<u8>) -> Self {
        self.prefix = prefix;
        self
    }
    pub fn interval(mut self, time: Arc<AtomicU64>) -> Self {
        self.interval = Some(time);
        self
    }
    pub fn send_addr(mut self, addr: SocketAddr) -> Self {
        self.send_addr = addr;
        self
    }
    pub fn socket_addr(mut self, addr: SocketAddr) -> Self {
        self.socket_addr = addr;
        self
    }
    pub fn broadcast_addr(mut self, addr: SocketAddr) -> Self {
        self.broadcast_addr = addr;
        self
    }
    pub fn build(self) -> SenderInfo {
        SenderInfo {
            prefix: self.prefix,
            socket_addr: self.socket_addr,
            broadcast_addr: self.broadcast_addr,
            interval: self.interval,
            flags: 0,
            send_addr: self.send_addr,
        }
    }
}
