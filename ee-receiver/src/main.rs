mod handler;
mod receiver;
mod utils;

use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, atomic::AtomicUsize},
    time::Duration,
};

use ee_broadcaster::ReceiverInfo;

use crate::utils::{process, process_broadcast_data};

const BROADCAST_PORT: u16 = 6923;
const BROADCAST_DATA_PREFIX: &str = ":eagle-eye:";
const MAX_BROADCAST_DATA_LEN: usize = 2028;

const MAX_CONNECTIONS: usize = 3;

fn main() -> io::Result<()> {
    let key: [u8; 32] = [33; 32];
    let id: u128 = 123;

    let thread_counter = Arc::new(AtomicUsize::new(0));
    let handler = handler::handler::<512>(key);

    let mut receiver = ReceiverInfo::builder()
        .prefix(BROADCAST_DATA_PREFIX)
        .buffer([0; MAX_BROADCAST_DATA_LEN])
        .socket_addr(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            BROADCAST_PORT,
        ))
        .build()?;

    while let Ok(Some((addr, data))) = receiver.next() {
        let count = thread_counter.load(std::sync::atomic::Ordering::SeqCst);
        println!("Thread Count: {count}");
        if count >= MAX_CONNECTIONS {
            std::thread::sleep(Duration::from_millis(300));
            continue;
        }
        if let Some((addr, secret)) = process_broadcast_data(key, id, addr, data) {
            let handler = handler.clone();
            thread_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let count = thread_counter.clone();
            std::thread::spawn(move || {
                process(handler, count, addr, secret);
            });
        }
    }

    Ok(())
}
