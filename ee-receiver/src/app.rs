/*
use std::{
    io::{self, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    sync::{Arc, atomic::AtomicUsize},
    time::Duration,
};

use ee_broadcaster::ReceiverInfo;
use ee_stream::{EStreamSync, FlowControl};
use ee_task::{ExeReceiverSync, create_task_registery};

use crate::utils::{handle_auth_on_receiver_sync, process_broadcast_data};

//  pub struct ReceiverTaskRegisterySync<const N: usize> {
//      inner: Vec<(&'static str, $t)>,
//  }
create_task_registery! {
    name: pub ReceiverTaskRegisterySync,
    handler: fn(
        EStreamSync,
    ) -> io::Result<EStreamSync>
}

pub struct ReceiverConfigSync {
    id: u128,
    key: [u8; 32],
    socket_addr: SocketAddr,
    broadcast_buf_size: usize,
    broadcast_data_prefix: &'static str,
    max_connection: usize,
    handler: ReceiverTaskRegisterySync,
}

impl<'a> Default for ReceiverConfigSync {
    fn default() -> Self {
        Self {
            id: 0,
            key: [0; 32],
            socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 6923),
            broadcast_buf_size: 8 * 1024,
            broadcast_data_prefix: "",
            max_connection: 4,
            handler: ReceiverTaskRegisterySync::default(),
        }
    }
}

impl ReceiverConfigSync {
    pub fn id(mut self, id: u128) -> Self {
        self.id = id;
        self
    }
    pub fn key(mut self, key: [u8; 32]) -> Self {
        self.key = key;
        self
    }
    pub fn socket_addr(mut self, addr: SocketAddr) -> Self {
        self.socket_addr = addr;
        self
    }
    pub fn broadcast_buf_size(mut self, size: usize) -> Self {
        self.broadcast_buf_size = size;
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
    pub fn register<A: ExeReceiverSync>(&mut self) -> &mut Self {
        self.handler.register(A::id(), A::execute_on_receiver);
        self
    }
}

pub struct AppSync {
    id: u128,
    key: [u8; 32],
    broadcast_socket_addr: SocketAddr,
    broadcast_buf_size: usize,
    broadcast_data_prefix: &'static str,
    max_connection: usize,
    count_connection: Arc<AtomicUsize>,
    handler: Arc<ReceiverTaskRegisterySync>,
}

impl AppSync {
    pub fn new(config: ReceiverConfigSync) -> Self {
        let ReceiverConfigSync {
            id,
            key,
            socket_addr: broadcast_socket_addr,
            broadcast_buf_size,
            broadcast_data_prefix,
            max_connection,
            handler,
        } = config;
        Self {
            id,
            key,
            broadcast_socket_addr,
            broadcast_buf_size,
            broadcast_data_prefix,
            max_connection,
            count_connection: Arc::new(AtomicUsize::new(0)),
            handler: Arc::new(handler),
        }
    }
}

impl AppSync {
    pub fn run(self) -> io::Result<()> {
        let mut receiver = ReceiverInfo::builder()
            .prefix(self.broadcast_data_prefix)
            .buffer_size(self.broadcast_buf_size)
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

impl AppSync {
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
        handler: Arc<ReceiverTaskRegisterySync>,
        r: &TcpStream,
        w: &TcpStream,
    ) -> io::Result<()> {
        let e_stream = handle_auth_on_receiver_sync(key, r, w)?;
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
}*/
