use std::{
    io::{self, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    path::PathBuf,
    sync::{Arc, atomic::AtomicUsize},
    time::Duration,
};

use ee_broadcaster::ReceiverInfo;
use ee_stream::{EStreamSync, FlowControl};
use ee_task::{ExeReceiverSync, create_task_registery};

use crate::utils::{handle_auth_on_receiver_sync, process_broadcast_data};

create_task_registery!(
    pub ReceiverTaskRegisterySync,
    for<'a> fn(
        EStreamSync<N, &'a TcpStream, &'a TcpStream>,
    ) -> io::Result<EStreamSync<N, &'a TcpStream, &'a TcpStream>>
);

pub struct ReceiverConfigSync<'a, const N: usize> {
    id: u128,
    key: [u8; 32],
    socket_addr: SocketAddr,
    broadcast_buf: Option<&'a mut [u8]>,
    broadcast_data_prefix: &'static str,
    max_connection: usize,
    handler: ReceiverTaskRegisterySync<N>,
    log: Option<PathBuf>,
}

impl<'a, const N: usize> Default for ReceiverConfigSync<'a, N> {
    fn default() -> Self {
        Self {
            id: 0,
            key: [0; 32],
            socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 6923),
            broadcast_buf: None,
            broadcast_data_prefix: "",
            max_connection: 4,
            handler: ReceiverTaskRegisterySync::default(),
            log: None,
        }
    }
}

impl<'a, const N: usize> ReceiverConfigSync<'a, N> {
    pub fn id(mut self, id: u128) -> Self {
        self.id = id;
        self
    }
    pub fn key(mut self, key: [u8; 32]) -> Self {
        self.key = key;
        self
    }
    pub fn set_log_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.log = Some(path.into());
        self
    }
    pub fn socket_addr(mut self, addr: SocketAddr) -> Self {
        self.socket_addr = addr;
        self
    }
    pub fn broadcast_buf(mut self, buf: &'a mut [u8]) -> Self {
        self.broadcast_buf = Some(buf);
        self
    }
    pub fn broadcast_data_prefix(mut self, prefix: &'static str) -> Self {
        self.broadcast_data_prefix = prefix;
        self
    }
    pub fn max_connection(mut self, v: usize) -> Self {
        self.max_connection = v;
        self
    }
    pub fn register<A: ExeReceiverSync<N>>(&mut self) -> &mut Self {
        self.handler.register(A::id(), A::execute_on_receiver);
        self
    }
}

pub struct AppSync<'a, const N: usize> {
    id: u128,
    key: [u8; 32],
    broadcast_socket_addr: SocketAddr,
    broadcast_buf: &'a mut [u8],
    broadcast_data_prefix: &'static str,
    max_connection: usize,
    count_connection: Arc<AtomicUsize>,
    log: Option<PathBuf>,
    handler: Arc<ReceiverTaskRegisterySync<N>>,
}

impl<'a, const N: usize> AppSync<'a, N> {
    pub fn new(config: ReceiverConfigSync<'a, N>) -> Self {
        let ReceiverConfigSync {
            id,
            key,
            socket_addr: broadcast_socket_addr,
            broadcast_buf,
            broadcast_data_prefix,
            max_connection,
            handler,
            log,
        } = config;
        Self {
            id,
            key,
            broadcast_socket_addr,
            broadcast_buf: broadcast_buf.expect("Buffer not found..."),
            broadcast_data_prefix,
            max_connection,
            count_connection: Arc::new(AtomicUsize::new(0)),
            log,
            handler: Arc::new(handler),
        }
    }
}

impl<const N: usize> AppSync<'_, N> {
    pub fn run(self) -> io::Result<()> {
        let mut receiver = ReceiverInfo::builder()
            .prefix(self.broadcast_data_prefix)
            .buffer(self.broadcast_buf)
            .socket_addr(self.broadcast_socket_addr)
            .build()?;
        while let Ok(Some((addr, data))) = receiver.next() {
            let count = self
                .count_connection
                .load(std::sync::atomic::Ordering::SeqCst);
            if count >= self.max_connection {
                std::thread::sleep(Duration::from_millis(300));
                continue;
            }
            if let Some((addr, secret)) = process_broadcast_data(self.key, self.id, addr, data) {
                self.count_connection
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let count = self.count_connection.clone();
                let key = self.key;
                let handler = self.handler.clone();
                std::thread::spawn(move || {
                    let stream = TcpStream::connect(addr);
                    if stream.is_err() {
                        count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                        return;
                    }
                    let mut stream = stream.unwrap();
                    if stream.write_all(&[0]).is_err() {
                        count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                        return;
                    }
                    if stream.write_all(&secret).is_err() {
                        count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                        return;
                    }
                    let _ = Self::handle_stream(key, handler, &stream, &stream);
                    count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                });
            } else {
                std::thread::sleep(Duration::from_millis(100));
            }
        }
        Ok(())
    }
}

impl<const N: usize> AppSync<'_, N> {
    fn read_task_id<U: io::Read>(mut stream: U) -> io::Result<String> {
        let mut buf = [0; 1];
        let mut result = String::new();
        loop {
            let n = stream.read(&mut buf)?;
            if n == 0 || buf[0] == b'\n' {
                break;
            }
            result.push(buf[0] as char);
        }
        Ok(result)
    }
    pub fn handle_stream(
        key: [u8; 32],
        handler: Arc<ReceiverTaskRegisterySync<N>>,
        r: &TcpStream,
        w: &TcpStream,
    ) -> io::Result<()> {
        let e_stream = handle_auth_on_receiver_sync::<N, _, _>(key, r, w)?;
        if e_stream.is_none() {
            return Ok(());
        }
        let mut e_stream = e_stream.unwrap();
        loop {
            let id = Self::read_task_id(&mut e_stream).unwrap();
            if matches!(id.as_str(), ":end:" | ":stop-server:") {
                break;
            }
            let fun = handler.get(id);
            if fun.is_none() {
                e_stream.write_all(&FlowControl::Close.to_be_bytes())?;
                e_stream.flush()?;
                continue;
            } else {
                e_stream.write_all(&FlowControl::Continue.to_be_bytes())?;
                e_stream.flush()?;
            };
            e_stream = fun.unwrap()(e_stream)?;
        }
        Ok(())
    }
}
