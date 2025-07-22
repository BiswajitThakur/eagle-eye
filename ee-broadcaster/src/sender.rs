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
    data: Vec<u8>,
    is_running: Arc<AtomicBool>,
    socket_addr: SocketAddr,
    broadcast_addr: SocketAddr,
    interval: Option<Arc<AtomicU64>>,
}

impl SenderInfo {
    pub fn builder() -> SenderInfoBuilder {
        SenderInfoBuilder::default()
    }
    pub fn send(&self) -> io::Result<()> {
        let Self {
            prefix,
            data,
            is_running,
            interval,
            socket_addr,
            broadcast_addr,
        } = self;
        let mut buffer = Vec::with_capacity(prefix.len() + data.len());
        buffer.extend_from_slice(prefix);
        buffer.extend_from_slice(data);
        let socket = UdpSocket::bind(socket_addr)?;
        socket.set_broadcast(true)?;
        loop {
            if !is_running.load(Ordering::Relaxed) {
                break;
            }
            socket.send_to(&buffer, broadcast_addr)?;
            match interval {
                Some(interval) => {
                    let millis = interval.load(Ordering::Relaxed);
                    std::thread::sleep(Duration::from_millis(millis));
                }
                None => std::thread::sleep(Duration::from_millis(300)),
            }
        }
        Ok(())
    }
}

pub struct SenderInfoBuilder {
    prefix: Vec<u8>,
    data: Vec<u8>,
    is_running: Arc<AtomicBool>,
    socket_addr: SocketAddr,
    broadcast_addr: SocketAddr,
    interval: Option<Arc<AtomicU64>>,
}

impl Default for SenderInfoBuilder {
    fn default() -> Self {
        Self {
            prefix: Vec::new(),
            data: Vec::new(),
            is_running: Arc::new(AtomicBool::new(true)),
            socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            broadcast_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)), 7511),
            interval: None,
        }
    }
}

impl SenderInfoBuilder {
    pub fn prefix<T: Into<Vec<u8>>>(mut self, prefix: T) -> Self {
        self.prefix = prefix.into();
        self
    }
    pub fn data<T: AsRef<[u8]>>(mut self, data: T) -> Self {
        self.data.extend_from_slice(data.as_ref());
        self
    }
    pub fn is_running(mut self, is_running: Arc<AtomicBool>) -> Self {
        self.is_running = is_running;
        self
    }
    pub fn interval(mut self, time: Arc<AtomicU64>) -> Self {
        self.interval = Some(time);
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
            data: self.data,
            is_running: self.is_running,
            socket_addr: self.socket_addr,
            broadcast_addr: self.broadcast_addr,
            interval: self.interval,
        }
    }
}
