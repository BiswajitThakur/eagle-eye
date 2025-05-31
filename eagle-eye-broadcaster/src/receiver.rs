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
    pub prefix: Vec<u8>,
    pub is_running: Arc<AtomicBool>,
    pub block_ip: Vec<IpAddr>,
    pub block_ip_stack: Arc<Mutex<Vec<IpAddr>>>,
    //interval: Arc<Mutex<Duration>>,
    pub buf: [u8; S],
    pub recv_buf_len: Vec<usize>,
    pub socket: UdpSocket,
}

impl<const S: usize> Iterator for ReceiverInfo<S> {
    type Item = io::Result<AddrType>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if !self.is_running.load(Ordering::Relaxed) {
                return None;
            }
            let mut b = self.block_ip_stack.lock().unwrap();
            while let Some(addr) = b.pop() {
                self.block_ip.push(addr);
            }
            match self.socket.recv_from(&mut self.buf) {
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
                            return Some(Ok(AddrType::Verified(SocketAddr::new(
                                IpAddr::V4(ipv4),
                                port,
                            ))));
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
