use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

pub struct ReceiverInfo {
    prefix: Vec<u8>,
    is_running: Arc<AtomicBool>,
    buf: Box<[u8]>,
    socket: UdpSocket,
}

impl ReceiverInfo {
    pub fn builder() -> ReceiverInfoBuilder {
        ReceiverInfoBuilder::default()
    }
    pub fn next(&mut self) -> io::Result<Option<(SocketAddr, &mut [u8])>> {
        let socket = &self.socket;
        loop {
            if !self.is_running.load(Ordering::Relaxed) {
                return Ok(None);
            }
            let v = socket.recv_from(&mut self.buf);
            dbg!("data received ( remove me )");
            match v {
                Ok((total, addr)) if self.buf.starts_with(&self.prefix) => {
                    return Ok(Some((addr, &mut self.buf[self.prefix.len()..total])));
                }
                Ok(_) => {
                    if !self.is_running.load(Ordering::Relaxed) {
                        return Ok(None);
                    }
                    continue;
                }
                Err(err) => return Err(err),
            }
        }
    }
}

pub struct ReceiverInfoBuilder {
    prefix: Vec<u8>,
    is_running: Arc<AtomicBool>,
    time_out: Option<Duration>,
    buf_size: Option<std::num::NonZero<usize>>,
    socket_addr: SocketAddr,
}

impl Default for ReceiverInfoBuilder {
    fn default() -> Self {
        Self {
            prefix: Vec::new(),
            is_running: Arc::new(AtomicBool::new(true)),
            time_out: None,
            buf_size: std::num::NonZero::new(8 * 1024), // 8kb
            socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
        }
    }
}

impl ReceiverInfoBuilder {
    pub fn prefix<T: Into<Vec<u8>>>(mut self, value: T) -> Self {
        self.prefix = value.into();
        self
    }
    pub fn is_running(mut self, r: Arc<AtomicBool>) -> Self {
        self.is_running = r;
        self
    }
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buf_size = std::num::NonZero::new(size);
        self
    }
    pub fn socket_addr(mut self, addr: SocketAddr) -> Self {
        self.socket_addr = addr;
        self
    }
    pub fn build(self) -> io::Result<ReceiverInfo> {
        let buf_size = self.buf_size.expect("Buffer size can not be zero...").get();
        let v = Box::<[u8]>::new_uninit_slice(buf_size);
        let buf = unsafe { v.assume_init() };
        let socket = UdpSocket::bind(self.socket_addr)?;
        socket.set_read_timeout(self.time_out)?;
        Ok(ReceiverInfo {
            prefix: self.prefix,
            is_running: self.is_running,
            buf,
            socket,
        })
    }
}
