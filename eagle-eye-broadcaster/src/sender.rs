use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
    time::Duration,
};

pub struct SenderInfo {
    prefix: Vec<u8>,
    socket_addr: SocketAddr,
    broadcast_addr: SocketAddr,
    interval: Option<Arc<Mutex<Duration>>>,
    flags: u16, // for future uses
    port: u16,
    addr: IpAddr,
}

impl SenderInfo {
    pub fn builder() -> SenderInfoBuilder {
        SenderInfoBuilder::default()
    }
    pub fn send(self, is_running: Arc<Mutex<bool>>) -> io::Result<()> {
        let Self {
            prefix,
            interval,
            socket_addr,
            broadcast_addr,
            flags,
            port,
            addr,
        } = self;
        let version: u16 = match addr {
            IpAddr::V4(_) => 4,
            IpAddr::V6(_) => 6,
        };
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&prefix);
        buffer.extend_from_slice(&version.to_be_bytes());
        buffer.extend_from_slice(&flags.to_be_bytes());
        buffer.extend_from_slice(&port.to_be_bytes());
        match addr {
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
            let v = *is_running.lock().unwrap();
            if !v {
                break;
            }
            socket.send_to(&buffer, broadcast_addr)?;
            match interval {
                Some(ref time) => {
                    let t = *time.lock().unwrap();
                    std::thread::sleep(t);
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
    interval: Option<Arc<Mutex<Duration>>>,
    port: u16,
    addr: IpAddr,
}

impl Default for SenderInfoBuilder {
    fn default() -> Self {
        Self {
            prefix: Vec::new(),
            socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            broadcast_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)), 7511),
            interval: None,
            port: 0,
            addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
        }
    }
}

impl SenderInfoBuilder {
    pub fn prefix(mut self, prefix: Vec<u8>) -> Self {
        self.prefix = prefix;
        self
    }
    pub fn interval(mut self, time: Arc<Mutex<Duration>>) -> Self {
        self.interval = Some(time);
        self
    }
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    pub fn addr(mut self, addr: IpAddr) -> Self {
        self.addr = addr;
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
            port: self.port,
            addr: self.addr,
        }
    }
}
