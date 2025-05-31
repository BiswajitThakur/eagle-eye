use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
    time::{self, Duration},
};

use eagle_eye_broadcaster::{ReceiverInfo, SenderInfo};

#[test]
fn test_running() {
    let interval = Arc::new(AtomicU64::new(200));
    let sender = SenderInfo::builder()
        .prefix("hello".into())
        .send_addr(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 96, 69, 96)),
            6969,
        ))
        .interval(interval)
        .socket_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .broadcast_addr(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 255, 255, 255)),
            7766,
        ))
        .build();
    let is_running_sender = Arc::new(AtomicBool::new(true));
    let is_running_receiver = Arc::new(AtomicBool::new(true));
    let v = {
        let is_running_sender_clone = Arc::clone(&is_running_sender);
        std::thread::spawn(move || sender.send(is_running_sender_clone))
    };
    let now = time::Instant::now();
    let mut count = Arc::new(AtomicU32::new(0));
    let socket = UdpSocket::bind("127.0.0.1:7766").unwrap();
    let receiver = ReceiverInfo {
        prefix: "hello".into(),
        is_running: is_running_receiver.clone(),
        block_ip: Vec::new(),
        block_ip_stack: Arc::new(Mutex::new(Vec::new())),
        buf: [0; 27],
        recv_buf_len: vec![15, 27],
        socket,
    };
    let t = {
        let count_clone = count.clone();
        std::thread::spawn(move || {
            for _ in receiver {
                count_clone.fetch_add(1, Ordering::Relaxed);
            }
        })
    };
    let mut flag_1 = false;
    loop {
        if now.elapsed() > Duration::from_secs(3) && now.elapsed() < Duration::from_secs(4) {
            flag_1 = true;
            let total_count = count.load(Ordering::Relaxed);
            assert!(total_count > 10);
            assert!(total_count < 22);
            break;
        }
    }
    assert!(flag_1);
    v.join().unwrap().unwrap();
}
